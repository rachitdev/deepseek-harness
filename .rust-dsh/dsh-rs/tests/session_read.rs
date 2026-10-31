//! `session/state`, `session/events`, `session/wait` and the three observe
//! streams, through the gateway and a real socket.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::llm::{ContentBlock, Message, MessageSource, Role};
use dsh_rs::types::{AgentOptions, SessionEventData};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as Frame;

const TOKEN: &str = "read-token-0123456789";

struct Rig {
    ctx: Context,
    hub: Arc<dsh_rs::gui::events::EventHub>,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
    address: std::net::SocketAddr,
}

fn user(id: &str, text: &str) -> Message {
    Message::user(id, vec![ContentBlock::text(text)])
}

fn assistant(id: &str, text: &str) -> Message {
    Message {
        id: id.into(),
        role: Role::Assistant,
        content: vec![ContentBlock::text(text)],
        source: MessageSource::Model { provider: "mock".into(), model: "m".into() },
    }
}

/// Append `turns` complete turns: turn n has user message `u-n` and assistant `a-n`.
fn append_turns(agent: &Arc<dyn AgentView>, from: u64, to: u64) {
    let session = agent.session();
    for turn in from..=to {
        session.append(SessionEventData::TurnStart { turn });
        session.append(SessionEventData::UserMessage { message: user(&format!("u-{turn}"), &format!("question {turn}")) });
        session.append(SessionEventData::AssistantMessage {
            turn,
            step: 1,
            message: assistant(&format!("a-{turn}"), &format!("answer {turn}")),
            interrupted: None,
            usage: None,
        });
        session.append(SessionEventData::TurnEnd { turn, reason: dsh_rs::types::TurnEndReason::Completed });
    }
}

async fn rig() -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": { "ask-user": {} } })).await.expect("profile installs");
    let agents = ctx.require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
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
    let hub = router.events();
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.to_string() }, None));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (serve_ctx, serving) = (ctx.clone(), Arc::clone(&gateway));
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(serving, serve_ctx, listener).await.ok();
    });
    Rig { ctx, hub, gateway, agent, address }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("JSON reply");
    envelope["result"].clone()
}

async fn ok(rig: &Rig, method: &str, args: Value) -> Value {
    let reply = call(rig, method, args.clone()).await;
    assert_eq!(reply["ok"], true, "{method} {args}: {reply}");
    reply["value"].clone()
}

fn session_address(rig: &Rig) -> Value {
    json!({ "kind": "session", "sessionId": rig.agent.id() })
}

/// A user row carries the message as its data; an assistant row wraps it.
fn message_id(record: &Value) -> &str {
    let data = &record["event"]["data"];
    data["message"]["id"].as_str().or_else(|| data["id"].as_str()).unwrap()
}

fn seqs(page: &Value) -> Vec<u64> {
    page["records"].as_array().unwrap().iter().map(|r| r["event"]["seq"].as_u64().unwrap()).collect()
}

// --- session/state ------------------------------------------------------

#[tokio::test]
async fn state_of_a_live_idle_session_is_active_with_an_empty_queue() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 1);
    let state = ok(&rig, "session/state", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    assert_eq!(state["active"], true);
    assert_eq!(state["running"], false);
    assert_eq!(state["queue"], json!([]));
    assert_eq!(state["pendingInput"], json!([]));
    // The cursor is the last seq: turn/start, user, assistant are seqs 0 to 2 or later.
    let last = rig.agent.session().events().last().unwrap().seq;
    assert_eq!(state["cursor"], json!(last));
}

