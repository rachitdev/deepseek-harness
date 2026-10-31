//! Images in prompts: the wire part, the admission order, and
//! `session/attachment`, which reads an image back for a session that logged it.
//!
//! Admission follows the TypeScript host, step for step: the selected model
//! must accept images; every payload must be canonical base64; the batch must
//! fit the count and byte limits and use accepted types; each image is then
//! checked and normalized before anything is stored. Admissions for one
//! session run one at a time, so two prompts' images land in the order the
//! prompts arrived.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, check_descriptor, persistence, require_session};
use super::uploads::attachment_failure;
use super::wire::{Failure, RpcResult};
use crate::attachments::{decode_canonical_base64, AttachmentError, AttachmentStore};
use crate::images::{ImageConfig, ImageLimits, ImageModels, Normalization};
use crate::types::{ContentBlock, ImageRef, SessionEvent, SessionEventData};

const ATTACHMENT: &str = "session/attachment";

/// One image as the prompt carries it, before admission.
#[derive(Debug, Clone)]
pub struct WireImage {
    pub media_type: String,
    pub data: String,
    pub name: Option<String>,
}

/// The image policy of a deployment and the store it saves into.
pub struct ImageAdmission {
    store: Option<AttachmentStore>,
    limits: ImageLimits,
    normalization: Normalization,
    models: ImageModels,
    /// One lock per session with an admission in flight.
    sessions: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

impl Default for ImageAdmission {
    /// No store and no image-capable model: every image is refused the way the
    /// TypeScript host refuses a model without image input.
    fn default() -> Self {
        ImageAdmission::new(None, &ImageConfig::default(), ImageModels::default())
    }
}

impl ImageAdmission {
    pub fn new(store: Option<AttachmentStore>, config: &ImageConfig, models: ImageModels) -> Self {
        ImageAdmission {
            store,
            limits: config.limits(),
            normalization: config.normalization(),
            models,
            sessions: Mutex::default(),
        }
    }

    pub fn limits(&self) -> &ImageLimits {
        &self.limits
    }

    pub fn models(&self) -> &ImageModels {
        &self.models
    }

    /// The largest request body an image prompt can need: every image at the
    /// aggregate limit as base64. Zero while no model accepts images, so an
    /// unused feature does not raise the cap on what is read before auth.
    pub fn max_body_bytes(&self) -> usize {
        if self.models.is_empty() {
            return 0;
        }
        usize::try_from(
            self.limits
                .max_message_image_bytes
                .saturating_mul(4)
                .div_ceil(3),
        )
        .unwrap_or(usize::MAX)
        .saturating_add(64 * 1024)
    }

    /// Hold the session's admission lock until the returned guard drops.
    pub async fn serialize(&self, session_id: &str) -> Serialized<'_> {
        let lock = self
            .sessions
            .lock()
            .unwrap()
            .entry(session_id.to_string())
            .or_default()
            .clone();
        let guard = lock.clone().lock_owned().await;
        Serialized {
            admission: self,
            session_id: session_id.to_string(),
            guard: Some(guard),
            lock: Some(lock),
        }
    }

    /// Whether images can be saved at all.
    pub fn has_store(&self) -> bool {
        self.store.is_some()
    }

    /// The model gate: refuse before any bytes are examined.
    pub fn require_model(&self, provider: &str, model: &str) -> Result<(), Failure> {
        if self.models.accepts(provider, model) {
            return Ok(());
        }
        Err(attachment_failure(
            "MODEL_DOES_NOT_SUPPORT_IMAGES",
            &format!("Model \"{model}\" does not support image input."),
        ))
    }

