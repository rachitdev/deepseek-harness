//! Images in prompts through the gateway: the model gate, the admission order,
//! what the log and the model request hold, and `session/attachment`.

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, LlmService};
use dsh_rs::attachments::{AttachmentStore, ATTACHMENTS_SERVICE};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::images::ImageAdmission;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::images::{ImageConfig, ImageModels, IMAGE_MODELS_SERVICE};
use dsh_rs::llm::adapters::mock::MockAdapter;
use dsh_rs::llm::{ContentBlock, GenerateOptions, LlmAdapter, LlmError, StreamChunk};
use dsh_rs::session::SessionEventData;
use dsh_rs::types::AgentOptions;
use serde_json::{json, Value};

const TOKEN: &str = "images-token-0123456789";

struct Recorder(Arc<Mutex<Vec<GenerateOptions>>>);

impl LlmAdapter for Recorder {
    fn name(&self) -> &'static str {
        "recorder"
    }

    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        self.0.lock().unwrap().push(options);
        Box::pin(async {
            Ok(dsh_rs::llm::runtime::stream_from_chunks(
                MockAdapter::text_response("ok"),
            ))
        })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
    requests: Arc<Mutex<Vec<GenerateOptions>>>,
    store_dir: tempfile::TempDir,
    admission: Arc<ImageAdmission>,
}

/// `capable` lists the mock model as accepting images.
async fn rig(capable: bool, config: ImageConfig) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let runtime = ctx.require::<LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(&["mock"], Arc::new(Recorder(requests.clone())))
        .unwrap();
    let agent = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let store_dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::open(store_dir.path()).unwrap();
    ctx.provide(ATTACHMENTS_SERVICE, store.clone())
        .await
        .unwrap();
    let models = if capable {
        ImageModels::new([("mock".to_string(), "m".to_string())])
    } else {
        ImageModels::default()
    };
    ctx.provide(IMAGE_MODELS_SERVICE, models.clone())
        .await
        .unwrap();
    let images = Arc::new(ImageAdmission::new(Some(store), &config, models));
    let router = Router::new(RouterConfig {
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        ..RouterConfig::default()
    })
    .with_images(Arc::clone(&images));
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    Rig {
        ctx,
        gateway,
        agent,
        requests,
        store_dir,
        admission: images,
    }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

async fn prompt(rig: &Rig, content: Value) -> Value {
    let request_id = format!("req-{}", rig.requests.lock().unwrap().len() + rand_suffix());
    call(
        rig,
        "session/prompt",
        json!({ "request": {
            "sessionId": rig.agent.id(), "requestId": request_id, "mode": "queue", "content": content,
        } }),
    )
    .await
}

fn rand_suffix() -> usize {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let raster = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x * 7 % 256) as u8, (y * 5 % 256) as u8, 90])
    });
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(raster)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn b64(bytes: &[u8]) -> String {
    dsh_rs::gui::base64::encode(bytes)
}

fn image_part(bytes: &[u8]) -> Value {
    json!({ "type": "image", "mediaType": "image/png", "data": b64(bytes), "name": "shot.png" })
}

fn logged_prompts(rig: &Rig) -> Vec<dsh_rs::types::Message> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|event| match event.data {
            SessionEventData::UserMessage { message } => Some(message),
            _ => None,
        })
        .collect()
}

fn objects_stored(rig: &Rig) -> usize {
    let root = rig.store_dir.path().join("objects");
    if !root.exists() {
        return 0;
    }
    std::fs::read_dir(root)
        .unwrap()
        .flatten()
        .map(|shard| std::fs::read_dir(shard.path()).unwrap().count())
        .sum()
}

fn reason(reply: &Value) -> (&str, &str) {
    (
        reply["error"]["code"].as_str().unwrap_or("none"),
        reply["error"]["details"]["reason"]
            .as_str()
            .unwrap_or("none"),
    )
}

#[tokio::test]
async fn a_model_not_listed_as_accepting_images_refuses_them_as_the_typescript_host_does() {
    let rig = rig(false, ImageConfig::default()).await;
    let reply = prompt(
        &rig,
        json!([{ "type": "text", "text": "look" }, image_part(&png(8, 8))]),
    )
    .await;
    assert_eq!(
        reason(&reply),
        (
            "session/attachment-invalid",
            "MODEL_DOES_NOT_SUPPORT_IMAGES"
        )
    );
    assert_eq!(
        reply["error"]["message"],
        "Model \"m\" does not support image input."
    );
    assert!(
        logged_prompts(&rig).is_empty(),
        "a refused prompt is not logged"
    );
    assert_eq!(objects_stored(&rig), 0, "nothing was stored");
}

