//! Sign-in contracts exercised with no sockets, platform requests, or OS browser launches.
use dsh_rs::api::services::BoxFuture;
use dsh_rs::gui::account::{signin::*, Account};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

struct Transport {
    calls: Mutex<Vec<(String, Value)>>,
    answer: Mutex<Option<Result<PlatformResponse, SignInError>>>,
}
impl Platform for Transport {
    fn request(
        &self,
        method: &str,
        body: Value,
        _: BTreeMap<String, String>,
    ) -> BoxFuture<Result<PlatformResponse, SignInError>> {
        self.calls.lock().unwrap().push((method.into(), body));
        let answer = if method == "auth_cancel" {
            Some(Ok(response(json!({"code":0,"data":{"biz_code":0}}))))
        } else {
            self.answer.lock().unwrap().take()
        };
        Box::pin(async move {
            match answer {
                Some(a) => a,
                None => std::future::pending().await,
            }
        })
    }
}
#[derive(Default)]
struct Effects {
    route: Mutex<Option<Arc<CallbackRoute>>>,
    opened: Mutex<Vec<String>>,
    grant: Mutex<Option<Value>>,
}
impl Browser for Effects {
    fn open(&self, url: String) -> BoxFuture<Result<(), SignInError>> {
        self.opened.lock().unwrap().push(url);
        Box::pin(async { Ok(()) })
    }
}
impl CallbackHost for Effects {
    fn listen(
        &self,
        _: String,
        route: Arc<CallbackRoute>,
        _: CancellationToken,
    ) -> BoxFuture<Result<(), SignInError>> {
        *self.route.lock().unwrap() = Some(route);
        Box::pin(async { Ok(()) })
    }
}
impl GrantStore for Effects {
    fn read(&self) -> BoxFuture<Result<Option<Value>, SignInError>> {
        let g = self.grant.lock().unwrap().clone();
        Box::pin(async move { Ok(g) })
    }
    fn write(&self, g: Value) -> BoxFuture<Result<(), SignInError>> {
        *self.grant.lock().unwrap() = Some(g);
        Box::pin(async { Ok(()) })
    }
    fn delete(&self) -> BoxFuture<Result<(), SignInError>> {
        *self.grant.lock().unwrap() = None;
        Box::pin(async { Ok(()) })
    }
    fn device(&self) -> BoxFuture<Result<String, SignInError>> {
        Box::pin(async { Ok("c7e3e683-ed46-4990-9983-a94b2b26b183".into()) })
    }
}
fn rig(
    answer: Option<Result<PlatformResponse, SignInError>>,
) -> (Arc<Account>, Arc<Transport>, Arc<Effects>) {
    let t = Arc::new(Transport {
        calls: Mutex::new(vec![]),
        answer: Mutex::new(answer),
    });
    let e = Arc::new(Effects::default());
    (
        Account::new(
            "https://platform.deepseek.com",
            "web",
            t.clone(),
            e.clone(),
            e.clone(),
            e.clone(),
        )
        .unwrap(),
        t,
        e,
    )
}
fn request() -> SignInRequest {
    SignInRequest {
        client: json!({"version":"1","locale":"zh-TW","timezoneOffsetSeconds":28800}),
        callback_origin: "http://localhost:3081".into(),
        login_source: "web".into(),
    }
}
fn response(v: Value) -> PlatformResponse {
    PlatformResponse {
        status: 200,
        body: serde_json::to_vec(&v).unwrap(),
    }
}
async fn phase(a: &Account, expected: &str) -> Value {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let v = a.get_state().await.unwrap();
            if v["attempt"]["phase"] == expected {
                return v;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}
#[test]
fn loopback_origin_contract() {
    for (v, want) in [
        ("http://localhost:80", "http://localhost:80"),
        ("HTTP://LOCALHOST:003081/", "http://localhost:3081"),
        ("http://127.0.0.1:1", "http://127.0.0.1:1"),
        ("http://[::1]:65535", "http://[::1]:65535"),
    ] {
        assert_eq!(login_origin(v).unwrap(), want)
    }
    for v in [
        "http://localhost",
        "http://localhost:0",
        "http://localhost:3/path",
        "http://localhost:3?x",
        "http://localhost:3#x",
        "http://u@localhost:3",
        "http://example.com:3",
        "https://localhost:3",
        "http://localhost:65536",
        "http://127.1:3",
        " http://localhost:3",
    ] {
        assert_eq!(login_origin(v), Err(SignInError::Protocol), "{v}")
    }
}
#[tokio::test]
async fn initializing_is_reused_and_cancellation_matches_ts_no_op() {
    let (a, t, _) = rig(None);
    let first = a.start_sign_in(request()).await.unwrap();
    assert_eq!(first["attempt"]["phase"], "initializing");
    assert!(uuid::Uuid::parse_str(first["attempt"]["id"].as_str().unwrap()).is_ok());
    assert_eq!(a.start_sign_in(request()).await.unwrap(), first);
    assert_eq!(a.cancel_sign_in("unknown").await.unwrap(), first);
    let cancelled = a
        .cancel_sign_in(first["attempt"]["id"].as_str().unwrap())
        .await
        .unwrap();
    assert_eq!(cancelled["attempt"]["phase"], "cancelled");
    assert_eq!(a.cancel_sign_in("unknown").await.unwrap(), cancelled);
    assert!(t.calls.lock().unwrap().len() <= 1);
}
#[tokio::test]
async fn failures_pin_transport_vs_envelope_classification() {
    for (answer, code) in [
        (Ok(response(json!({"token":"not-an-envelope"}))), "protocol"),
        (Err(SignInError::NoResponse), "no-response"),
        (
            Ok(PlatformResponse {
                status: 503,
                body: vec![],
            }),
            "network",
        ),
    ] {
        let (a, _, _) = rig(Some(answer));
        a.start_sign_in(request()).await.unwrap();
        let view = phase(&a, "failed").await;
        assert_eq!(view["attempt"]["errorCode"], code);
        assert!(view["attempt"].get("authorizeUrl").is_none());
    }
}
#[tokio::test]
async fn wrong_state_never_exchanges_and_browser_gets_validated_url() {
    let url = "https://platform.deepseek.com/dsh/authorize?id=test";
    let (a, t, e) = rig(Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{"authorize_url":url,"authorize_id":"a","expires_in":300}}}),
    ))));
    let initial = a.start_sign_in(request()).await.unwrap();
    let waiting = phase(&a, "waiting-browser").await;
    assert_eq!(waiting["attempt"]["authorizeUrl"], url);
    assert!(waiting["attempt"]["expiresAt"].as_u64().is_some());
    let route = e.route.lock().unwrap().clone().unwrap();
    assert!(matches!(
        route
            .receive("GET", "/oauth/callback?code=secret&state=wrong")
            .await,
        Err(400)
    ));
    let calls = t.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "auth_init");
    assert_eq!(calls[0].1["locale"], "zh_CN");
    assert_eq!(
        calls[0].1["redirect_uri"],
        "http://localhost:3081/oauth/callback"
    );
    assert_eq!(calls[0].1["code_challenge_method"], "S256");
    assert_eq!(calls[0].1["code_challenge"].as_str().unwrap().len(), 43);
    assert_eq!(e.opened.lock().unwrap().as_slice(), [url]);
    let state = calls[0].1["state"].as_str().unwrap();
    assert!(matches!(
        route
            .receive(
                "GET",
                &format!("/oauth/callback?code=a&state={state}&state={state}")
            )
            .await,
        Err(400)
    ));
    assert!(matches!(
        route
            .receive(
                "GET",
                &format!("/oauth/callback?code=a&code=b&state={state}")
            )
            .await,
        Err(400)
    ));
    a.cancel_sign_in(initial["attempt"]["id"].as_str().unwrap())
        .await
        .unwrap();
    assert!(matches!(
        route
            .receive("GET", &format!("/oauth/callback?code=a&state={state}"))
            .await,
        Err(410)
    ));
}

