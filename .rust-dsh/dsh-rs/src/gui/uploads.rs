//! `fileUploads/upload`, the raw `POST /api/session/uploadFileBinary` route, and
//! the receipts a prompt later redeems.
//!
//! An upload is stored first and staged second: the store keeps the bytes, and
//! a receipt held here per session names them. A prompt cites the receipt, not
//! the bytes, so a client uploads once and a retried prompt cites the same file.
//! A receipt belongs to the session it was minted for; another session cannot
//! redeem it. It is retired when the prompt that used it reaches the log.

use std::collections::HashMap;
use std::sync::Mutex;

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, check_descriptor, require_session};
use super::wire::{Failure, RpcResult};
use crate::attachments::{decode_canonical_base64, AttachmentError, AttachmentStore};
use crate::types::FileRef;

const UPLOAD: &str = "fileUploads/upload";

/// What a receipt was bound to before a prompt claimed it.
#[derive(Debug, Default)]
pub struct Binding {
    previous: Vec<(String, Option<String>)>,
}

/// A stored file waiting for a prompt to cite it.
#[derive(Debug, Clone)]
struct Staged {
    file: FileRef,
    /// The message that cited this receipt, once one has.
    request_id: Option<String>,
}

/// The upload service: the store, its size limit, and every session's receipts.
pub struct Uploads {
    store: Option<AttachmentStore>,
    max_upload_bytes: usize,
    staged: Mutex<HashMap<String, HashMap<String, Staged>>>,
}

impl Uploads {
    /// Build the service. Without a store every upload is refused as
    /// unavailable; a limit of zero would refuse every file and is rejected.
    pub fn new(store: Option<AttachmentStore>, max_upload_bytes: usize) -> Result<Self, String> {
        if max_upload_bytes == 0 {
            return Err("the upload size limit must be at least 1 byte".to_string());
        }
        Ok(Uploads {
            store,
            max_upload_bytes,
            staged: Mutex::default(),
        })
    }

    /// The largest request body worth reading: the biggest file as base64, plus
    /// room for the JSON around it.
    pub fn max_body_bytes(&self) -> usize {
        self.max_upload_bytes
            .saturating_mul(4)
            .div_ceil(3)
            .saturating_add(64 * 1024)
    }

    /// The file a session's receipt names, if the session minted it.
    pub fn resolve(&self, session_id: &str, receipt_id: &str) -> Option<FileRef> {
        let staged = self.staged.lock().unwrap();
        staged
            .get(session_id)?
            .get(receipt_id)
            .map(|entry| entry.file.clone())
    }

    /// Remember which prompt cites these receipts. A receipt that vanished since
    /// it was resolved (its session ended) makes the prompt unadmittable.
    ///
    /// Like the TypeScript service, a receipt a second prompt cites moves to the
    /// second prompt; the returned [`Binding`] puts the earlier owners back if
    /// this prompt is not admitted.
    pub fn bind(
        &self,
        session_id: &str,
        receipt_ids: &[String],
        request_id: &str,
    ) -> Option<Binding> {
        let mut staged = self.staged.lock().unwrap();
        let Some(session) = staged.get_mut(session_id) else {
            return receipt_ids.is_empty().then(Binding::default);
        };
        if receipt_ids.iter().any(|id| !session.contains_key(id)) {
            return None;
        }
        let mut previous = Vec::with_capacity(receipt_ids.len());
        for id in receipt_ids {
            if let Some(entry) = session.get_mut(id) {
                previous.push((id.clone(), entry.request_id.replace(request_id.to_string())));
            }
        }
        Some(Binding { previous })
    }

    /// Undo [`Uploads::bind`] for a prompt that was not admitted. A receipt that
    /// was retired in the meantime stays gone.
    pub fn rollback(&self, session_id: &str, binding: Binding) {
        let mut staged = self.staged.lock().unwrap();
        let Some(session) = staged.get_mut(session_id) else {
            return;
        };
        for (id, previous) in binding.previous.into_iter().rev() {
            if let Some(entry) = session.get_mut(&id) {
                entry.request_id = previous;
            }
        }
    }

    /// Drop every receipt the prompt `request_id` cited.
    pub fn retire(&self, session_id: &str, request_id: &str) {
        let mut staged = self.staged.lock().unwrap();
        if let Some(session) = staged.get_mut(session_id) {
            session.retain(|_, entry| entry.request_id.as_deref() != Some(request_id));
            if session.is_empty() {
                staged.remove(session_id);
            }
        }
    }

    /// Drop a session's receipts, because the session is gone.
    pub fn forget_session(&self, session_id: &str) {
        self.staged.lock().unwrap().remove(session_id);
    }

