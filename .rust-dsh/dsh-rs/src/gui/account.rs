//! Account views, a single-attempt browser sign-in registry, and account endpoint dispatch.

use cordis::Context;
use serde_json::{json, Value};

use super::wire::RpcResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

// Kept local so this slice builds without editing the shared module list.
#[path = "account_signin.rs"]
pub mod signin;
use signin::{GrantStore, SignInError, SignInRequest};

/// Platform sign-in configuration nested under `gui.account`.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AccountConfig {
    /// HTTPS origin serving the Platform protocol and browser pages.
    pub platform_origin: String,
}
impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            platform_origin: "https://platform.deepseek.com".into(),
        }
    }
}

/// One account provider with a single local attempt registry.
pub struct Account {
    pub(crate) origin: String,
    pub(crate) client_platform: String,
    pub(crate) platform: Arc<dyn signin::Platform>,
    pub(crate) browser: Arc<dyn signin::Browser>,
    pub(crate) callbacks: Arc<dyn signin::CallbackHost>,
    pub(crate) store: Arc<dyn GrantStore>,
    attempt: Mutex<Option<Attempt>>,
    signing_out: Mutex<()>,
    pub(crate) callback: Mutex<Option<(String, Arc<signin::CallbackRoute>)>>,
}
struct Attempt {
    view: Value,
    stop: CancellationToken,
    done: CancellationToken,
    deadline: u64,
}
impl Account {
    /// Construct a provider with injectable transport, browser, listener and grant storage.
    pub fn new(
        origin: &str,
        client_platform: &str,
        platform: Arc<dyn signin::Platform>,
        browser: Arc<dyn signin::Browser>,
        callbacks: Arc<dyn signin::CallbackHost>,
        store: Arc<dyn GrantStore>,
    ) -> Result<Arc<Self>, SignInError> {
        if !["web", "desktop-mac", "desktop-win"].contains(&client_platform) {
            return Err(SignInError::Protocol);
        }
        Ok(Arc::new(Self {
            origin: signin::platform_origin(origin)?,
            client_platform: client_platform.into(),
            platform,
            browser,
            callbacks,
            store,
            attempt: Mutex::new(None),
            signing_out: Mutex::new(()),
            callback: Mutex::new(None),
        }))
    }

    /// Production provider using the credential service and OS browser opener.
    pub fn system(
        origin: &str,
        store: Arc<super::credentials::CredentialStore>,
    ) -> Result<Arc<Self>, SignInError> {
        Self::new(
            origin,
            "web",
            Arc::new(signin::HttpPlatform::new(origin)?),
            Arc::new(signin::SystemBrowser),
            Arc::new(signin::LoopbackHost),
            Arc::new(RecordGrantStore { store }),
        )
    }

    /// Current safe projection; malformed local grants are storage failures.
    pub async fn get_state(&self) -> Result<Value, SignInError> {
        let stored = self.store.read().await?;
        if let Some(grant) = &stored {
            if grant["version"] != 1
                || grant["token"].as_str().is_none_or(str::is_empty)
                || grant["issuer"]
                    .as_str()
                    .is_none_or(|s| reqwest::Url::parse(s).is_err())
            {
                return Err(SignInError::Storage);
            }
        }
        let mut attempt = self
            .attempt
            .lock()
            .await
            .as_ref()
            .map(|a| a.view.clone())
            .unwrap_or(Value::Null);
        if stored.is_none() && attempt["phase"] == "succeeded" {
            attempt = Value::Null;
        }
        Ok(
            json!({"status": if stored.is_some() { "credential-stored" } else { "signed-out" }, "links": {"usageUrl": format!("{}/usage", self.origin), "topUpUrl": format!("{}/top_up", self.origin)}, "attempt": attempt}),
        )
    }

    /// Start at most one attempt; validate the origin even when an attempt already runs.
    pub async fn start_sign_in(
        self: &Arc<Self>,
        mut request: SignInRequest,
    ) -> Result<Value, SignInError> {
        let _lifecycle = self.signing_out.lock().await;
        request.callback_origin = signin::login_origin(&request.callback_origin)?;
        loop {
            let mut slot = self.attempt.lock().await;
            if let Some(active) = slot.as_ref() {
                if running(&active.view) {
                    drop(slot);
                    return self.get_state().await;
                }
                if !active.done.is_cancelled() {
                    let done = active.done.clone();
                    drop(slot);
                    done.cancelled().await;
                    continue;
                }
            }
            let id = uuid::Uuid::new_v4().to_string();
            let stop = CancellationToken::new();
            let done = CancellationToken::new();
            let deadline = signin::now_ms() + 600_000;
            *slot = Some(Attempt {
                view: json!({"id": id, "phase": "initializing"}),
                stop: stop.clone(),
                done: done.clone(),
                deadline,
            });
            // Capture the answer before scheduling effects: callers always see initializing.
            let attempt_view = slot.as_ref().map(|a| a.view.clone()).unwrap_or(Value::Null);
            drop(slot);
            let mut view = match self.get_state().await {
                Ok(view) => view,
                Err(error) => {
                    self.update(&id, "failed", Some(json!({"errorCode": error.code()})))
                        .await;
                    done.cancel();
                    return Err(error);
                }
            };
            view["attempt"] = attempt_view;
            let account = self.clone();
            tokio::spawn(async move {
                let result = Some(signin::run(account.clone(), &id, request, stop.clone()).await);
                let committing = {
                    let slot = account.attempt.lock().await;
                    slot.as_ref()
                        .is_some_and(|a| a.view["id"] == id && a.view["phase"] == "committing")
                };
                if committing {
                    done.cancelled().await;
                } else if stop.is_cancelled() {
                    account.update(&id, "cancelled", None).await;
                } else if let Some(result) = result {
                    match result {
                        Ok(()) => {}
                        Err(error) => {
                            account
                                .update(
                                    &id,
                                    if error == SignInError::Expired {
                                        "expired"
                                    } else {
                                        "failed"
                                    },
                                    Some(json!({"errorCode": error.code()})),
                                )
                                .await;
                        }
                    }
                } else {
                    account.update(&id, "cancelled", None).await;
                }
                stop.cancel();
                done.cancel();
            });
            return Ok(view);
        }
    }