#[tokio::test]
async fn matching_callback_exchanges_pkce_and_commits_only_safe_view() {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let (a, t, e) = rig(Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{"authorize_url":"https://platform.deepseek.com/dsh/authorize", "authorize_id":"a", "expires_in":300}}}),
    ))));
    a.start_sign_in(request()).await.unwrap();
    phase(&a, "waiting-browser").await;
    let init = t.calls.lock().unwrap()[0].1.clone();
    *t.answer.lock().unwrap() = Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{"token":"secret-test-token","authorized_url":"https://platform.deepseek.com/dsh/authorized"}}}),
    )));
    let route = e.route.lock().unwrap().clone().unwrap();
    let callback = route
        .receive(
            "GET",
            &format!(
                "/oauth/callback?code=one-use-code&state={}",
                init["state"].as_str().unwrap()
            ),
        )
        .await
        .unwrap();
    assert_eq!(
        callback.await.unwrap().unwrap(),
        "https://platform.deepseek.com/dsh/authorized?login_source=web"
    );
    let view = phase(&a, "succeeded").await;
    assert_eq!(view["status"], "credential-stored");
    assert!(view["attempt"].get("authorizeUrl").is_none());
    assert!(!view.to_string().contains("secret-test-token"));
    let calls = t.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].0, "auth_exchange");
    let verifier = calls[1].1["code_verifier"].as_str().unwrap();
    assert_eq!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(verifier)
            .unwrap()
            .len(),
        32
    );
    assert_eq!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(verifier.as_bytes())),
        init["code_challenge"]
    );
    assert_eq!(
        e.grant.lock().unwrap().as_ref().unwrap(),
        &json!({"version":1,"token":"secret-test-token","issuer":"https://platform.deepseek.com"})
    );
}

