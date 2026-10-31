//! Message ratings and session remarks, through the gateway.

use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{
    AgentRegistryService, AgentView, SessionPersistenceApi, SessionService,
};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::llm::{ContentBlock, Message, MessageSource, Role};
use dsh_rs::session::store::SESSIONS_SERVICE;
use dsh_rs::types::{AgentOptions, SessionEvent, SessionEventData};
use serde_json::{json, Value};

const TOKEN: &str = "feedback-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
}

async fn rig_with_note_limit(max_note_bytes: usize) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .expect("profile installs");
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    let session = agent.session();
    session.append(SessionEventData::TurnStart { turn: 1 });
    session.append(SessionEventData::UserMessage {
        message: Message::user("u-1", vec![ContentBlock::text("hello")]),
    });
    session.append(SessionEventData::AssistantMessage {
        turn: 1,
        step: 1,
        message: Message {
            id: "a-1".into(),
            role: Role::Assistant,
            content: vec![ContentBlock::text("hi")],
            source: MessageSource::Model {
                provider: "mock".into(),
                model: "m".into(),
            },
        },
        usage: None,
        interrupted: None,
    });
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
    .with_max_note_bytes(max_note_bytes)
    .expect("a positive note limit");
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
    }
}

async fn rig() -> Rig {
    rig_with_note_limit(8_192).await
}

async fn call(rig: &Rig, method: &str, payload: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": payload },
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

fn put(rig: &Rig, extra: Value) -> Value {
    let mut request = json!({
        "sessionId": rig.agent.id(), "messageId": "a-1", "rating": "positive", "ifVersion": null,
    });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    json!({ "request": request })
}

/// The business reply inside a successful gateway reply.
fn business(reply: &Value) -> &Value {
    assert_eq!(reply["ok"], true, "{reply}");
    &reply["value"]
}

fn feedback_events(rig: &Rig) -> Vec<SessionEvent> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter(|event| event.data.event_type().starts_with("feedback/"))
        .collect()
}

#[tokio::test]
async fn a_rating_is_created_listed_updated_and_deleted() {
    let rig = rig().await;
    let sid = rig.agent.id().to_string();
    let list = || json!({ "request": { "sessionId": sid } });
    assert_eq!(
        business(&call(&rig, "messageFeedback/list", list()).await)["value"]["items"],
        json!([])
    );

    let created = call(&rig, "messageFeedback/put", put(&rig, json!({}))).await;
    let item = business(&created)["value"].clone();
    assert_eq!(item["rating"], "positive");
    assert_eq!(item["createdAt"], item["updatedAt"]);
    let version = item["version"].as_str().unwrap().to_string();
    assert_eq!(
        version.len(),
        36,
        "a UUID-shaped token, as the TypeScript reader requires"
    );

    let updated = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "ifVersion": version, "rating": "negative", "note": "  too long ", "category": "task-result" })),
    )
    .await;
    let after = business(&updated)["value"].clone();
    assert_eq!(after["rating"], "negative");
    assert_eq!(after["note"], "  too long ", "a note is kept verbatim");
    assert_eq!(after["createdAt"], item["createdAt"]);
    assert_ne!(after["version"], item["version"]);

    let listed = call(&rig, "messageFeedback/list", list()).await;
    assert_eq!(business(&listed)["value"]["items"], json!([after.clone()]));

    let stale = call(
        &rig,
        "messageFeedback/delete",
        json!({ "request": { "sessionId": sid, "messageId": "a-1", "ifVersion": version } }),
    )
    .await;
    assert_eq!(business(&stale)["error"]["code"], "version-conflict");
    assert_eq!(business(&stale)["error"]["current"], after);

    let delete = json!({ "request": { "sessionId": sid, "messageId": "a-1", "ifVersion": after["version"] } });
    for _ in 0..2 {
        let gone = call(&rig, "messageFeedback/delete", delete.clone()).await;
        assert_eq!(
            business(&gone)["value"],
            json!({ "absent": true }),
            "deleting twice succeeds"
        );
    }
    assert_eq!(
        business(&call(&rig, "messageFeedback/list", list()).await)["value"]["items"],
        json!([])
    );
    let kinds: Vec<_> = feedback_events(&rig)
        .iter()
        .map(|e| e.data.event_type())
        .collect();
    assert_eq!(
        kinds,
        [
            "feedback/message-put",
            "feedback/message-put",
            "feedback/message-delete"
        ],
        "a repeated delete adds no event"
    );
}