    /// Unknown ids are a no-op. Once committing begins, cancellation waits for commit.
    pub async fn cancel_sign_in(&self, id: &str) -> Result<Value, SignInError> {
        let done = {
            let slot = self.attempt.lock().await;
            slot.as_ref().filter(|a| a.view["id"] == id).map(|a| {
                if running(&a.view) && a.view["phase"] != "committing" {
                    a.stop.cancel();
                }
                a.done.clone()
            })
        };
        if let Some(done) = done {
            done.cancelled().await;
        }
        self.get_state().await
    }
    /// Cancel before deleting, and serialize against starting a replacement attempt.
    pub async fn sign_out(&self) -> Result<Value, SignInError> {
        let _lifecycle = self.signing_out.lock().await;
        let done = {
            let slot = self.attempt.lock().await;
            slot.as_ref().map(|a| {
                a.stop.cancel();
                a.done.clone()
            })
        };
        if let Some(done) = done {
            done.cancelled().await;
        }
        self.store.delete().await?;
        *self.attempt.lock().await = None;
        self.get_state().await
    }

    pub(crate) async fn waiting(&self, id: &str) -> bool {
        self.attempt.lock().await.as_ref().is_some_and(|a| {
            a.view["id"] == id && a.view["phase"] == "waiting-browser" && !a.stop.is_cancelled()
        })
    }
    pub(crate) async fn deadline(&self, id: &str) -> u64 {
        self.attempt
            .lock()
            .await
            .as_ref()
            .filter(|a| a.view["id"] == id)
            .map(|a| a.deadline)
            .unwrap_or(0)
    }
    pub(crate) async fn update(&self, id: &str, phase: &str, extra: Option<Value>) {
        let mut slot = self.attempt.lock().await;
        if let Some(a) = slot.as_mut().filter(|a| a.view["id"] == id) {
            if let Some(view) = a.view.as_object_mut() {
                view.remove("authorizeUrl");
                view.insert("phase".into(), json!(phase));
                if let Some(Value::Object(extra)) = extra {
                    view.extend(extra);
                }
            }
        }
    }
    pub(crate) async fn commit(
        self: &Arc<Self>,
        id: &str,
        grant: Value,
    ) -> Result<(), SignInError> {
        // The lock makes the cancellation/commit boundary atomic. Commit itself is shielded
        // from the run-loop timeout, just as authorization.commit is in TypeScript.
        let mut slot = self.attempt.lock().await;
        let attempt = slot
            .as_mut()
            .filter(|a| a.view["id"] == id)
            .ok_or(SignInError::Protocol)?;
        if attempt.stop.is_cancelled() {
            return Err(SignInError::Expired);
        }
        attempt
            .view
            .as_object_mut()
            .map(|v| v.remove("authorizeUrl"));
        attempt.view["phase"] = json!("committing");
        let store = self.store.clone();
        let account = self.clone();
        let id = id.to_string();
        let done = attempt.done.clone();
        let task = tokio::spawn(async move {
            let result = store.write(grant).await;
            account
                .update(
                    &id,
                    if result.is_ok() {
                        "succeeded"
                    } else {
                        "failed"
                    },
                    if result.is_err() {
                        Some(json!({"errorCode": "storage"}))
                    } else {
                        None
                    },
                )
                .await;
            done.cancel();
            result
        });
        drop(slot);
        task.await.map_err(|_| SignInError::Storage)?
    }
}
fn running(view: &Value) -> bool {
    view["phase"].as_str().is_some_and(|p| {
        [
            "initializing",
            "waiting-browser",
            "exchanging",
            "committing",
        ]
        .contains(&p)
    })
}

/// The record the grant is written to.
const GRANT_RECORD: &str = "deepseek-account-platform/default";
/// The record the device id is written to.
const DEVICE_RECORD: &str = "deepseek-account-platform/device";