#[tokio::test]
async fn the_model_gate_runs_before_the_bytes_are_examined() {
    let rig = rig(false, ImageConfig::default()).await;
    let reply = prompt(
        &rig,
        json!([{ "type": "image", "mediaType": "image/png", "data": "not base64!!" }]),
    )
    .await;
    assert_eq!(reason(&reply).1, "MODEL_DOES_NOT_SUPPORT_IMAGES");
}

#[tokio::test]
async fn a_malformed_image_part_is_a_boundary_failure_on_request() {
    let rig = rig(true, ImageConfig::default()).await;
    let good = b64(&png(4, 4));
    for part in [
        json!({ "type": "image", "data": good }),
        json!({ "type": "image", "mediaType": "image/bmp", "data": good }),
        json!({ "type": "image", "mediaType": "image/png" }),
        json!({ "type": "image", "mediaType": "image/png", "data": 5 }),
        json!({ "type": "image", "mediaType": "image/png", "data": good, "name": 7 }),
    ] {
        let reply = prompt(&rig, json!([part.clone()])).await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{part}: {reply}"
        );
        assert_eq!(reply["error"]["details"]["field"], "request", "{part}");
    }
}

#[tokio::test]
async fn an_unknown_key_on_an_image_part_is_ignored() {
    let rig = rig(true, ImageConfig::default()).await;
    let mut part = image_part(&png(8, 8));
    part["attachment"] = json!({ "attachmentId": "sha256:forged" });
    let reply = prompt(&rig, json!([part])).await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    assert_ne!(
        attachment.attachment_id, "sha256:forged",
        "a client cannot name its own reference"
    );
}

#[tokio::test]
async fn an_accepted_image_is_logged_by_reference_and_sent_to_the_model_as_bytes() {
    let rig = rig(true, ImageConfig::default()).await;
    let source = png(40, 30);
    let reply = prompt(
        &rig,
        json!([{ "type": "text", "text": "what is this" }, image_part(&source)]),
    )
    .await;
    assert_eq!(
        reply,
        json!({ "ok": true, "value": { "accepted": true } }),
        "{reply}"
    );
    rig.agent.when_idle().await;

    let logged = logged_prompts(&rig).remove(0);
    let ContentBlock::Image { attachment } = &logged.content[1] else {
        panic!("the log holds a reference: {:?}", logged.content);
    };
    assert_eq!((attachment.width, attachment.height), (40, 30));
    assert_eq!(attachment.name.as_deref(), Some("shot.png"));
    assert_eq!(
        attachment.bytes,
        source.len() as u64,
        "a clean image is kept as it came"
    );
    let digest = attachment.attachment_id.strip_prefix("sha256:").unwrap();
    assert_eq!(
        std::fs::read(
            rig.store_dir
                .path()
                .join(format!("objects/{}/{digest}", &digest[..2]))
        )
        .unwrap(),
        source
    );

    let requests = rig.requests.lock().unwrap();
    let sent = &requests[0].messages.last().unwrap().content;
    assert_eq!(sent[0], ContentBlock::text("what is this"));
    assert!(
        matches!(&sent[1], ContentBlock::ImageData { data, .. } if *data == b64(&source)),
        "the model gets the stored bytes: {sent:?}"
    );
}

#[tokio::test]
async fn an_image_for_a_model_that_cannot_see_it_is_replaced_by_a_note_in_the_request() {
    let rig = rig(true, ImageConfig::default()).await;
    let reply = prompt(&rig, json!([image_part(&png(8, 8))])).await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
    rig.agent.when_idle().await;
    let history = logged_prompts(&rig);
    // The same history, asked of a route the deployment does not list.
    let sent = dsh_rs::attachments::request_messages(&rig.ctx, history, "other", "blind").unwrap();
    let text = sent[0].content[0].as_text().expect("a note, not bytes");
    assert_eq!(
        text,
        "[Image shot.png was attached, but the current model cannot view images.]"
    );
}

#[tokio::test]
async fn session_attachment_returns_an_image_only_the_session_logged() {
    let rig = rig(true, ImageConfig::default()).await;
    let source = png(16, 16);
    prompt(&rig, json!([image_part(&source)])).await;
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    let id = rig.agent.id().to_string();

    let reply = call(
        &rig,
        "session/attachment",
        json!({ "request": { "sessionId": id, "attachment": attachment } }),
    )
    .await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
    assert_eq!(reply["value"]["data"], json!(b64(&source)));
    assert_eq!(
        reply["value"]["attachment"]["attachmentId"],
        json!(attachment.attachment_id)
    );

    for stranger in [
        format!("sha256:{}", "0".repeat(64)),
        "zzz".to_string(),
        String::new(),
    ] {
        let reply = call(
            &rig,
            "session/attachment",
            json!({ "request": { "sessionId": id, "attachment": {
                "attachmentId": stranger, "mediaType": "image/png", "bytes": 1, "width": 1, "height": 1 } } }),
        )
        .await;
        assert_eq!(
            (
                reply["error"]["code"].as_str(),
                reply["error"]["message"].as_str(),
                reason(&reply).1
            ),
            (
                Some("session/attachment-invalid"),
                Some("Image is not referenced by this session."),
                "ATTACHMENT_NOT_REFERENCED"
            ),
            "{stranger:?}"
        );
    }
    let ghost = call(
        &rig,
        "session/attachment",
        json!({ "request": { "sessionId": "ghost", "attachment": attachment } }),
    )
    .await;
    assert_eq!(ghost["error"]["code"], "session/not-found");
    let malformed = call(
        &rig,
        "session/attachment",
        json!({ "request": { "sessionId": id } }),
    )
    .await;
    assert_eq!(malformed["error"]["code"], "gateway/input-invalid");
    assert_eq!(malformed["error"]["details"]["field"], "request");
}

