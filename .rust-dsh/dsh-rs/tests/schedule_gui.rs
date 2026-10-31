//! `schedule/*` and the archive admission, through the gateway.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::schedule::service::{CreateRequest, ScheduleService};
use dsh_rs::schedule::store::{Retention, Store};
use serde_json::{json, Value};

const TOKEN: &str = "test-token-0123456789abcdef";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    service: Arc<ScheduleService>,
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} })).await.unwrap();
    let clock = Arc::new(AtomicI64::new(1_793_000_000_000));
    let service = ScheduleService::with_clock(Store::in_memory(), Retention::default(), Box::new(move || clock.load(Ordering::SeqCst)));
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
    })
    .with_schedule(Arc::clone(&service));
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    Rig { ctx, gateway, service }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let body = serde_json::to_vec(&json!({ "type": "client-request", "rpcId": "r", "method": method, "payload": { "args": args } })).unwrap();
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

async fn session(rig: &Rig) -> String {
    call(rig, "session/create", json!({ "request": {} })).await["value"]["sessionId"].as_str().unwrap().to_string()
}

fn after(seconds: i64) -> CreateRequest {
    CreateRequest { prompt: "p".into(), title: "T".into(), after_seconds: Some(json!(seconds)), ..Default::default() }
}

#[tokio::test]
async fn list_catalog_and_misses_answer_like_typescript() {
    let rig = rig().await;
    let one = session(&rig).await;
    let made = rig.service.create(&one, &after(600)).await.unwrap();
    let id = made["id"].as_str().unwrap();

    assert_eq!(call(&rig, "schedule/list", json!({ "request": { "sessionId": one } })).await["value"], json!([made]));
    assert_eq!(call(&rig, "schedule/list", json!({ "request": { "sessionId": "ghost" } })).await["value"], json!([]));
    let catalog = call(&rig, "schedule/catalog", json!({})).await["value"].clone();
    assert_eq!(catalog[0]["sessionId"], one.as_str());
    assert_eq!(catalog[0]["status"], "active");

    let miss = call(&rig, "schedule/delete", json!({ "request": { "sessionId": "ghost", "id": id } })).await;
    assert_eq!(miss["value"], json!({ "id": id, "deleted": false, "code": "schedule_not_found" }));
    let history = call(&rig, "schedule/history", json!({ "request": { "sessionId": one, "id": id, "limit": 5 } })).await["value"].clone();
    assert_eq!(history["records"], json!([]));
    assert_eq!(history["retention"], json!({ "days": 30, "records": 200 }));
    for limit in [0, 101, -1] {
        let bad = call(&rig, "schedule/history", json!({ "request": { "sessionId": one, "id": id, "limit": limit } })).await;
        assert_eq!(bad["error"]["message"], "Delivery history limit must be a safe integer from 1 through 100.", "{limit}");
    }
    let gone = call(&rig, "schedule/delete", json!({ "request": { "sessionId": one, "id": id } })).await;
    assert_eq!(gone["value"], json!({ "id": id, "deleted": true }));
}

#[tokio::test]
async fn update_takes_the_record_the_caller_saw() {
    let rig = rig().await;
    let one = session(&rig).await;
    let made = rig.service.create(&one, &after(600)).await.unwrap();
    let id = made["id"].as_str().unwrap();
    let request = |expected: &Value, extra: Value| {
        let mut request = json!({ "sessionId": one, "id": id, "expected": expected });
        for (k, v) in extra.as_object().unwrap() {
            request[k] = v.clone();
        }
        json!({ "request": request })
    };

    let renamed = call(&rig, "schedule/update", request(&made, json!({ "title": "Walk" }))).await["value"].clone();
    assert_eq!(renamed["updated"], true);
    assert_eq!(renamed["record"]["title"], "Walk");
    // The old record is now stale.
    let stale = call(&rig, "schedule/update", request(&made, json!({ "title": "Run" }))).await["value"].clone();
    assert_eq!(stale, json!({ "id": id, "updated": false, "code": "schedule_conflict" }));
    // Unknown properties inside `expected` are dropped before the comparison, as in TypeScript.
    let mut padded = renamed["record"].clone();
    padded["extra"] = json!(1);
    let still = call(&rig, "schedule/update", request(&padded, json!({}))).await["value"].clone();
    assert_eq!(still["updated"], false);
    assert_eq!(still["record"]["title"], "Walk");
}

#[tokio::test]
async fn archiving_a_session_with_armed_reminders_is_refused_until_activity_is_stopped() {
    let rig = rig().await;
    let one = session(&rig).await;
    let two = session(&rig).await;
    let made = rig.service.create(&one, &after(600)).await.unwrap();
    rig.service.create(&two, &after(600)).await.unwrap();

    let refused = call(&rig, "workspace/archiveSession", json!({ "request": { "sessionId": one } })).await;
    assert_eq!(refused["error"]["code"], "workspace/session-active");
    assert_eq!(refused["error"]["message"], format!("cannot archive session '{one}': the session is active (schedule)"));
    assert_eq!(
        refused["error"]["details"]["activity"],
        json!([{ "kind": "schedule", "items": [{ "id": made["id"], "label": "T" }] }])
    );
    assert_eq!(rig.service.list(&one).await.len(), 1, "a refusal changes nothing");

    let archived = call(&rig, "workspace/archiveSession", json!({ "request": { "sessionId": one, "stopActivity": true } })).await;
    assert_eq!(archived["value"]["archivedSessionIds"], json!([one]));
    assert_eq!(rig.service.list(&one).await, Vec::<Value>::new(), "the stopping archive deleted them");
    assert_eq!(rig.service.list(&two).await.len(), 1, "another session keeps its reminders");

    // A session with no reminders archives plainly.
    let plain = call(&rig, "workspace/archiveSession", json!({ "request": { "sessionId": two, "stopActivity": true } })).await;
    assert!(plain["error"].is_null(), "{plain}");
}