    /// Listen for the log events that retire receipts. Call once per context.
    pub async fn observe(self: &std::sync::Arc<Self>, ctx: &Context) -> Result<(), cordis::Error> {
        use crate::api::events::{on, SessionDisposedPayload, SessionEventPayload};
        use crate::types::{MessageSource, SessionEventData};
        let uploads = std::sync::Arc::clone(self);
        // The listeners live as long as the context that registered them.
        on::<SessionEventPayload, _>(ctx, move |_ctx, payload| {
            let uploads = std::sync::Arc::clone(&uploads);
            Box::pin(async move {
                if let SessionEventData::UserMessage { message } = &payload.event.data {
                    if matches!(message.source, MessageSource::User { .. }) {
                        uploads.retire(&payload.session, &message.id);
                    }
                }
                Ok(Value::Null)
            })
        })
        .await?;
        let uploads = std::sync::Arc::clone(self);
        on::<SessionDisposedPayload, _>(ctx, move |_ctx, payload| {
            let uploads = std::sync::Arc::clone(&uploads);
            Box::pin(async move {
                uploads.forget_session(&payload.session);
                Ok(Value::Null)
            })
        })
        .await?;
        Ok(())
    }

    /// Store `data` and stage a receipt for it in `session_id`.
    async fn commit(
        &self,
        ctx: &Context,
        endpoint: &str,
        session_id: &str,
        data: Vec<u8>,
        name: Option<String>,
    ) -> Result<Value, Failure> {
        let store = self.store.clone().ok_or_else(|| no_store(endpoint))?;
        if data.len() > self.max_upload_bytes {
            return Err(attachment_failure(
                "FILE_TOO_LARGE",
                &format!(
                    "File exceeds the {} byte upload limit.",
                    self.max_upload_bytes
                ),
            ));
        }
        let saved = tokio::task::spawn_blocking(move || store.save_file(&data, name.as_deref()))
            .await
            .map_err(|error| internal(format!("failed to store file upload: {error}")))?;
        let file = saved.map_err(|error| store_failure(ctx, error))?;
        let receipt_id = uuid::Uuid::new_v4().to_string();
        self.staged
            .lock()
            .unwrap()
            .entry(session_id.to_string())
            .or_default()
            .insert(
                receipt_id.clone(),
                Staged {
                    file: file.clone(),
                    request_id: None,
                },
            );
        // The session may have ended while the bytes were being written. Checked
        // after staging, so a disposal either precedes this check and is seen, or
        // follows the insert and clears it.
        if let Err(gone) = require_session(ctx, endpoint, session_id) {
            self.forget_session(session_id);
            return Err(gone);
        }
        Ok(json!({ "receiptId": receipt_id, "file": file_json(&file) }))
    }
}

fn file_json(file: &FileRef) -> Value {
    json!({ "attachmentId": file.attachment_id, "name": file.name, "bytes": file.bytes })
}

/// A refusal the page can route on: `details.reason` is the stable code.
pub(super) fn attachment_failure(reason: &str, message: &str) -> Failure {
    Failure {
        code: "session/attachment-invalid".to_string(),
        message: message.to_string(),
        details: json!({ "reason": reason }),
    }
}

fn internal(message: String) -> Failure {
    Failure {
        code: super::wire::codes::INTERNAL.to_string(),
        message,
        details: json!({}),
    }
}

/// Uploads were not configured: there is no directory to keep files in.
fn no_store(endpoint: &str) -> Failure {
    Failure::at(
        super::wire::codes::SERVICE_UNAVAILABLE,
        "file uploads need gui.attachmentsDir, which is not set",
        endpoint,
    )
}

/// A receipt the session never minted, or one already spent.
pub(super) fn file_not_staged() -> Failure {
    attachment_failure("FILE_NOT_STAGED", "File was not uploaded for this session.")
}

/// Report a store failure. The cause goes to the log; the page gets the reason.
fn store_failure(ctx: &Context, error: AttachmentError) -> Failure {
    if let Some(cause) = &error.cause {
        ctx.logger().log_event(
            cordis::logger::LogLevel::Warn,
            "gui".to_string(),
            None,
            format!("attachment store: {}: {cause}", error.code),
        );
    }
    attachment_failure(error.code, &error.message)
}

