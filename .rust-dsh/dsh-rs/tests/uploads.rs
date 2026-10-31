//! File uploads through the gateway: the unary `fileUploads/upload`, the raw
//! `POST /api/session/uploadFileBinary` route, receipts, and what a prompt that
//! cites them puts in the log and in the model request.

use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, LlmService};
use dsh_rs::attachments::{AttachmentStore, ATTACHMENTS_SERVICE};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::uploads::Uploads;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::llm::adapters::mock::MockAdapter;
use dsh_rs::llm::{ContentBlock, GenerateOptions, LlmAdapter, LlmError, StreamChunk};
use dsh_rs::session::SessionEventData;
use dsh_rs::types::AgentOptions;
use serde_json::{json, Value};

const TOKEN: &str = "uploads-token-0123456789";
const LIMIT: usize = 1024;

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
}

async fn rig() -> Rig {
    rig_with(true).await
}

async fn rig_with(with_store: bool) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
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
    let store = with_store.then(|| AttachmentStore::open(store_dir.path()).unwrap());
    if let Some(store) = &store {
        ctx.provide(ATTACHMENTS_SERVICE, store.clone())
            .await
            .unwrap();
    }
    let uploads = Arc::new(Uploads::new(store, LIMIT).unwrap());
    uploads.observe(&ctx).await.unwrap();
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        max_page_events: 100,
        max_search_results: 20,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
    })
    .with_uploads(uploads);
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
    }
}

fn headers(content_type: Option<&str>) -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    if let Some(content_type) = content_type {
        headers.insert(http::header::CONTENT_TYPE, content_type.parse().unwrap());
    }
    headers
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers(None), &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

async fn raw(
    rig: &Rig,
    method: http::Method,
    query: Option<&str>,
    content_type: Option<&str>,
    body: &[u8],
) -> (u16, Value, String) {
    let (reply, _) = rig
        .gateway
        .route_with_query(
            &rig.ctx,
            &method,
            "/api/session/uploadFileBinary",
            query,
            &headers(content_type),
            body,
        )
        .await;
    let text = String::from_utf8_lossy(&reply.body).into_owned();
    (
        reply.status,
        serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    )
}

fn b64(bytes: &[u8]) -> String {
    dsh_rs::gui::base64::encode(bytes)
}

async fn upload(rig: &Rig, data: &[u8], name: Option<&str>) -> Value {
    let mut request = json!({ "data": b64(data) });
    if let Some(name) = name {
        request["name"] = json!(name);
    }
    call(
        rig,
        "fileUploads/upload",
        json!({ "agentId": rig.agent.id(), "request": request }),
    )
    .await
}

fn prompt_args(rig: &Rig, request_id: &str, content: Value) -> Value {
    json!({ "request": {
        "sessionId": rig.agent.id(), "requestId": request_id, "mode": "queue", "content": content,
    }})
}

fn reason(reply: &Value) -> &str {
    reply["error"]["details"]["reason"].as_str().unwrap_or("")
}

const HELLO_SHA: &str = "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447";

#[tokio::test]
async fn an_upload_is_stored_and_answered_with_a_receipt_and_the_file_reference() {
    let rig = rig().await;
    let reply = upload(&rig, b"hello world\n", Some("dir/notes.txt")).await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
    let value = &reply["value"];
    assert_eq!(
        value["file"]["attachmentId"],
        json!(format!("sha256:{HELLO_SHA}"))
    );
    assert_eq!(value["file"]["name"], json!("notes.txt"));
    assert_eq!(value["file"]["bytes"], json!(12));
    let receipt = value["receiptId"].as_str().unwrap();
    assert_eq!(receipt.len(), 36, "a uuid: {receipt}");
    let object = rig.store_dir.path().join("file-objects/a9").join(HELLO_SHA);
    assert_eq!(std::fs::read(object).unwrap(), b"hello world\n");
}