#[tokio::test]
async fn a_stored_image_that_was_altered_is_not_returned() {
    let rig = rig(true, ImageConfig::default()).await;
    prompt(&rig, json!([image_part(&png(16, 16))])).await;
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    let digest = attachment
        .attachment_id
        .strip_prefix("sha256:")
        .unwrap()
        .to_string();
    let object = rig
        .store_dir
        .path()
        .join(format!("objects/{}/{digest}", &digest[..2]));
    // The store keeps its objects read-only; an operator who edits one has to
    // make it writable first.
    let mut mode = std::fs::metadata(&object).unwrap().permissions();
    mode.set_readonly(false);
    std::fs::set_permissions(&object, mode).unwrap();
    std::fs::write(&object, b"tampered").unwrap();
    let reply = call(
        &rig,
        "session/attachment",
        json!({ "request": { "sessionId": rig.agent.id(), "attachment": attachment } }),
    )
    .await;
    assert_eq!(
        reason(&reply),
        ("session/attachment-invalid", "ATTACHMENT_CORRUPT")
    );
}

#[tokio::test]
async fn admission_refusals_name_the_reason_and_leave_no_trace() {
    let config = ImageConfig {
        max_images_per_message: 2,
        max_message_image_bytes: 600,
        media_types: vec!["image/png".to_string(), "image/jpeg".to_string()],
        ..ImageConfig::default()
    };
    let rig = rig(true, config).await;
    let ok = png(8, 8);
    let cases: Vec<(&str, Value, &str)> = vec![
        (
            "non-canonical base64",
            json!([{ "type": "image", "mediaType": "image/png", "data": "AAA" }]),
            "INVALID_IMAGE_BASE64",
        ),
        (
            "empty data",
            json!([{ "type": "image", "mediaType": "image/png", "data": "" }]),
            "INVALID_IMAGE_BASE64",
        ),
        (
            "not an image",
            json!([{ "type": "image", "mediaType": "image/png", "data": b64(b"hello") }]),
            "INVALID_IMAGE",
        ),
        (
            "declared type differs",
            json!([{ "type": "image", "mediaType": "image/jpeg", "data": b64(&ok) }]),
            "IMAGE_TYPE_MISMATCH",
        ),
        (
            "type this deployment does not accept",
            json!([{ "type": "image", "mediaType": "image/webp", "data": b64(&ok) }]),
            "UNSUPPORTED_IMAGE_TYPE",
        ),
        (
            "too many images",
            json!([image_part(&ok), image_part(&ok), image_part(&ok)]),
            "TOO_MANY_IMAGES",
        ),
        (
            "too many bytes in all",
            json!([image_part(&png(40, 40)), image_part(&png(40, 40))]),
            "IMAGES_TOO_LARGE",
        ),
        (
            "the batch is checked before any image is decoded",
            json!([
                image_part(&ok),
                image_part(&ok),
                json!({ "type": "image", "mediaType": "image/png", "data": b64(b"x") })
            ]),
            "TOO_MANY_IMAGES",
        ),
        (
            "one bad image refuses the prompt and stores none",
            json!([
                image_part(&ok),
                json!({ "type": "image", "mediaType": "image/png", "data": b64(b"x") })
            ]),
            "INVALID_IMAGE",
        ),
    ];
    for (what, content, expected) in cases {
        let reply = prompt(&rig, content).await;
        assert_eq!(
            reason(&reply),
            ("session/attachment-invalid", expected),
            "{what}: {reply}"
        );
    }
    assert!(logged_prompts(&rig).is_empty());
    assert_eq!(objects_stored(&rig), 0, "a refused batch stores no member");
}