/// Serve `fileUploads/upload`; `None` for every other endpoint.
pub(super) async fn dispatch(
    router: &super::router::Router,
    ctx: &Context,
    endpoint: &str,
    payload: &Value,
) -> Option<RpcResult> {
    if endpoint != UPLOAD {
        return None;
    }
    Some(match upload(router, ctx, payload).await {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

async fn upload(
    router: &super::router::Router,
    ctx: &Context,
    payload: &Value,
) -> Result<Value, Failure> {
    check_descriptor(UPLOAD, &["agentId", "request"], payload)?;
    let session_id = payload["agentId"]
        .as_str()
        .ok_or_else(|| boundary(UPLOAD, "agentId"))?;
    let request = payload["request"]
        .as_object()
        .ok_or_else(|| boundary(UPLOAD, "request"))?;
    let data = request
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| boundary(UPLOAD, "request"))?;
    let name = match request.get("name") {
        None => None,
        Some(Value::String(name)) => Some(name.clone()),
        Some(_) => return Err(boundary(UPLOAD, "request")),
    };
    require_session(ctx, UPLOAD, session_id)?;
    let uploads = router.uploads();
    // Cheap bound first: three quarters of the text, less up to two padding bytes.
    if (data.len() / 4 * 3).saturating_sub(2) > uploads.max_upload_bytes {
        return Err(attachment_failure(
            "FILE_TOO_LARGE",
            &format!(
                "File exceeds the {} byte upload limit.",
                uploads.max_upload_bytes
            ),
        ));
    }
    let bytes = decode_canonical_base64(data).ok_or_else(|| {
        attachment_failure(
            "INVALID_FILE_BASE64",
            "File upload is not canonical base64.",
        )
    })?;
    uploads.commit(ctx, UPLOAD, session_id, bytes, name).await
}

/// Serve one raw-byte upload: the body is the file. Returns the value the
/// `{ok, value | error}` reply carries.
pub(super) async fn upload_stream(
    router: &super::router::Router,
    ctx: &Context,
    session_id: &str,
    name: Option<String>,
    body: &[u8],
) -> Result<Value, Failure> {
    const ENDPOINT: &str = "session/uploadFileBinary";
    require_session(ctx, ENDPOINT, session_id)?;
    router
        .uploads()
        .commit(ctx, ENDPOINT, session_id, body.to_vec(), name)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uploads() -> Uploads {
        Uploads::new(None, 1024).unwrap()
    }

    fn stage(uploads: &Uploads, session: &str, receipt: &str) {
        uploads
            .staged
            .lock()
            .unwrap()
            .entry(session.to_string())
            .or_default()
            .insert(
                receipt.to_string(),
                Staged {
                    file: FileRef {
                        attachment_id: format!("sha256:{}", "0".repeat(64)),
                        name: "n".to_string(),
                        bytes: 0,
                    },
                    request_id: None,
                },
            );
    }

    #[test]
    fn a_receipt_resolves_only_in_the_session_that_minted_it() {
        let uploads = uploads();
        stage(&uploads, "s1", "r1");
        assert!(uploads.resolve("s1", "r1").is_some());
        assert!(uploads.resolve("s2", "r1").is_none());
        assert!(uploads.resolve("s1", "r2").is_none());
    }

    #[test]
    fn a_retired_prompt_takes_only_its_own_receipts() {
        let uploads = uploads();
        stage(&uploads, "s", "a");
        stage(&uploads, "s", "b");
        assert!(uploads.bind("s", &["a".to_string()], "req-1").is_some());
        uploads.retire("s", "req-1");
        assert!(uploads.resolve("s", "a").is_none());
        assert!(
            uploads.resolve("s", "b").is_some(),
            "an unrelated receipt stays"
        );
    }

    #[test]
    fn binding_a_vanished_receipt_is_refused_and_binds_nothing() {
        let uploads = uploads();
        stage(&uploads, "s", "a");
        assert!(uploads
            .bind("s", &["a".to_string(), "gone".to_string()], "req")
            .is_none());
        uploads.retire("s", "req");
        assert!(
            uploads.resolve("s", "a").is_some(),
            "a refused bind leaves no mark"
        );
    }

    #[test]
    fn rolling_back_restores_the_prompt_that_held_the_receipt_before() {
        let uploads = uploads();
        stage(&uploads, "s", "a");
        let _first = uploads.bind("s", &["a".to_string()], "first").unwrap();
        let second = uploads.bind("s", &["a".to_string()], "second").unwrap();
        uploads.rollback("s", second);
        uploads.retire("s", "first");
        assert!(
            uploads.resolve("s", "a").is_none(),
            "the first prompt landing still spends it after the second is undone"
        );
    }

    #[test]
    fn a_disposed_session_forgets_its_receipts() {
        let uploads = uploads();
        stage(&uploads, "s", "a");
        uploads.forget_session("s");
        assert!(uploads.resolve("s", "a").is_none());
    }

    #[test]
    fn a_zero_upload_limit_is_refused() {
        assert!(Uploads::new(None, 0).is_err());
    }

    #[test]
    fn the_body_limit_leaves_room_for_base64_overhead() {
        assert!(uploads().max_body_bytes() >= 1024 * 4 / 3);
    }
}