#[tokio::test]
async fn a_change_must_name_the_version_it_saw() {
    let rig = rig().await;
    let first = call(&rig, "messageFeedback/put", put(&rig, json!({}))).await;
    let current = business(&first)["value"].clone();

    let again = call(&rig, "messageFeedback/put", put(&rig, json!({}))).await;
    assert_eq!(
        business(&again)["error"],
        json!({ "code": "version-conflict", "current": current })
    );
    let wrong = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "ifVersion": "bogus" })),
    )
    .await;
    assert_eq!(business(&wrong)["error"]["code"], "version-conflict");

    let same = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "ifVersion": current["version"] })),
    )
    .await;
    assert_eq!(
        business(&same)["value"],
        current,
        "an identical rating keeps its version"
    );
    assert_eq!(feedback_events(&rig).len(), 1, "and adds no event");
}

#[tokio::test]
async fn only_an_assistant_message_can_be_rated() {
    let rig = rig().await;
    for target in ["u-1", "nope", ""] {
        let reply = call(
            &rig,
            "messageFeedback/put",
            put(&rig, json!({ "messageId": target })),
        )
        .await;
        assert_eq!(
            business(&reply)["error"],
            json!({ "code": "target-not-found", "sessionId": rig.agent.id(), "messageId": target }),
        );
    }
    assert!(feedback_events(&rig).is_empty());
}

#[tokio::test]
async fn notes_are_checked_before_the_session_is_looked_up() {
    let rig = rig_with_note_limit(10).await;
    let blank = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "sessionId": "ghost", "note": "   " })),
    )
    .await;
    assert_eq!(business(&blank)["error"], json!({ "code": "note-blank" }));
    let empty = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "note": "" })),
    )
    .await;
    assert_eq!(business(&empty)["error"]["code"], "note-blank");
    let big = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "sessionId": "ghost", "note": "ééééééé" })),
    )
    .await;
    assert_eq!(
        business(&big)["error"],
        json!({ "code": "note-too-large", "maxBytes": 10, "actualBytes": 14 }),
        "the limit counts UTF-8 bytes, not characters"
    );
    let exact = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "note": "ééééé" })),
    )
    .await;
    assert_eq!(
        business(&exact)["ok"],
        true,
        "a note of exactly the limit is accepted"
    );
}

#[tokio::test]
async fn an_unknown_session_is_a_business_answer_for_every_call() {
    let rig = rig().await;
    let missing = json!({ "code": "session-not-found", "sessionId": "ghost" });
    let calls = [
        (
            "messageFeedback/list",
            json!({ "request": { "sessionId": "ghost" } }),
        ),
        (
            "messageFeedback/put",
            put(&rig, json!({ "sessionId": "ghost" })),
        ),
        (
            "messageFeedback/delete",
            json!({ "request": { "sessionId": "ghost", "messageId": "a-1", "ifVersion": "v" } }),
        ),
        (
            "sessionFeedback/record",
            json!({ "request": { "sessionId": "ghost" } }),
        ),
    ];
    for (method, payload) in calls {
        let reply = call(&rig, method, payload).await;
        assert_eq!(business(&reply)["error"], missing, "{method}");
    }
}

#[tokio::test]
async fn an_empty_session_id_fails_as_the_typescript_host_does() {
    let rig = rig().await;
    for (method, payload) in [
        (
            "messageFeedback/list",
            json!({ "request": { "sessionId": "" } }),
        ),
        ("messageFeedback/put", put(&rig, json!({ "sessionId": "" }))),
    ] {
        let reply = call(&rig, method, payload).await;
        assert_eq!(reply["error"]["code"], "gateway/internal", "{method}");
    }
    let record = call(
        &rig,
        "sessionFeedback/record",
        json!({ "request": { "sessionId": "" } }),
    )
    .await;
    assert_eq!(
        business(&record)["error"]["code"],
        "session-not-found",
        "a remark only looks the session up"
    );
}