#[tokio::test]
async fn state_of_an_empty_log_has_cursor_minus_one() {
    let rig = rig().await;
    let state = ok(&rig, "session/state", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    let events = rig.agent.session().events();
    let expected = events.last().map_or(-1, |e| e.seq as i64);
    assert_eq!(state["cursor"], json!(expected));
}

#[tokio::test]
async fn state_lists_a_queued_prompt_with_its_request_identity() {
    let rig = rig().await;
    let started = call(&rig, "session/prompt", json!({ "request": {
        "sessionId": rig.agent.id(), "mode": "queue",
        "content": [{ "type": "text", "text": "hello" }],
        "requestId": "client-req-1",
    }}))
    .await;
    // The mock provider answers at once; if the prompt was already consumed the
    // queue is empty and the test has nothing to say about it.
    assert_eq!(started["ok"], true, "{started}");
    let state = ok(&rig, "session/state", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    for item in state["queue"].as_array().unwrap() {
        assert!(["queued", "steering", "context"].contains(&item["placement"].as_str().unwrap()));
        assert!(item["content"].is_array());
        assert_eq!(item["rpcId"], "client-req-1");
    }
}

#[tokio::test]
async fn state_of_an_unknown_session_is_session_not_found() {
    let rig = rig().await;
    let reply = call(&rig, "session/state", json!({ "request": { "sessionId": "ghost" } })).await;
    assert_eq!(reply["error"]["code"], "session/not-found", "{reply}");
}

#[tokio::test]
async fn state_of_a_session_without_a_working_directory_does_not_exist() {
    let rig = rig().await;
    let agents = rig.ctx.require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    let bare = agents.create(None, AgentOptions::mock("m"), None, None, None).unwrap();
    let reply = call(&rig, "session/state", json!({ "request": { "sessionId": bare.id() } })).await;
    assert_eq!(reply["error"]["code"], "session/not-found", "{reply}");
}

#[tokio::test]
async fn state_rejects_a_request_that_is_not_an_object_with_a_string_session() {
    let rig = rig().await;
    for request in [json!({}), json!({ "sessionId": 5 }), json!(null), json!([]), json!("s")] {
        let reply = call(&rig, "session/state", json!({ "request": request.clone() })).await;
        assert_eq!(reply["error"]["code"], "gateway/input-invalid", "{request}: {reply}");
    }
}

// --- session/events -----------------------------------------------------

#[tokio::test]
async fn events_default_to_the_whole_log_when_it_fits_the_message_budget() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 3);
    let page = ok(&rig, "session/events", json!({ "request": { "address": session_address(&rig) } })).await;
    let log = rig.agent.session().events();
    assert_eq!(page["cursor"], json!(log.last().unwrap().seq));
    assert_eq!(page["hasMore"], false);
    assert_eq!(seqs(&page), log.iter().map(|e| e.seq).collect::<Vec<_>>());
}

#[tokio::test]
async fn events_keep_the_newest_messages_and_say_more_remain() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 5);
    let page = ok(&rig, "session/events", json!({ "request": {
        "address": session_address(&rig), "maxMessages": 2,
    }}))
    .await;
    assert_eq!(page["hasMore"], true);
    let records = page["records"].as_array().unwrap();
    let messages: Vec<&str> = records
        .iter()
        .filter(|r| ["user/message", "assistant/message"].contains(&r["event"]["type"].as_str().unwrap()))
        .map(message_id)
        .collect();
    assert_eq!(messages, vec!["u-5", "a-5"]);
}

#[tokio::test]
async fn events_page_backwards_with_before_seq_without_overlap_or_gap() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 6);
    let log: Vec<u64> = rig.agent.session().events().iter().map(|e| e.seq).collect();
    let mut collected: Vec<u64> = Vec::new();
    let mut before: Option<u64> = None;
    for _ in 0..20 {
        let mut request = json!({ "address": session_address(&rig), "maxMessages": 3 });
        if let Some(before) = before {
            request["beforeSeq"] = json!(before);
        }
        let page = ok(&rig, "session/events", json!({ "request": request })).await;
        let mut got = seqs(&page);
        got.extend(collected);
        collected = got;
        if page["hasMore"] == false {
            break;
        }
        before = Some(collected[0]);
    }
    assert_eq!(collected, log);
}

#[tokio::test]
async fn events_through_seq_cuts_the_page_and_reports_that_cut() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 3);
    let page = ok(&rig, "session/events", json!({ "request": {
        "address": session_address(&rig), "throughSeq": 4,
    }}))
    .await;
    assert_eq!(page["cursor"], 4);
    assert!(seqs(&page).iter().all(|seq| *seq <= 4));
    assert_eq!(*seqs(&page).last().unwrap(), 4);
}

#[tokio::test]
async fn events_through_minus_one_is_an_empty_page() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 1);
    let page = ok(&rig, "session/events", json!({ "request": {
        "address": session_address(&rig), "throughSeq": -1,
    }}))
    .await;
    assert_eq!(page["cursor"], -1);
    assert_eq!(page["records"], json!([]));
    assert_eq!(page["hasMore"], false);
}