/// The grant store, kept in the credential document's records section.
struct RecordGrantStore {
    store: Arc<super::credentials::CredentialStore>,
}

impl GrantStore for RecordGrantStore {
    fn read(&self) -> crate::api::services::BoxFuture<Result<Option<Value>, SignInError>> {
        let store = self.store.clone();
        Box::pin(async move {
            let stored = store
                .read_record(GRANT_RECORD)
                .await
                .map_err(|_| SignInError::Storage)?;
            match stored {
                Some(record) if record["kind"] == "grant" => Ok(Some(record["payload"].clone())),
                Some(_) => Err(SignInError::Storage),
                None => Ok(None),
            }
        })
    }

    fn write(&self, grant: Value) -> crate::api::services::BoxFuture<Result<(), SignInError>> {
        let store = self.store.clone();
        Box::pin(async move {
            store
                .write_record(GRANT_RECORD, json!({"kind":"grant", "payload":grant}))
                .await
                .map_err(|_| SignInError::Storage)
        })
    }

    fn delete(&self) -> crate::api::services::BoxFuture<Result<(), SignInError>> {
        let store = self.store.clone();
        Box::pin(async move {
            store
                .delete_record(GRANT_RECORD)
                .await
                .map_err(|_| SignInError::Storage)
        })
    }

    fn device(&self) -> crate::api::services::BoxFuture<Result<String, SignInError>> {
        let store = self.store.clone();
        Box::pin(async move {
            let record = store.modify_record(DEVICE_RECORD, |current| {
                current.is_none().then(|| json!({"kind":"grant", "payload":{"id":uuid::Uuid::new_v4().to_string()}}))
            }).await.map_err(|_| SignInError::Storage)?.ok_or(SignInError::Storage)?;
            let id = record["payload"]["id"]
                .as_str()
                .ok_or(SignInError::Storage)?;
            if record["kind"] != "grant" || uuid::Uuid::parse_str(id).is_err() {
                return Err(SignInError::Storage);
            }
            Ok(id.to_string())
        })
    }
}

/// Serve the stateful account endpoints after descriptor validation.
/// The router must call this before the signed-out fallback dispatcher.
pub async fn dispatch_signin(
    account: &Arc<Account>,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let fields: &[&str] = match method {
        "account/startSignIn" => &["client", "callbackOrigin", "loginSource"],
        "account/cancelSignIn" => &["attemptId"],
        "account/signOut" => &["client"],
        "account/getState" => &[],
        _ => return None,
    };
    if let Err(failure) = super::router::check_descriptor(method, fields, payload) {
        return Some(RpcResult::failed(failure));
    }
    let args = payload;
    let result = match method {
        "account/startSignIn" => {
            account
                .start_sign_in(SignInRequest {
                    client: args["client"].clone(),
                    callback_origin: args["callbackOrigin"].as_str().unwrap_or("").into(),
                    login_source: args["loginSource"].as_str().unwrap_or("").into(),
                })
                .await
        }
        "account/cancelSignIn" => {
            account
                .cancel_sign_in(args["attemptId"].as_str().unwrap_or(""))
                .await
        }
        "account/signOut" => account.sign_out().await,
        _ => account.get_state().await,
    };
    Some(match result {
        Ok(view) => RpcResult::ok(view),
        Err(error) => RpcResult::failed(super::wire::Failure {
            code: super::wire::codes::INTERNAL.into(),
            message: error.to_string(),
            details: json!({}),
        }),
    })
}
use crate::api::services::AgentRegistryService;
use crate::types::AgentStatus;

/// The provider id whose runs count as account tasks.
const ACCOUNT_PROVIDER: &str = "deepseek-account";

/// The fallback projection when no account provider is installed.
pub(super) fn signed_out_view() -> Value {
    json!({
        "status": "signed-out",
        "attempt": null,
        "links": {
            "usageUrl": "https://platform.deepseek.com/usage",
            "topUpUrl": "https://platform.deepseek.com/top_up",
        },
    })
}

/// Serve one `account/*` unary endpoint, or `None` for another method. The
/// gateway has already checked the arguments against the inventory.
pub(super) fn dispatch(ctx: &Context, method: &str, _payload: &Value) -> Option<RpcResult> {
    Some(match method {
        "account/getState" | "account/signOut" | "account/cancelSignIn" => {
            RpcResult::ok(signed_out_view())
        }
        "account/getProfile" | "account/getBalance" | "account/getUnnotifiedBonuses" => {
            RpcResult::ok(Value::Null)
        }
        "account/ackBonusNotified" => RpcResult::ok(json!(false)),
        "account/hasRunningAccountTasks" => RpcResult::ok(json!(running_account_task(ctx))),
        _ => return None,
    })
}

/// Whether any live agent is mid-turn on the account provider.
fn running_account_task(ctx: &Context) -> bool {
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return false;
    };
    agents.list().iter().any(|agent| {
        agent.status() == AgentStatus::Running
            && agent
                .session()
                .request_header()
                .is_some_and(|header| header.config.provider == ACCOUNT_PROVIDER)
    })
}