#[tokio::test]
async fn a_large_image_is_stored_smaller_and_remembers_how_big_it_was() {
    let rig = rig(true, ImageConfig::default()).await;
    let reply = prompt(&rig, json!([image_part(&png(3000, 2000))])).await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    assert!(u64::from(attachment.width) * u64::from(attachment.height) <= 2048 * 2048);
    assert_eq!(
        attachment.original_dimensions,
        Some(dsh_rs::types::ImageSize {
            width: 3000,
            height: 2000
        })
    );
    assert_eq!(attachment.media_type, "image/jpeg");
}

#[tokio::test]
async fn a_retried_prompt_is_accepted_without_storing_its_images_again() {
    let rig = rig(true, ImageConfig::default()).await;
    let args = json!({ "request": {
        "sessionId": rig.agent.id(), "requestId": "same", "mode": "queue",
        "content": [image_part(&png(8, 8))],
    } });
    let first = call(&rig, "session/prompt", args.clone()).await;
    let again = call(&rig, "session/prompt", args).await;
    assert_eq!(first, again);
    rig.agent.when_idle().await;
    assert_eq!(logged_prompts(&rig).len(), 1);
}

#[tokio::test]
async fn the_page_is_told_what_images_it_may_send() {
    let rig = rig(true, ImageConfig::default()).await;
    let reply = call(
        &rig,
        "session/projections",
        json!({ "request": { "sessionId": rig.agent.id() } }),
    )
    .await;
    assert_eq!(
        reply["value"]["values"]["imageLimits"],
        json!({
            "maxImageBytes": 20 * 1024 * 1024,
            "maxImagesPerMessage": 20,
            "maxMessageImageBytes": 200 * 1024 * 1024,
            "maxImagePixels": 64_000_000,
            "maxImageDimension": 8192,
            "mediaTypes": ["image/png", "image/jpeg", "image/webp", "image/gif"],
        }),
        "{reply}"
    );
}

#[tokio::test]
async fn the_body_limit_admits_a_full_batch_only_when_a_model_takes_images() {
    let on = rig(true, ImageConfig::default()).await;
    let off = rig(false, ImageConfig::default()).await;
    assert!(on.gateway.max_body_bytes() > 200 * 1024 * 1024 * 4 / 3);
    assert!(off.gateway.max_body_bytes() < 200 * 1024 * 1024);
}

#[tokio::test]
async fn a_request_for_a_model_that_sees_images_fails_when_the_image_cannot_be_read() {
    let rig = rig(true, ImageConfig::default()).await;
    prompt(&rig, json!([image_part(&png(16, 16))])).await;
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    let digest = attachment
        .attachment_id
        .strip_prefix("sha256:")
        .unwrap()
        .to_string();
    std::fs::remove_file(
        rig.store_dir
            .path()
            .join(format!("objects/{}/{digest}", &digest[..2])),
    )
    .unwrap();

    let error = dsh_rs::attachments::request_messages(&rig.ctx, logged_prompts(&rig), "mock", "m")
        .expect_err("a request must not go out without the picture the user attached");
    assert_eq!(error.code, "ATTACHMENT_NOT_FOUND");
}

#[tokio::test]
async fn an_object_of_another_size_than_its_reference_is_refused_before_it_is_read_whole() {
    let rig = rig(true, ImageConfig::default()).await;
    prompt(&rig, json!([image_part(&png(16, 16))])).await;
    rig.agent.when_idle().await;
    let ContentBlock::Image { attachment } = logged_prompts(&rig).remove(0).content.remove(0)
    else {
        panic!("an image was logged");
    };
    let digest = attachment
        .attachment_id
        .strip_prefix("sha256:")
        .unwrap()
        .to_string();
    let object = rig
        .store_dir
        .path()
        .join(format!("objects/{}/{digest}", &digest[..2]));
    let mut mode = std::fs::metadata(&object).unwrap().permissions();
    mode.set_readonly(false);
    std::fs::set_permissions(&object, mode).unwrap();
    std::fs::write(&object, vec![0u8; 1024 * 1024]).unwrap();
    let reply = call(
        &rig,
        "session/attachment",
        json!({ "request": { "sessionId": rig.agent.id(), "attachment": attachment } }),
    )
    .await;
    assert_eq!(
        reason(&reply),
        ("session/attachment-invalid", "ATTACHMENT_CORRUPT")
    );
}

#[tokio::test]
async fn choosing_a_model_waits_for_an_image_admission_already_running() {
    let rig = rig(true, ImageConfig::default()).await;
    let held = rig.admission.serialize(rig.agent.id()).await;
    let select = call(
        &rig,
        "session/selectModel",
        json!({ "request": { "sessionId": rig.agent.id(), "provider": "mock", "model": "m" } }),
    );
    tokio::pin!(select);
    let early = tokio::time::timeout(std::time::Duration::from_millis(150), &mut select).await;
    assert!(
        early.is_err(),
        "the selection must wait for the admission in flight"
    );
    drop(held);
    let reply = select.await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
}