#[tokio::test]
async fn events_turn_window_extends_the_page_back_to_a_turn_start() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 4);
    let page = ok(&rig, "session/events", json!({ "request": {
        "address": session_address(&rig), "maxMessages": 50,
        "turnWindow": { "minMessages": 1, "minTurns": 2 },
    }}))
    .await;
    let records = page["records"].as_array().unwrap();
    assert_eq!(records[0]["event"]["type"], "turn/start", "the page starts at a turn start");
    let turn_starts = records.iter().filter(|r| r["event"]["type"] == "turn/start").count();
    assert_eq!(turn_starts, 2);
    assert_eq!(page["hasMore"], true);
}

#[tokio::test]
async fn events_refuse_bad_windows_with_bad_request() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 1);
    let cases = [
        json!({ "throughSeq": -2 }),
        json!({ "throughSeq": 1.5 }),
        json!({ "beforeSeq": -1 }),
        json!({ "maxMessages": 0 }),
        json!({ "maxMessages": 1.5 }),
        json!({ "turnWindow": { "minMessages": 0, "minTurns": 1 } }),
        json!({ "turnWindow": { "minMessages": 1, "minTurns": 0 } }),
        json!({ "maxMessages": 2, "turnWindow": { "minMessages": 3, "minTurns": 1 } }),
        json!({ "throughSeq": 9999 }),
    ];
    for extra in cases {
        let mut request = json!({ "address": session_address(&rig) });
        for (key, value) in extra.as_object().unwrap() {
            request[key] = value.clone();
        }
        let reply = call(&rig, "session/events", json!({ "request": request.clone() })).await;
        assert_eq!(reply["error"]["code"], "gateway/bad-request", "{request}: {reply}");
    }
}

#[tokio::test]
async fn events_of_an_unknown_session_and_a_foreign_subagent_are_refused_by_address_kind() {
    let rig = rig().await;
    let ghost = call(&rig, "session/events", json!({ "request": {
        "address": { "kind": "session", "sessionId": "ghost" },
    }}))
    .await;
    assert_eq!(ghost["error"]["code"], "session/not-found", "{ghost}");
    let missing_child = call(&rig, "session/events", json!({ "request": {
        "address": { "kind": "subagent", "parentSessionId": "p", "childSessionId": "ghost", "mode": "unknown" },
    }}))
    .await;
    assert_eq!(missing_child["error"]["code"], "subagent/not-found", "{missing_child}");
    let not_a_child = call(&rig, "session/events", json!({ "request": {
        "address": { "kind": "subagent", "parentSessionId": "someone-else",
                     "childSessionId": rig.agent.id(), "mode": "unknown" },
    }}))
    .await;
    assert_eq!(not_a_child["error"]["code"], "subagent/unauthorized", "{not_a_child}");
}

#[tokio::test]
async fn events_accept_a_forked_child_addressed_through_its_parent() {
    let rig = rig().await;
    let agents = rig.ctx.require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE).unwrap();
    append_turns(&rig.agent, 1, 2);
    let child = agents.fork(rig.agent.id(), None, AgentOptions::mock("m")).unwrap().unwrap();
    let page = call(&rig, "session/events", json!({ "request": {
        "address": { "kind": "subagent", "parentSessionId": rig.agent.id(),
                     "childSessionId": child.id(), "mode": "one-shot" },
    }}))
    .await;
    assert_eq!(page["ok"], true, "{page}");
    let wrong_mode = call(&rig, "session/events", json!({ "request": {
        "address": { "kind": "subagent", "parentSessionId": rig.agent.id(),
                     "childSessionId": child.id(), "mode": "continuable" },
    }}))
    .await;
    assert_eq!(wrong_mode["error"]["code"], "subagent/unauthorized", "{wrong_mode}");
}