fn gateway(account: Arc<Account>) -> dsh_rs::gui::Gateway {
    use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
    let config = RouterConfig::default();
    Gateway::new(
        Router::new(config).with_account(account),
        Auth {
            token: "test".into(),
        },
        None,
    )
}
async fn rpc(g: &dsh_rs::gui::Gateway, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert("host", "localhost:3081".parse().unwrap());
    headers.insert("authorization", "Bearer test".parse().unwrap());
    let body = serde_json::to_vec(
        &json!({"type":"client-request","rpcId":"1","method":method,"payload":{"args":args}}),
    )
    .unwrap();
    let (reply, _) = g
        .route(
            &cordis::Context::new(),
            &http::Method::POST,
            "/api",
            &headers,
            &body,
        )
        .await;
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}
#[tokio::test]
async fn gateway_signin_arguments_reuse_refusal_and_signout() {
    let (a, _, e) = rig(None);
    let g = gateway(a.clone());
    let args = json!({"client":request().client,"callbackOrigin":"http://localhost:3081","loginSource":"web"});
    let first = rpc(&g, "account/startSignIn", args.clone()).await;
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["value"]["attempt"]["phase"], "initializing");
    let second = rpc(&g, "account/startSignIn", args.clone()).await;
    assert_eq!(
        first["value"]["attempt"]["id"],
        second["value"]["attempt"]["id"]
    );
    let mut bad = args;
    bad["callbackOrigin"] = json!("https://evil.example:3081");
    let refused = rpc(&g, "account/startSignIn", bad).await;
    assert_eq!(refused["error"]["message"], "account: protocol");
    *e.grant.lock().unwrap() =
        Some(json!({"version":1,"token":"t","issuer":"https://platform.deepseek.com"}));
    assert_eq!(
        rpc(&g, "account/signOut", json!({"client":request().client})).await["value"]["status"],
        "signed-out"
    );
    assert!(e.grant.lock().unwrap().is_none());
    assert_eq!(
        rpc(&g, "account/getState", json!({})).await["value"]["status"],
        "signed-out"
    );
}

#[tokio::test]
async fn gateway_callback_uses_existing_listener_and_checks_state() {
    let (_, t, e) = rig(Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{
            "authorize_url":"https://platform.deepseek.com/dsh/authorize","authorize_id":"a","expires_in":300
        }}}),
    ))));
    let a = Account::new(
        "https://platform.deepseek.com",
        "web",
        t.clone(),
        e.clone(),
        Arc::new(LoopbackHost),
        e.clone(),
    )
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let g = Arc::new(gateway(a.clone()));
    let task = tokio::spawn(dsh_rs::gui::server::serve(
        g.clone(),
        cordis::Context::new(),
        listener,
    ));
    let mut req = request();
    req.callback_origin = origin.clone();
    a.start_sign_in(req).await.unwrap();
    phase(&a, "waiting-browser").await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let bad = client
        .get(format!("{origin}/oauth/callback?code=c&state=wrong"))
        .header("sec-fetch-site", "cross-site")
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
    assert_eq!(t.calls.lock().unwrap().len(), 1);
    let state = t.calls.lock().unwrap()[0].1["state"]
        .as_str()
        .unwrap()
        .to_string();
    *t.answer.lock().unwrap() = Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{
            "token":"test-token","authorized_url":"https://platform.deepseek.com/dsh/authorized"
        }}}),
    )));
    let good = client
        .get(format!("{origin}/oauth/callback?code=c&state={state}"))
        .send()
        .await
        .unwrap();
    assert_eq!(good.status(), 302);
    assert_eq!(t.calls.lock().unwrap()[1].0, "auth_exchange");
    task.abort();
}

#[tokio::test]
async fn cancelling_exchange_sends_remote_cancel_and_prevents_commit() {
    let (a, t, e) = rig(Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{
            "authorize_url":"https://platform.deepseek.com/dsh/authorize","authorize_id":"remote-id","expires_in":300
        }}}),
    ))));
    a.start_sign_in(request()).await.unwrap();
    phase(&a, "waiting-browser").await;
    let state = t.calls.lock().unwrap()[0].1["state"]
        .as_str()
        .unwrap()
        .to_string();
    let route = e.route.lock().unwrap().clone().unwrap();
    let response = route
        .receive("GET", &format!("/oauth/callback?code=c&state={state}"))
        .await
        .unwrap();
    phase(&a, "exchanging").await;
    assert_eq!(a.sign_out().await.unwrap()["status"], "signed-out");
    assert!(response.await.is_err());
    assert!(e.grant.lock().unwrap().is_none());
    let calls = t.calls.lock().unwrap();
    let cancel = calls
        .iter()
        .find(|(method, _)| method == "auth_cancel")
        .expect("cancel before signout returns");
    assert_eq!(cancel.1["authorize_id"], "remote-id");
    assert!(cancel.1["code_verifier"].as_str().is_some());
}

#[tokio::test]
async fn expiry_sends_remote_cancel_before_attempt_finishes() {
    let (a, t, _) = rig(Some(Ok(response(
        json!({"code":0,"data":{"biz_code":0,"biz_data":{
            "authorize_url":"https://platform.deepseek.com/dsh/authorize","authorize_id":"remote-id","expires_in":0.01
        }}}),
    ))));
    a.start_sign_in(request()).await.unwrap();
    phase(&a, "expired").await;
    assert!(t
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|(method, _)| method == "auth_cancel"));
}