#[tokio::test]
async fn a_malformed_request_is_refused_before_anything_is_logged() {
    let rig = rig().await;
    let malformed = [
        ("messageFeedback/put", put(&rig, json!({ "rating": "bad" }))),
        ("messageFeedback/put", put(&rig, json!({ "note": 5 }))),
        ("messageFeedback/put", put(&rig, json!({ "note": null }))),
        (
            "messageFeedback/put",
            put(&rig, json!({ "category": null })),
        ),
        (
            "messageFeedback/put",
            put(&rig, json!({ "category": "bogus" })),
        ),
        ("messageFeedback/put", put(&rig, json!({ "ifVersion": 5 }))),
        (
            "messageFeedback/put",
            json!({ "request": { "sessionId": rig.agent.id(), "messageId": "a-1", "rating": "positive" } }),
        ),
        ("messageFeedback/put", json!({ "request": [] })),
        (
            "messageFeedback/list",
            json!({ "request": { "sessionId": 5 } }),
        ),
        ("messageFeedback/list", json!({ "request": {} })),
        (
            "messageFeedback/delete",
            json!({ "request": { "sessionId": rig.agent.id(), "messageId": "a-1", "ifVersion": null } }),
        ),
        (
            "sessionFeedback/record",
            json!({ "request": { "sessionId": rig.agent.id(), "text": 5 } }),
        ),
        (
            "sessionFeedback/record",
            json!({ "request": { "sessionId": rig.agent.id(), "category": "bad" } }),
        ),
    ];
    for (method, payload) in malformed {
        let reply = call(&rig, method, payload.clone()).await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{method} {payload}"
        );
    }
    let no_request = call(&rig, "messageFeedback/list", json!({})).await;
    assert_eq!(no_request["error"]["code"], "gateway/arguments-invalid");
    assert!(feedback_events(&rig).is_empty());
}

#[tokio::test]
async fn extra_request_fields_are_ignored_like_the_typescript_host() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "messageFeedback/put",
        put(&rig, json!({ "extra": 1 })),
    )
    .await;
    assert_eq!(business(&reply)["ok"], true);
}

#[tokio::test]
async fn a_remark_is_trimmed_and_blank_ones_are_recorded_as_absent() {
    let rig = rig().await;
    let sid = rig.agent.id().to_string();
    for request in [
        json!({ "sessionId": sid, "text": "  hello  ", "category": "other" }),
        json!({ "sessionId": sid, "text": "   " }),
        json!({ "sessionId": sid }),
    ] {
        let reply = call(
            &rig,
            "sessionFeedback/record",
            json!({ "request": request }),
        )
        .await;
        assert_eq!(business(&reply)["value"], json!({ "recorded": true }));
    }
    let remarks: Vec<_> = feedback_events(&rig).into_iter().map(|e| e.data).collect();
    assert_eq!(
        remarks,
        [
            SessionEventData::FeedbackRecord {
                text: Some("hello".into()),
                category: Some("other".into())
            },
            SessionEventData::FeedbackRecord {
                text: None,
                category: None
            },
            SessionEventData::FeedbackRecord {
                text: None,
                category: None
            },
        ]
    );
}

#[tokio::test]
async fn a_rating_that_cannot_be_made_durable_is_reported_not_swallowed() {
    struct Full;
    impl SessionPersistenceApi for Full {
        fn on_event(&self, _: &str, _: &SessionEvent) {}
        fn flush(&self, _: &str) -> std::io::Result<()> {
            Err(std::io::Error::other("the disk is full"))
        }
        fn load(&self, _: &str) -> std::io::Result<Vec<SessionEvent>> {
            Ok(Vec::new())
        }
        fn list(&self) -> Vec<String> {
            Vec::new()
        }
    }
    let rig = rig().await;
    rig.ctx
        .require::<SessionService>(SESSIONS_SERVICE)
        .unwrap()
        .attach_persistence(Arc::new(Full));
    let reply = call(&rig, "messageFeedback/put", put(&rig, json!({}))).await;
    assert_eq!(reply["error"]["code"], "gateway/internal");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("the disk is full"),
        "{reply}"
    );
}