    /// Admit one prompt's images: check, normalize and store them, in order.
    /// Blocking: call from a blocking context.
    pub fn admit(&self, images: &[WireImage]) -> Result<Vec<ImageRef>, AttachmentError> {
        let mut decoded = Vec::with_capacity(images.len());
        for image in images {
            match decode_canonical_base64(&image.data) {
                Some(bytes) if !image.data.is_empty() => decoded.push(bytes),
                _ => {
                    return Err(AttachmentError::new(
                        "INVALID_IMAGE_BASE64",
                        "Image upload is not canonical base64.",
                    ))
                }
            }
        }
        if images.len() > self.limits.max_images_per_message {
            return Err(AttachmentError::new(
                "TOO_MANY_IMAGES",
                "Image batch exceeds the configured image-count limit.",
            ));
        }
        let total: u64 = decoded.iter().map(|bytes| bytes.len() as u64).sum();
        if total > self.limits.max_message_image_bytes {
            return Err(AttachmentError::new(
                "IMAGES_TOO_LARGE",
                "Image batch exceeds the configured aggregate image-byte limit.",
            ));
        }
        if let Some(image) = images
            .iter()
            .find(|image| !self.limits.media_types.contains(&image.media_type))
        {
            return Err(AttachmentError {
                code: "UNSUPPORTED_IMAGE_TYPE",
                message: format!(
                    "Image type {} is not accepted by this deployment.",
                    image.media_type
                ),
                cause: None,
            });
        }
        let mut prepared = Vec::with_capacity(images.len());
        for (image, bytes) in images.iter().zip(&decoded) {
            prepared.push(crate::images::prepare(
                bytes,
                &image.media_type,
                image.name.as_deref(),
                &self.limits,
                &self.normalization,
            )?);
        }
        let store = self.store.as_ref().ok_or_else(|| {
            AttachmentError::new(
                "ATTACHMENT_STORE_UNAVAILABLE",
                "Image attachments need gui.attachmentsDir, which is not set.",
            )
        })?;
        for image in &prepared {
            store.save_image(image)?;
        }
        Ok(prepared.into_iter().map(|image| image.image).collect())
    }

    /// Read an image, verified against its reference. Blocking.
    pub fn read(&self, reference: &ImageRef) -> Result<Vec<u8>, AttachmentError> {
        match &self.store {
            Some(store) => store.read_image(reference),
            None => Err(AttachmentError::new(
                "ATTACHMENT_NOT_FOUND",
                "Attachment object is missing.",
            )),
        }
    }
}

/// An admission lock held for one prompt.
pub struct Serialized<'a> {
    admission: &'a ImageAdmission,
    session_id: String,
    guard: Option<tokio::sync::OwnedMutexGuard<()>>,
    lock: Option<Arc<tokio::sync::Mutex<()>>>,
}

impl Drop for Serialized<'_> {
    fn drop(&mut self) {
        drop(self.guard.take());
        let lock = self.lock.take();
        let mut sessions = self.admission.sessions.lock().unwrap();
        // Ours and the map's are the only handles left once nobody waits.
        if lock.is_some_and(|lock| Arc::strong_count(&lock) == 2) {
            sessions.remove(&self.session_id);
        }
    }
}

/// Serve `session/attachment`; `None` for every other endpoint.
pub(super) async fn dispatch(
    router: &super::router::Router,
    ctx: &Context,
    endpoint: &str,
    payload: &Value,
) -> Option<RpcResult> {
    if endpoint != ATTACHMENT {
        return None;
    }
    Some(match read(router, ctx, payload).await {
        Ok(value) => RpcResult::ok(value),
        Err(failure) => RpcResult::failed(failure),
    })
}

async fn read(
    router: &super::router::Router,
    ctx: &Context,
    payload: &Value,
) -> Result<Value, Failure> {
    check_descriptor(ATTACHMENT, &["request"], payload)?;
    let request = payload["request"]
        .as_object()
        .ok_or_else(|| boundary(ATTACHMENT, "request"))?;
    let session_id = request
        .get("sessionId")
        .and_then(Value::as_str)
        .ok_or_else(|| boundary(ATTACHMENT, "request"))?;
    // The page sends the whole reference it read from the log; the stored
    // reference is looked up by its id and is what the reply returns.
    let attachment_id = request
        .get("attachment")
        .and_then(|attachment| attachment.get("attachmentId"))
        .and_then(Value::as_str)
        .ok_or_else(|| boundary(ATTACHMENT, "request"))?;
    require_session(ctx, ATTACHMENT, session_id)?;
    let events = session_events(ctx, session_id)?;
    let Some(reference) = referenced_image(&events, attachment_id) else {
        return Err(attachment_failure(
            "ATTACHMENT_NOT_REFERENCED",
            "Image is not referenced by this session.",
        ));
    };
    let images = router.images();
    let stored = reference.clone();
    let bytes = tokio::task::spawn_blocking({
        let images = Arc::clone(&images);
        move || images.read(&stored)
    })
    .await
    .map_err(|error| {
        Failure::at(
            super::wire::codes::INTERNAL,
            format!("failed to read image attachment: {error}"),
            ATTACHMENT,
        )
    })?
    .map_err(|error| {
        if let Some(cause) = &error.cause {
            ctx.logger().log_event(
                cordis::logger::LogLevel::Warn,
                "gui".to_string(),
                None,
                format!("image read: {}: {cause}", error.code),
            );
        }
        attachment_failure(error.code, &error.message)
    })?;
    use base64::Engine;
    Ok(json!({
        "attachment": reference,
        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
    }))
}

