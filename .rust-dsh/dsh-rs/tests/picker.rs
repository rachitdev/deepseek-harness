//! `directoryPicker/pick`: the native chooser, and the host without one.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::BoxFuture;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::desktop::Platform;
use dsh_rs::gui::picker::{Picker, PickerError, PickerRunner};
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "test-token-0123456789abcdef";

/// A chooser that answers from a script, so no dialog opens.
struct Script {
    answers: std::sync::Mutex<std::collections::VecDeque<Result<String, PickerError>>>,
}

impl Script {
    fn new(answers: Vec<Result<String, PickerError>>) -> Arc<Self> {
        Arc::new(Script { answers: std::sync::Mutex::new(answers.into()) })
    }
}

impl PickerRunner for Script {
    fn run(&self, _program: &str, _args: Vec<String>) -> BoxFuture<Result<String, PickerError>> {
        let answer = self.answers.lock().unwrap().pop_front().unwrap_or(Ok(String::new()));
        Box::pin(async move { answer })
    }
}

async fn rig(picker: Option<Arc<Picker>>) -> (Context, Arc<Gateway>) {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.unwrap();
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel { provider: "mock".into(), model: "m".into() },
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
    });
    let router = match picker {
        Some(picker) => router.with_picker(picker),
        None => router,
    };
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    (ctx, gateway)
}

async fn call(gateway: &Gateway, ctx: &Context, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let body = serde_json::to_vec(&json!({ "type": "client-request", "rpcId": "r", "method": method, "payload": { "args": args } })).unwrap();
    let (reply, _) = gateway.route(ctx, &http::Method::POST, "/api", &headers, &body).await;
    assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

#[tokio::test]
async fn the_chosen_directory_is_answered() {
    let picker = Picker::new(Platform::Mac, Script::new(vec![Ok("/Users/someone/Projects\n".into())]));
    let (ctx, gateway) = rig(Some(Arc::new(picker))).await;
    let reply = call(&gateway, &ctx, "directoryPicker/pick", json!({})).await;
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["value"], "/Users/someone/Projects");
}

#[tokio::test]
async fn a_dismissed_chooser_answers_nothing() {
    let picker = Picker::new(Platform::Mac, Script::new(vec![Ok("\n".into())]));
    let (ctx, gateway) = rig(Some(Arc::new(picker))).await;
    let reply = call(&gateway, &ctx, "directoryPicker/pick", json!({})).await;
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["value"], Value::Null);
}

#[tokio::test]
async fn a_failed_chooser_says_it_failed() {
    let picker = Picker::new(Platform::Mac, Script::new(vec![Err(PickerError::Other("the chooser exploded".into()))]));
    let (ctx, gateway) = rig(Some(Arc::new(picker))).await;
    let reply = call(&gateway, &ctx, "directoryPicker/pick", json!({})).await;
    assert_eq!(reply["error"]["code"], "gateway/internal");
    assert_eq!(reply["error"]["message"], "directory picker failed: the chooser exploded");
}

#[tokio::test]
async fn a_host_without_the_native_capability_says_so() {
    let (ctx, gateway) = rig(None).await;
    let reply = call(&gateway, &ctx, "directoryPicker/pick", json!({})).await;
    assert_eq!(reply["error"]["code"], "directory-picker/unavailable");
    assert_eq!(
        reply["error"]["message"],
        "directoryPicker.pick needs the native capability; the composed picker serves \"browse\""
    );
    assert_eq!(reply["error"]["details"]["capability"], "browse");
}