#[tokio::test]
async fn a_fork_keeps_its_parents_ratings_as_the_parents() {
    let rig = rig().await;
    let created = call(&rig, "messageFeedback/put", put(&rig, json!({}))).await;
    let item = business(&created)["value"].clone();
    // A fork copies the parent's log; the copied rating is not the child's.
    let child = rig.agent.session();
    let copied: Vec<_> = child.events();
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let other = agents
        .create(None, AgentOptions::mock("m"), None, None, None)
        .unwrap();
    for event in copied
        .into_iter()
        .filter(|e| e.data.event_type().starts_with("feedback/"))
    {
        other.session().append(event.data);
    }
    let reply = call(
        &rig,
        "messageFeedback/list",
        json!({ "request": { "sessionId": other.id() } }),
    )
    .await;
    assert_eq!(business(&reply)["value"]["items"], json!([]), "{item}");
}

#[tokio::test]
async fn a_stored_session_that_is_not_loaded_is_refused_not_reported_missing() {
    struct Stored;
    impl SessionPersistenceApi for Stored {
        fn on_event(&self, _: &str, _: &SessionEvent) {}
        fn flush(&self, _: &str) -> std::io::Result<()> {
            Ok(())
        }
        fn load(&self, _: &str) -> std::io::Result<Vec<SessionEvent>> {
            Ok(Vec::new())
        }
        fn list(&self) -> Vec<String> {
            vec!["session-on-disk".to_string()]
        }
    }
    let rig = rig().await;
    rig.ctx
        .provide(
            dsh_rs::api::services::SESSION_PERSISTENCE_SERVICE,
            Arc::new(Stored) as Arc<dyn SessionPersistenceApi>,
        )
        .await
        .unwrap();
    let reply = call(
        &rig,
        "messageFeedback/list",
        json!({ "request": { "sessionId": "session-on-disk" } }),
    )
    .await;
    assert_eq!(
        reply["error"]["code"], "gateway/service-unavailable",
        "{reply}"
    );
    assert!(reply["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not loaded"));
}

#[tokio::test]
async fn a_remark_is_flushed_before_it_is_acknowledged() {
    use std::sync::Mutex;
    struct Counting(Arc<Mutex<usize>>, bool);
    impl SessionPersistenceApi for Counting {
        fn on_event(&self, _: &str, _: &SessionEvent) {}
        fn flush(&self, _: &str) -> std::io::Result<()> {
            *self.0.lock().unwrap() += 1;
            if self.1 {
                Err(std::io::Error::other("the disk is full"))
            } else {
                Ok(())
            }
        }
        fn load(&self, _: &str) -> std::io::Result<Vec<SessionEvent>> {
            Ok(Vec::new())
        }
        fn list(&self) -> Vec<String> {
            Vec::new()
        }
    }
    let rig = rig().await;
    let flushes = Arc::new(Mutex::new(0));
    let store = rig.ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap();
    store.attach_persistence(Arc::new(Counting(flushes.clone(), false)));
    let sid = rig.agent.id();
    let ok = call(
        &rig,
        "sessionFeedback/record",
        json!({ "request": { "sessionId": sid, "text": "x" } }),
    )
    .await;
    assert_eq!(business(&ok)["value"], json!({ "recorded": true }));
    assert_eq!(
        *flushes.lock().unwrap(),
        1,
        "the remark reached the backend before the reply"
    );

    store.attach_persistence(Arc::new(Counting(flushes.clone(), true)));
    let failed = call(
        &rig,
        "sessionFeedback/record",
        json!({ "request": { "sessionId": sid, "text": "y" } }),
    )
    .await;
    assert_eq!(failed["error"]["code"], "gateway/internal");
    assert!(failed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("the disk is full"));
}

#[tokio::test]
async fn a_remark_on_a_stored_session_that_is_not_loaded_is_not_found_as_in_typescript() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "sessionFeedback/record",
        json!({ "request": { "sessionId": "session-on-disk" } }),
    )
    .await;
    assert_eq!(business(&reply)["error"]["code"], "session-not-found");
}

#[test]
fn a_zero_note_limit_is_rejected_when_the_router_is_built() {
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        max_page_events: 1,
        max_search_results: 1,
        max_title_bytes: 1,
        max_bytes: 1,
        max_file_bytes: 1,
        max_lines: 1,
        max_entries: 1,
        default_max_goal_rounds: 1,
        export_compression_level: 6,
        models: Vec::new(),
    });
    assert!(router.with_max_note_bytes(0).is_err());
}