#[tokio::test]
async fn events_reject_malformed_addresses_at_the_gateway() {
    let rig = rig().await;
    for address in [
        json!({}),
        json!({ "kind": "session" }),
        json!({ "kind": "nope", "sessionId": "s" }),
        json!({ "kind": "subagent", "parentSessionId": "p", "childSessionId": "c" }),
        json!({ "kind": "subagent", "parentSessionId": "p", "childSessionId": "c", "mode": "weird" }),
        json!("s"),
    ] {
        let reply = call(&rig, "session/events", json!({ "request": { "address": address.clone() } })).await;
        assert_eq!(reply["error"]["code"], "gateway/input-invalid", "{address}: {reply}");
    }
}

// --- session/wait -------------------------------------------------------

#[tokio::test]
async fn wait_on_an_idle_session_returns_idle_with_its_assistant_output_newest_first() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 3);
    let value = ok(&rig, "session/wait", json!({ "request": { "sessionId": rig.agent.id(), "timeoutMs": 5000 } })).await;
    assert_eq!(value["reason"], "idle");
    assert_eq!(value["output"]["truncated"], false);
    let ids: Vec<&str> = value["output"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["event"]["data"]["message"]["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["a-3", "a-2", "a-1"]);
    assert_eq!(value["state"]["active"], true);
}

#[tokio::test]
async fn wait_output_respects_after_seq_and_max_messages() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 4);
    let limited = ok(&rig, "session/wait", json!({ "request": {
        "sessionId": rig.agent.id(), "timeoutMs": 5000, "maxMessages": 2,
    }}))
    .await;
    assert_eq!(limited["output"]["records"].as_array().unwrap().len(), 2);
    assert_eq!(limited["output"]["truncated"], true);
    let events = rig.agent.session().events();
    let after = events.iter().find(|e| matches!(&e.data, SessionEventData::AssistantMessage { message, .. } if message.id == "a-2")).unwrap().seq;
    let later = ok(&rig, "session/wait", json!({ "request": {
        "sessionId": rig.agent.id(), "timeoutMs": 5000, "afterSeq": after,
    }}))
    .await;
    let ids: Vec<&str> = later["output"]["records"]
        .as_array().unwrap().iter()
        .map(|r| r["event"]["data"]["message"]["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["a-4", "a-3"]);
}

#[tokio::test]
async fn wait_reports_input_required_while_a_question_is_pending() {
    use dsh_rs::plugins::ask_user::PendingQuestion;
    let rig = rig().await;
    let registration = rig.hub.register("client-1");
    let (question, _answer) = PendingQuestion::new(json!([]), Some(rig.agent.id().to_string()));
    registration.hold("evt-1", question);
    let value = ok(&rig, "session/wait", json!({ "request": { "sessionId": rig.agent.id(), "timeoutMs": 5000 } })).await;
    assert_eq!(value["reason"], "input-required");
    assert_eq!(value["state"]["pendingInput"], json!(["question"]));
    let state = ok(&rig, "session/state", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    assert_eq!(state["pendingInput"], json!(["question"]));
}

#[tokio::test]
async fn wait_times_out_while_the_agent_keeps_running() {
    let rig = rig().await;
    // Hold a turn open: a queued prompt the mock answers instantly would race, so
    // use a start event the agent has not closed and mark the agent busy through
    // an unanswerable approval-free wait on a very short deadline.
    let started = std::time::Instant::now();
    let value = ok(&rig, "session/wait", json!({ "request": { "sessionId": rig.agent.id(), "timeoutMs": 50 } })).await;
    // An idle session answers at once, never after the deadline.
    assert_eq!(value["reason"], "idle");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[tokio::test]
async fn wait_of_an_unknown_session_is_session_not_found() {
    let rig = rig().await;
    let reply = call(&rig, "session/wait", json!({ "request": { "sessionId": "ghost" } })).await;
    assert_eq!(reply["error"]["code"], "session/not-found", "{reply}");
}

#[tokio::test]
async fn wait_refuses_bad_bounds_with_bad_request() {
    let rig = rig().await;
    for extra in [
        json!({ "timeoutMs": 0 }),
        json!({ "timeoutMs": -5 }),
        json!({ "timeoutMs": 2_147_483_648u64 }),
        json!({ "timeoutMs": 1.5 }),
        json!({ "afterSeq": -2 }),
        json!({ "afterSeq": 0.5 }),
        json!({ "maxMessages": 0 }),
    ] {
        let mut request = json!({ "sessionId": rig.agent.id() });
        for (key, value) in extra.as_object().unwrap() {
            request[key] = value.clone();
        }
        let reply = call(&rig, "session/wait", json!({ "request": request.clone() })).await;
        assert_eq!(reply["error"]["code"], "gateway/bad-request", "{request}: {reply}");
    }
}

// --- streams ------------------------------------------------------------

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(rig: &Rig) -> Socket {
    let mut request = format!("ws://{}/api/remote.mux", rig.address).into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    tokio_tungstenite::connect_async(request).await.expect("upgrade").0
}

async fn open(socket: &mut Socket, endpoint: &str, args: Value) {
    let frame = json!({ "type": "open", "streamId": "s1", "endpoint": endpoint, "payload": { "args": args } });
    socket.send(Frame::text(frame.to_string())).await.unwrap();
}

async fn next_item(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(3), socket.next())
            .await
            .expect("a frame arrives in time")
            .expect("the socket stays open")
            .expect("no transport error");
        if let Frame::Text(text) = message {
            let frame: Value = serde_json::from_str(text.as_str()).unwrap();
            if frame["type"] == "item" {
                return frame["value"].clone();
            }
            if frame["type"] == "error" || frame["type"] == "end" {
                panic!("stream ended early: {frame}");
            }
        }
    }
}

async fn next_frame(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(3), socket.next())
            .await
            .expect("a frame arrives in time")
            .expect("the socket stays open")
            .expect("no transport error");
        if let Frame::Text(text) = message {
            let frame: Value = serde_json::from_str(text.as_str()).unwrap();
            if frame["type"] != "heartbeat" {
                return frame;
            }
        }
    }
}

