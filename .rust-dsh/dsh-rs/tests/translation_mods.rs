//! Translation and band contracts, compiled independently of the shared wiring.

use cordis::Context;
use dsh_rs::bundle::install_profile;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

// The endpoint modules are exercised before the parallel wiring change lands.
// Only the private gateway adapters are supplied here; endpoint code is included verbatim.
#[path = "../src/gui"]
mod gui {
    pub use dsh_rs::gui::{settings, wire};
    #[path = "descriptor.rs"]
    pub mod descriptor;
    pub fn translation(endpoint: &str, args: &serde_json::Value) -> super::Value {
        serde_json::to_value(cot_translation::dispatch(&settings::SettingsStore::default(), endpoint, args).unwrap()).unwrap()
    }
    pub fn press(ctx: &cordis::Context, args: &serde_json::Value) -> super::Value {
        serde_json::to_value(claude_code_mods::dispatch(ctx, "claudeCodeMods/pressBand", args).unwrap()).unwrap()
    }
    pub async fn watch(ctx: &cordis::Context, args: &serde_json::Value, out: &tokio::sync::mpsc::Sender<String>, cancel: &tokio_util::sync::CancellationToken) -> mux::Outcome {
        claude_code_mods::watch_band(ctx, "claudeCodeMods/watchBand", args, "band", out, cancel).await
    }

    pub mod router {
        use cordis::Context;
        use super::wire::Failure;
        pub fn boundary(endpoint: &str, field: &str) -> Failure {
            Failure { code: "gateway/input-invalid".into(), message: format!("typert gateway: {endpoint}: wire field {field:?} failed boundary validation"), details: serde_json::json!({ "endpoint": endpoint, "field": field }) }
        }
        pub fn require_session(ctx: &Context, _: &str, session: &str) -> Result<(), Failure> {
            let agents = ctx.get::<dsh_rs::api::services::AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
            if agents.get(session).is_some() { Ok(()) } else {
                Err(Failure { code: "session/not-found".into(), message: format!("session {session:?} not found"), details: serde_json::json!({ "sessionId": session }) })
            }
        }
    }

    pub mod mux {
        use serde_json::{json, Value};
        use tokio::sync::mpsc;
        use tokio_util::sync::CancellationToken;
        pub enum Outcome { Cancelled, Failed(super::wire::Failure) }
        pub async fn deliver(out: &mpsc::Sender<String>, id: &str, value: Value, cancel: &CancellationToken) -> Result<(), ()> {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => Err(()),
                sent = out.send(json!({ "type": "item", "streamId": id, "value": value }).to_string()) => sent.map_err(|_| ()),
            }
        }
    }

    #[path = "cot_translation.rs"]
    pub mod cot_translation;
    #[path = "claude_code_mods.rs"]
    pub mod claude_code_mods;
}

fn translation(endpoint: &str, args: Value) -> Value {
    gui::translation(endpoint, &args)
}

#[test]
fn limits_report_defaults_and_no_eligible_providers() {
    assert_eq!(translation("cotTranslation/limits", json!({})), json!({
        "ok": true, "value": {
            "maxTextChars": 4000,
            "preferences": { "provider": "bing", "targetLanguage": "auto" },
            "availableProviders": [],
        }
    }));
}

#[test]
fn translation_failures_are_sanitized_including_over_limit_utf16() {
    for text in ["private source".to_string(), "x".repeat(4001), "\u{1f600}".repeat(2001)] {
        for provider in ["google", "bing", "deepseek-account", "deepseek-official"] {
            assert_eq!(translation("cotTranslation/translate", json!({ "request": {
                "text": text, "targetLanguage": "fr", "provider": provider,
            }})), json!({ "ok": false, "error": {
                "code": "cotTranslation/failed", "message": "Translation failed", "details": {},
            }}));
        }
    }
}

#[test]
fn nested_required_fields_are_checked_at_the_boundary() {
    for request in [json!({ "text": "x" }), json!({ "targetLanguage": "fr" }), json!({ "text": 1, "targetLanguage": "fr" })] {
        assert_eq!(translation("cotTranslation/translate", json!({ "request": request })), json!({
            "ok": false, "error": {
                "code": "gateway/input-invalid",
                "message": "typert gateway: cotTranslation/translate: wire field \"request\" failed boundary validation",
                "details": { "endpoint": "cotTranslation/translate", "field": "request" },
            }
        }));
    }
    let bad = translation("cotTranslation/translate", json!({ "wrong": {} }));
    assert_eq!(bad["error"]["code"], "gateway/arguments-invalid");
    assert!(bad["error"]["message"].as_str().unwrap().contains("missing \"request\"; unexpected \"wrong\""));
}

async fn session() -> (Context, String) {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.unwrap();
    ctx.provide(gui::claude_code_mods::BANDS_SERVICE, gui::claude_code_mods::Bands::default()).await.unwrap();
    let router = dsh_rs::gui::Router::new(dsh_rs::gui::RouterConfig::default());
    let result = router.dispatch(&ctx, &gui::wire::ClientRequest {
        rpc_id: "r".into(), method: "session/create".into(), payload: json!({ "args": { "request": {} } }),
    }).await;
    let value = serde_json::to_value(result).unwrap();
    (ctx, value["value"]["sessionId"].as_str().unwrap().into())
}

fn press(ctx: &Context, session: &str, generation: i64) -> Value {
    gui::press(ctx, &json!({
        "agentId": session, "generation": generation, "actionId": "missing",
    }))
}

#[tokio::test]
async fn ignored_presses_return_the_current_drawing_not_an_error() {
    let (ctx, id) = session().await;
    for generation in [0, 1, -1, 999] {
        assert_eq!(press(&ctx, &id, generation), json!({ "ok": true, "value": { "generation": 0, "tree": null } }));
    }
    let (out, mut rx) = tokio::sync::mpsc::channel(2);
    let cancel = CancellationToken::new();
    let child = cancel.clone();
    let args = json!({ "agentId": id });
    let cloned = ctx.clone();
    let task = tokio::spawn(async move {
        gui::watch(&cloned, &args, &out, &child).await
    });
    let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
    assert_eq!(frame, json!({ "type": "item", "streamId": "band", "value": { "generation": 1, "tree": null } }));
    assert_eq!(press(&ctx, &id, 0), json!({ "ok": true, "value": { "generation": 1, "tree": null } }));
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), rx.recv()).await.is_err());
    cancel.cancel();
    match tokio::time::timeout(std::time::Duration::from_secs(2), task).await.unwrap().unwrap() {
        gui::mux::Outcome::Cancelled => {},
        gui::mux::Outcome::Failed(error) => panic!("{}", error.message),
    }
    assert_eq!(rx.recv().await, None);
}