#[tokio::test]
async fn a_zero_byte_file_is_a_valid_upload() {
    let rig = rig().await;
    let reply = upload(&rig, b"", None).await;
    assert_eq!(reply["value"]["file"]["bytes"], json!(0), "{reply}");
    assert_eq!(reply["value"]["file"]["name"], json!("file"));
}

#[tokio::test]
async fn data_that_is_not_canonical_base64_is_refused_with_its_reason() {
    let rig = rig().await;
    for bad in ["not base64!!", "aGk", "aGk=\n"] {
        let reply = call(
            &rig,
            "fileUploads/upload",
            json!({ "agentId": rig.agent.id(), "request": { "data": bad } }),
        )
        .await;
        assert_eq!(
            reply["error"]["code"], "session/attachment-invalid",
            "{bad:?} {reply}"
        );
        assert_eq!(
            reply["error"]["message"],
            "File upload is not canonical base64."
        );
        assert_eq!(reason(&reply), "INVALID_FILE_BASE64");
    }
}

#[tokio::test]
async fn a_request_of_the_wrong_type_is_a_boundary_failure_on_request() {
    let rig = rig().await;
    let id = rig.agent.id().to_string();
    for request in [
        json!({ "data": 5 }),
        json!({ "data": "", "name": 5 }),
        json!("x"),
        json!({}),
    ] {
        let reply = call(
            &rig,
            "fileUploads/upload",
            json!({ "agentId": id, "request": request }),
        )
        .await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{request} {reply}"
        );
        assert_eq!(
            reply["error"]["details"],
            json!({ "endpoint": "fileUploads/upload", "field": "request" })
        );
    }
}

#[tokio::test]
async fn the_wire_name_of_the_session_is_agent_id() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "fileUploads/upload",
        json!({ "agent": rig.agent.id(), "request": { "data": "" } }),
    )
    .await;
    assert_eq!(
        reply["error"]["code"], "gateway/arguments-invalid",
        "{reply}"
    );
}

#[tokio::test]
async fn an_unknown_session_cannot_receive_an_upload() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "fileUploads/upload",
        json!({ "agentId": "ghost", "request": { "data": "" } }),
    )
    .await;
    assert_eq!(reply["error"]["code"], "session/not-found", "{reply}");
    assert_eq!(reply["error"]["details"], json!({ "sessionId": "ghost" }));
}

#[tokio::test]
async fn a_file_over_the_limit_is_refused_without_being_stored() {
    let rig = rig().await;
    let reply = upload(&rig, &vec![7u8; LIMIT + 1], Some("big.bin")).await;
    assert_eq!(
        reply["error"]["code"], "session/attachment-invalid",
        "{reply}"
    );
    assert_eq!(reason(&reply), "FILE_TOO_LARGE");
    assert!(
        !rig.store_dir.path().join("file-objects").exists(),
        "nothing was written"
    );
    let exact = upload(&rig, &vec![7u8; LIMIT], Some("fits.bin")).await;
    assert_eq!(
        exact["ok"],
        json!(true),
        "a file at the limit is kept: {exact}"
    );
}

#[tokio::test]
async fn without_a_store_uploads_are_unavailable_rather_than_silently_dropped() {
    let rig = rig_with(false).await;
    let reply = upload(&rig, b"x", None).await;
    assert_eq!(
        reply["error"]["code"], "gateway/service-unavailable",
        "{reply}"
    );
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("attachmentsDir"),
        "the message names the setting to change: {reply}"
    );
}