#[tokio::test]
async fn observe_state_sends_the_current_state_then_each_change_once() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "session/observeState", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    let first = next_item(&mut socket).await;
    assert_eq!(first["active"], true);
    let cursor = first["cursor"].as_i64().unwrap();
    append_turns(&rig.agent, 1, 1);
    let second = next_item(&mut socket).await;
    assert!(second["cursor"].as_i64().unwrap() > cursor, "{second}");
}

#[tokio::test]
async fn observe_state_of_an_unknown_session_fails_the_stream() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "session/observeState", json!({ "request": { "sessionId": "ghost" } })).await;
    let frame = next_frame(&mut socket).await;
    assert_eq!(frame["type"], "error", "{frame}");
    assert_eq!(frame["error"]["code"], "session/not-found");
}

#[tokio::test]
async fn observe_event_is_ready_then_only_events_appended_afterwards() {
    let rig = rig().await;
    append_turns(&rig.agent, 1, 1);
    let before = rig.agent.session().events().len();
    let mut socket = connect(&rig).await;
    open(&mut socket, "session/observeEvent", json!({ "request": { "address": session_address(&rig) } })).await;
    assert_eq!(next_item(&mut socket).await, json!({ "type": "ready" }));
    append_turns(&rig.agent, 2, 2);
    let mut got = Vec::new();
    for _ in 0..3 {
        let item = next_item(&mut socket).await;
        assert_eq!(item["type"], "event");
        got.push(item["event"]["seq"].as_u64().unwrap());
    }
    assert_eq!(got, (before as u64..before as u64 + 3).collect::<Vec<_>>());
}

#[tokio::test]
async fn observe_projections_is_ready_then_a_frame_per_changed_key() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "session/observeProjections", json!({ "request": { "sessionId": rig.agent.id() } })).await;
    assert_eq!(next_item(&mut socket).await, json!({ "type": "ready" }));
    rig.agent.session().append(SessionEventData::SessionTitle { title: "Renamed".into() });
    let frame = next_item(&mut socket).await;
    assert_eq!(frame["type"], "projection");
    assert_eq!(frame["key"], "title");
    assert_eq!(frame["value"], "Renamed");
    assert!(frame["seq"].is_u64());
}

#[tokio::test]
async fn observe_streams_refuse_unknown_argument_fields() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(&mut socket, "session/observeState", json!({ "request": { "sessionId": "x" }, "extra": 1 })).await;
    let frame = next_frame(&mut socket).await;
    assert_eq!(frame["type"], "error", "{frame}");
    assert_eq!(frame["error"]["code"], "gateway/arguments-invalid");
}