/// The events of a session, live or stored. A stored log that cannot be read is
/// an internal failure, not a missing session: `require_session` has already
/// shown the session exists.
fn session_events(ctx: &Context, session_id: &str) -> Result<Vec<SessionEvent>, Failure> {
    use crate::api::services::AgentRegistryService;
    if let Some(agent) = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(session_id))
    {
        return Ok(agent.session().events());
    }
    let unreadable = |reason: String| {
        Failure::at(
            super::wire::codes::INTERNAL,
            format!("session {session_id:?} cannot be read: {reason}"),
            ATTACHMENT,
        )
    };
    persistence(ctx)
        .ok_or_else(|| unreadable("no session store".to_string()))?
        .load(session_id)
        .map_err(|error| unreadable(error.to_string()))
}

/// The first image block of the session log that carries `attachment_id`.
/// Only the log's own message-bearing events count: an id that merely appears
/// in some other text does not authorise a read.
pub fn referenced_image(events: &[SessionEvent], attachment_id: &str) -> Option<ImageRef> {
    let in_blocks = |blocks: &[ContentBlock]| {
        blocks.iter().find_map(|block| match block {
            ContentBlock::Image { attachment } | ContentBlock::ImageData { attachment, .. }
                if attachment.attachment_id == attachment_id =>
            {
                Some(attachment.clone())
            }
            _ => None,
        })
    };
    events.iter().find_map(|event| match &event.data {
        SessionEventData::UserMessage { message }
        | SessionEventData::AssistantMessage { message, .. }
        | SessionEventData::ToolResult { message, .. } => in_blocks(&message.content),
        SessionEventData::InboxSpliced { inserted, .. } => inserted
            .iter()
            .find_map(|message| in_blocks(&message.content)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, MessageSource, Role};

    fn reference(id: &str) -> ImageRef {
        ImageRef {
            attachment_id: id.to_string(),
            media_type: "image/png".to_string(),
            bytes: 1,
            width: 1,
            height: 1,
            name: None,
            original_dimensions: None,
        }
    }

    fn user_image(id: &str) -> SessionEvent {
        SessionEvent {
            seq: 0,
            time: 0,
            data: SessionEventData::UserMessage {
                message: Message {
                    id: "m".into(),
                    role: Role::User,
                    content: vec![ContentBlock::Image {
                        attachment: reference(id),
                    }],
                    source: MessageSource::User { rpc_id: None },
                },
            },
        }
    }

    #[test]
    fn only_an_image_the_log_holds_is_referenced() {
        let events = [user_image("sha256:aa")];
        assert_eq!(
            referenced_image(&events, "sha256:aa"),
            Some(reference("sha256:aa"))
        );
        assert_eq!(referenced_image(&events, "sha256:bb"), None);
        assert_eq!(referenced_image(&[], "sha256:aa"), None);
    }

    #[test]
    fn the_cap_on_a_request_body_grows_only_when_a_model_accepts_images() {
        let off = ImageAdmission::default();
        assert_eq!(off.max_body_bytes(), 0);
        let on = ImageAdmission::new(
            None,
            &ImageConfig::default(),
            ImageModels::new([("p".to_string(), "m".to_string())]),
        );
        assert!(on.max_body_bytes() as u64 > 200 * 1024 * 1024 * 4 / 3);
    }

    #[test]
    fn the_gate_names_the_model_it_refuses() {
        let failure = ImageAdmission::default()
            .require_model("p", "m")
            .unwrap_err();
        assert_eq!(failure.code, "session/attachment-invalid");
        assert_eq!(failure.message, "Model \"m\" does not support image input.");
        assert_eq!(failure.details["reason"], "MODEL_DOES_NOT_SUPPORT_IMAGES");
    }
}