#[tokio::test]
async fn a_prompt_citing_a_receipt_logs_a_file_block_and_the_model_is_shown_its_path() {
    let rig = rig().await;
    let up = upload(&rig, b"hello world\n", Some("notes.txt")).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap().to_string();
    let reply = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "req-1",
            json!([{ "type": "text", "text": "read this" }, { "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(reply, json!({ "ok": true, "value": { "accepted": true } }));
    rig.agent.when_idle().await;

    let logged = rig
        .agent
        .session()
        .events()
        .into_iter()
        .find_map(|event| match event.data {
            SessionEventData::UserMessage { message } => Some(message),
            _ => None,
        })
        .expect("the prompt was logged");
    assert!(
        matches!(&logged.content[1], ContentBlock::File { attachment }
            if attachment.attachment_id == format!("sha256:{HELLO_SHA}") && attachment.name == "notes.txt"),
        "the log keeps the reference, not the bytes: {:?}",
        logged.content
    );

    let requests = rig.requests.lock().unwrap();
    let sent = requests[0].messages.last().unwrap();
    assert_eq!(sent.content[0], ContentBlock::text("read this"));
    let handle = sent.content[1].as_text().expect("the model sees text");
    let path = rig
        .store_dir
        .path()
        .join(format!("files/a9/{HELLO_SHA}/notes.txt"));
    assert!(
        handle.starts_with(
            "[File \"notes.txt\" (12 bytes, sha256:a948904f): verbatim read-only copy saved at"
        ),
        "{handle}"
    );
    assert!(
        handle.contains(&serde_json::to_string(&path.to_string_lossy()).unwrap()),
        "{handle}"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"hello world\n",
        "the named path holds the file"
    );
}

#[tokio::test]
async fn a_file_alone_is_a_complete_prompt() {
    let rig = rig().await;
    let up = upload(&rig, b"x", Some("a.txt")).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap();
    let reply = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "req-1",
            json!([{ "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(reply["ok"], json!(true), "{reply}");
}

#[tokio::test]
async fn an_unknown_receipt_is_refused_before_the_empty_text_check() {
    let rig = rig().await;
    for content in [
        json!([{ "type": "file", "receiptId": "nope" }]),
        json!([{ "type": "text", "text": "   " }, { "type": "file", "receiptId": "nope" }]),
        json!([{ "type": "text", "text": "hi" }, { "type": "file", "receiptId": "" }]),
    ] {
        let reply = call(
            &rig,
            "session/prompt",
            prompt_args(&rig, "req-1", content.clone()),
        )
        .await;
        assert_eq!(
            reply["error"]["code"], "session/attachment-invalid",
            "{content} {reply}"
        );
        assert_eq!(
            reply["error"]["message"],
            "File was not uploaded for this session."
        );
        assert_eq!(reason(&reply), "FILE_NOT_STAGED");
    }
}

#[tokio::test]
async fn a_receipt_is_good_in_the_session_that_minted_it_only() {
    let rig = rig().await;
    let other = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let up = upload(&rig, b"x", None).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap();
    let reply = call(
        &rig,
        "session/prompt",
        json!({ "request": {
            "sessionId": other.id(), "mode": "queue", "requestId": "r",
            "content": [{ "type": "file", "receiptId": receipt }],
        }}),
    )
    .await;
    assert_eq!(reason(&reply), "FILE_NOT_STAGED", "{reply}");
}

#[tokio::test]
async fn a_receipt_is_spent_once_its_prompt_reaches_the_log() {
    let rig = rig().await;
    let up = upload(&rig, b"x", None).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap().to_string();
    let first = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "req-1",
            json!([{ "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(first["ok"], json!(true), "{first}");
    rig.agent.when_idle().await;
    let second = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "req-2",
            json!([{ "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(
        reason(&second),
        "FILE_NOT_STAGED",
        "a spent receipt is gone: {second}"
    );
}

#[tokio::test]
async fn a_retried_prompt_is_accepted_again_without_its_receipt_or_a_second_copy() {
    let rig = rig().await;
    let up = upload(&rig, b"x", None).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap().to_string();
    let args = prompt_args(
        &rig,
        "req-1",
        json!([{ "type": "file", "receiptId": receipt }]),
    );
    assert_eq!(
        call(&rig, "session/prompt", args.clone()).await["ok"],
        json!(true)
    );
    rig.agent.when_idle().await;
    let again = call(&rig, "session/prompt", args).await;
    assert_eq!(again, json!({ "ok": true, "value": { "accepted": true } }));
    let users = rig
        .agent
        .session()
        .events()
        .into_iter()
        .filter(|event| matches!(event.data, SessionEventData::UserMessage { .. }))
        .count();
    assert_eq!(users, 1, "the retry queued nothing");
}

#[tokio::test]
async fn a_malformed_file_block_is_a_boundary_failure_on_request() {
    let rig = rig().await;
    for content in [
        json!([{ "type": "file" }]),
        json!([{ "type": "file", "receiptId": 5 }]),
        json!([{ "type": "bogus" }]),
        json!([{ "text": "no type" }]),
        json!([{ "type": "text", "text": 5 }]),
    ] {
        let reply = call(
            &rig,
            "session/prompt",
            prompt_args(&rig, "r", content.clone()),
        )
        .await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{content} {reply}"
        );
        assert_eq!(reply["error"]["details"]["field"], "request", "{content}");
    }
}

/// A model that answers only once told to, so a later prompt stays queued.
struct Gate(Arc<tokio::sync::Notify>);

impl LlmAdapter for Gate {
    fn name(&self) -> &'static str {
        "gate"
    }

    fn stream(
        &self,
        _options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        let gate = self.0.clone();
        Box::pin(async move {
            gate.notified().await;
            Ok(dsh_rs::llm::runtime::stream_from_chunks(
                MockAdapter::text_response("ok"),
            ))
        })
    }
}

#[tokio::test]
async fn removing_a_queued_prompt_retires_its_receipts() {
    let rig = rig().await;
    let gate = Arc::new(tokio::sync::Notify::new());
    let runtime = rig.ctx.require::<LlmService>("llm").unwrap();
    runtime.unregister_adapter(&["mock"]);
    runtime
        .register_adapter(&["mock"], Arc::new(Gate(gate.clone())))
        .unwrap();
    // The first prompt occupies the driver; the second waits behind it.
    call(
        &rig,
        "session/prompt",
        prompt_args(&rig, "busy", json!([{ "type": "text", "text": "first" }])),
    )
    .await;
    let up = upload(&rig, b"x", None).await;
    let receipt = up["value"]["receiptId"].as_str().unwrap().to_string();
    let queued = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "queued",
            json!([{ "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(queued["ok"], json!(true), "{queued}");
    let item_id = rig
        .agent
        .session()
        .events()
        .into_iter()
        .find_map(|event| match event.data {
            SessionEventData::InboxSpliced { inserted, .. } => inserted
                .iter()
                .find(|m| matches!(&m.source, dsh_rs::types::MessageSource::User { rpc_id: Some(r) } if r == "queued"))
                .map(|m| m.id.clone()),
            _ => None,
        })
        .expect("the queued prompt is in the inbox log");
    let removed = call(
        &rig,
        "session/updateQueue",
        json!({ "request": {
            "sessionId": rig.agent.id(), "itemId": item_id, "action": { "kind": "remove" },
        }}),
    )
    .await;
    assert_eq!(removed["ok"], json!(true), "{removed}");
    let again = call(
        &rig,
        "session/prompt",
        prompt_args(
            &rig,
            "again",
            json!([{ "type": "file", "receiptId": receipt }]),
        ),
    )
    .await;
    assert_eq!(
        reason(&again),
        "FILE_NOT_STAGED",
        "the removed prompt took its receipt with it: {again}"
    );
    gate.notify_waiters();
}

#[tokio::test]
async fn a_raw_upload_stores_the_body_byte_for_byte_and_answers_like_the_unary_route() {
    let rig = rig().await;
    let body: Vec<u8> = (0..=255u8).collect();
    let query = format!("sessionId={}&name=raw%20file.bin", rig.agent.id());
    let (status, json, text) = raw(
        &rig,
        http::Method::POST,
        Some(&query),
        Some("application/octet-stream"),
        &body,
    )
    .await;
    assert_eq!(status, 200, "{text}");
    assert_eq!(json["ok"], json!(true), "{text}");
    assert_eq!(json["value"]["file"]["name"], json!("raw file.bin"));
    assert_eq!(json["value"]["file"]["bytes"], json!(256));
    let digest = json["value"]["file"]["attachmentId"]
        .as_str()
        .unwrap()
        .strip_prefix("sha256:")
        .unwrap();
    let stored = std::fs::read(
        rig.store_dir
            .path()
            .join("file-objects")
            .join(&digest[..2])
            .join(digest),
    )
    .unwrap();
    assert_eq!(stored, body, "binary data is not altered");
    let receipt = json["value"]["receiptId"].as_str().unwrap();
    let prompt = call(
        &rig,
        "session/prompt",
        prompt_args(&rig, "r", json!([{ "type": "file", "receiptId": receipt }])),
    )
    .await;
    assert_eq!(
        prompt["ok"],
        json!(true),
        "a raw upload's receipt is redeemable: {prompt}"
    );
}

#[tokio::test]
async fn the_raw_route_refuses_what_the_typescript_route_refuses() {
    let rig = rig().await;
    let ok_type = Some("application/octet-stream");
    let query = format!("sessionId={}", rig.agent.id());
    for method in [http::Method::GET, http::Method::PUT, http::Method::DELETE] {
        let (status, _, text) = raw(&rig, method.clone(), Some(&query), ok_type, b"").await;
        assert_eq!((status, text.as_str()), (404, "not found"), "{method}");
    }
    let (status, _, text) = raw(
        &rig,
        http::Method::POST,
        Some(&query),
        Some("application/json"),
        b"{}",
    )
    .await;
    assert_eq!(
        (status, text.as_str()),
        (415, "content type must be application/octet-stream")
    );
    let (status, _, text) = raw(&rig, http::Method::POST, Some(&query), None, b"").await;
    assert_eq!(
        (status, text.as_str()),
        (415, "content type must be application/octet-stream")
    );
    for missing in [None, Some("name=x"), Some("sessionId=")] {
        let (status, _, text) = raw(&rig, http::Method::POST, missing, ok_type, b"").await;
        assert_eq!(
            (status, text.as_str()),
            (400, "sessionId is required"),
            "{missing:?}"
        );
    }
    let (status, _, _) = raw(
        &rig,
        http::Method::POST,
        Some(&query),
        Some("Application/Octet-Stream; charset=x"),
        b"",
    )
    .await;
    assert_eq!(
        status, 200,
        "the media type is compared case-insensitively, parameters ignored"
    );
}

#[tokio::test]
async fn the_raw_route_reports_session_and_size_faults_as_a_200_with_an_error_body() {
    let rig = rig().await;
    let ok_type = Some("application/octet-stream");
    let (status, json, text) = raw(
        &rig,
        http::Method::POST,
        Some("sessionId=ghost"),
        ok_type,
        b"x",
    )
    .await;
    assert_eq!(status, 200, "{text}");
    assert_eq!(json["ok"], json!(false));
    assert_eq!(json["error"]["code"], "session/not-found");
    let query = format!("sessionId={}", rig.agent.id());
    let (status, json, _) = raw(
        &rig,
        http::Method::POST,
        Some(&query),
        ok_type,
        &vec![0u8; LIMIT + 1],
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["error"]["details"]["reason"], "FILE_TOO_LARGE");
}

#[tokio::test]
async fn a_raw_upload_needs_the_launch_token() {
    let rig = rig().await;
    let mut bare = http::HeaderMap::new();
    bare.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    bare.insert(
        http::header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route_with_query(
            &rig.ctx,
            &http::Method::POST,
            "/api/session/uploadFileBinary",
            Some("sessionId=x"),
            &bare,
            b"x",
        )
        .await;
    assert_eq!(reply.status, 401);
}

#[tokio::test]
async fn the_body_limit_leaves_room_for_the_biggest_allowed_file_as_base64() {
    let rig = rig().await;
    let largest = b64(&vec![1u8; LIMIT]);
    assert!(rig.gateway.max_body_bytes() > largest.len() + 200);
    assert!(
        rig.gateway.max_body_bytes() < 1024 * 1024,
        "and not a free-for-all"
    );
}

#[tokio::test]
async fn the_endpoint_is_listed_so_the_page_can_discover_it() {
    assert!(Router::endpoints().contains(&"fileUploads/upload"));
}

#[tokio::test]
async fn a_session_without_a_working_directory_is_listed_with_no_cwd_key() {
    // The page derives a workspace title from `cwd` and throws on `null`, which
    // the TypeScript host avoids by leaving the key out.
    let rig = rig().await;
    let listed = call(&rig, "session/list", json!({ "_request": {} })).await;
    let row = listed["value"]["items"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["sessionId"] == json!(rig.agent.id()))
        })
        .unwrap_or_else(|| panic!("the session is listed: {listed}"));
    assert!(row.get("cwd").is_none(), "no cwd key, not null: {row}");
    assert_eq!(row["agentAvailable"], json!(true));
}

/// One HTTP/1.1 exchange on a fresh connection; returns the status line and body.
async fn exchange(address: std::net::SocketAddr, head: &str, body: &[u8]) -> (String, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    stream.write_all(head.as_bytes()).await.unwrap();
    // The server may answer 413 and close before reading everything; a failed
    // write after that point is the expected end of the exchange.
    let _ = stream.write_all(body).await;
    let mut reply = Vec::new();
    let _ = stream.read_to_end(&mut reply).await;
    let reply = String::from_utf8_lossy(&reply).into_owned();
    let status = reply.lines().next().unwrap_or_default().to_string();
    let body = reply
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_string();
    (status, body)
}

#[tokio::test]
async fn a_body_past_the_limit_is_refused_with_413_before_it_is_read_whole() {
    let rig = rig().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (gateway, ctx) = (Arc::clone(&rig.gateway), rig.ctx.clone());
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(gateway, ctx, listener)
            .await
            .ok();
    });
    let limit = rig.gateway.max_body_bytes();
    let head = |path: &str, length: usize| {
        format!(
            "POST {path} HTTP/1.1\r\nhost: 127.0.0.1\r\nauthorization: Bearer {TOKEN}\r\n\
             content-type: application/octet-stream\r\ncontent-length: {length}\r\nconnection: close\r\n\r\n"
        )
    };
    let big = vec![0u8; limit + 1];
    for path in [
        "/api/session/uploadFileBinary?sessionId=x",
        "/api/session/list",
    ] {
        let (status, body) = exchange(address, &head(path, big.len()), &big).await;
        assert!(status.contains("413"), "{path}: {status}");
        assert!(body.contains("gateway/bad-request"), "{path}: {body}");
        assert!(
            body.contains(&format!("{limit} byte limit")),
            "{path}: {body}"
        );
    }
    // A body at the limit is read and answered by the route itself.
    let fits = vec![0u8; LIMIT];
    let path = format!("/api/session/uploadFileBinary?sessionId={}", rig.agent.id());
    let (status, body) = exchange(address, &head(&path, fits.len()), &fits).await;
    assert!(status.contains("200"), "{status}");
    assert!(body.contains("\"ok\":true"), "{body}");
}

#[tokio::test]
async fn a_session_with_no_prompt_is_listed_with_its_creation_time_not_zero() {
    // The page prints the age of `updatedAt`; zero reads as "56y". The
    // TypeScript host orders by the later of creation and the last prompt.
    let rig = rig().await;
    let listed = call(&rig, "session/list", json!({ "_request": {} })).await;
    let row = listed["value"]["items"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["sessionId"] == json!(rig.agent.id())))
        .unwrap_or_else(|| panic!("the session is listed: {listed}"));
    let created = rig.agent.session().header_created_at();
    assert!(created > 0, "the rig session has a creation time");
    assert_eq!(row["updatedAt"], json!(created), "{row}");
}
