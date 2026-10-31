//! State streams over a real mux socket, including explicit live executor values.

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::{AgentOptions, ContentBlock, Message, MessageSource, Role, SessionEventData};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message as Frame};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "stream-state-token-0123456789";
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
struct Rig {
    ctx: Context,
    agent: Arc<dyn AgentView>,
    address: std::net::SocketAddr,
}
async fn rig() -> Rig {
    let ctx = Context::new();
    // The subagent plugin provides the lineage that says an agent is delegated.
    install_profile(
        &ctx,
        &json!({ "plugins": { "ask-user": {}, "subagent": {} } }),
    )
    .await
    .expect("profile installs");
    let agents = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .create(
            None,
            AgentOptions::mock("m"),
            Some("/tmp".into()),
            None,
            None,
        )
        .unwrap();
    let router = Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".into(),
            model: "m".into(),
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
    });
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (serve_ctx, serving) = (ctx.clone(), Arc::clone(&gateway));
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(serving, serve_ctx, listener)
            .await
            .ok();
    });
    let rig = Rig {
        ctx,
        agent,
        address,
    };
    // Standalone gateways lazily mount the observer on their first watch.
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "mount",
        "toolResults/watch",
        json!({"request":{"sessionId":"absent","seq":0}}),
    )
    .await;
    item(&mut socket, "mount", json!({"available":false})).await;
    socket.close(None).await.unwrap();
    rig
}

async fn connect(rig: &Rig) -> Socket {
    let mut request = format!("ws://{}/api/remote.mux", rig.address)
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (socket, response) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(response.status(), 101);
    socket
}
async fn send(socket: &mut Socket, value: Value) {
    socket.send(Frame::text(value.to_string())).await.unwrap();
}
async fn open(socket: &mut Socket, id: &str, endpoint: &str, args: Value) {
    send(
        socket,
        json!({"type":"open", "streamId":id, "endpoint":endpoint, "payload":{"args":args}}),
    )
    .await;
}
async fn next(socket: &mut Socket) -> Value {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(3), socket.next())
            .await
            .expect("frame arrives")
            .expect("socket remains open")
            .expect("valid transport");
        if let Frame::Text(text) = frame {
            return serde_json::from_str(text.as_str()).unwrap();
        }
    }
}
fn recorded_result(agent: &Arc<dyn AgentView>, text: &str, error: bool) -> u64 {
    let session = agent.session();
    session.append(SessionEventData::ToolResult {
        turn: 1,
        step: 1,
        message: Message {
            id: "result".into(),
            role: Role::User,
            source: MessageSource::Tool {
                tool: "example".into(),
            },
            content: vec![ContentBlock::ToolResult {
                tool_call_id: "call".into(),
                content: vec![ContentBlock::text(text)],
                is_error: Some(error),
            }],
        },
    });
    session.events().last().unwrap().seq
}
async fn result(ctx: &Context, agent: &Arc<dyn AgentView>, text: &str, error: bool) -> u64 {
    let seq = recorded_result(agent, text, error);
    if !error {
        // This is the executor's JSON value, deliberately unlike content blocks.
        ctx.emit_sync(
            "toolResults/value",
            json!({
                "session": agent.id(), "seq": seq, "value": {"answer": text}
            }),
        )
        .await
        .unwrap();
    }
    seq
}
async fn item(socket: &mut Socket, id: &str, value: Value) {
    assert_eq!(
        next(socket).await,
        json!({"type":"item", "streamId":id, "value":value})
    );
}

#[tokio::test]
async fn descriptor_refusals_precede_session_lookup_for_both_streams() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    for (endpoint, args, code, field) in [
        (
            "toolResults/watch",
            json!({}),
            "gateway/arguments-invalid",
            "request",
        ),
        (
            "toolResults/watch",
            json!({"request":{"sessionId":"absent","seq":"0"}}),
            "gateway/input-invalid",
            "request",
        ),
        (
            "toolResults/watch",
            json!({"request":{"sessionId":false,"seq":0}}),
            "gateway/input-invalid",
            "request",
        ),
        (
            "userQuestions/attachWait",
            json!({"agentId":"absent"}),
            "gateway/arguments-invalid",
            "callId",
        ),
        (
            "userQuestions/attachWait",
            json!({"agentId":"absent","callId":9}),
            "gateway/input-invalid",
            "callId",
        ),
        (
            "userQuestions/attachWait",
            json!({"agentId":false,"callId":"c"}),
            "gateway/input-invalid",
            "agentId",
        ),
    ] {
        open(&mut socket, "bad", endpoint, args).await;
        let frame = next(&mut socket).await;
        assert_eq!(frame["type"], "error", "{frame}");
        assert_eq!(frame["error"]["code"], code, "{frame}");
        if code == "gateway/input-invalid" {
            assert_eq!(frame["error"]["details"]["field"], field);
        } else {
            assert!(frame["error"]["message"]
                .as_str()
                .unwrap()
                .contains(&format!("missing \"{field}\"")));
        }
    }
}

#[tokio::test]
async fn durable_result_is_unavailable_and_cancellation_is_silent() {
    let rig = rig().await;
    let seq = recorded_result(&rig.agent, "not JSON: {\"answer\":42}", false);
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "result",
        "toolResults/watch",
        json!({"request":{"sessionId":rig.agent.id(),"seq":seq}}),
    )
    .await;
    item(&mut socket, "result", json!({"available":false})).await;
    send(&mut socket, json!({"type":"cancel","streamId":"result"})).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    agents.dispose(&rig.agent);
    assert!(
        tokio::time::timeout(Duration::from_millis(1200), socket.next())
            .await
            .is_err()
    );
    // A new logical stream proves the socket is still usable after cancellation.
    open(
        &mut socket,
        "probe",
        "toolResults/watch",
        json!({"request":{"sessionId":"absent","seq":0}}),
    )
    .await;
    item(&mut socket, "probe", json!({"available":false})).await;
}

#[tokio::test]
async fn missing_result_stays_open_then_appears_and_disappears_on_disposal() {
    let rig = rig().await;
    let seq = rig.agent.session().events().last().map_or(0, |e| e.seq + 1);
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "pending",
        "toolResults/watch",
        json!({"request":{"sessionId":rig.agent.id(),"seq":seq}}),
    )
    .await;
    item(&mut socket, "pending", json!({"available":false})).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), socket.next())
            .await
            .is_err()
    );
    assert_eq!(result(&rig.ctx, &rig.agent, "arrived", false).await, seq);
    item(
        &mut socket,
        "pending",
        json!({"available":true,"value":{"answer":"arrived"}}),
    )
    .await;
    rig.ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .dispose(&rig.agent);
    item(&mut socket, "pending", json!({"available":false})).await;
}

#[tokio::test]
async fn errors_non_result_rows_and_fractional_sequences_are_unavailable() {
    let rig = rig().await;
    let seq = result(&rig.ctx, &rig.agent, "failed", true).await;
    let mut socket = connect(&rig).await;
    for number in [seq as f64, seq as f64 + 0.5, -1.0] {
        open(
            &mut socket,
            "watch",
            "toolResults/watch",
            json!({"request":{"sessionId":rig.agent.id(),"seq":number}}),
        )
        .await;
        item(&mut socket, "watch", json!({"available":false})).await;
        send(&mut socket, json!({"type":"cancel","streamId":"watch"})).await;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn oldest_result_is_evicted_after_one_thousand_newer_results() {
    let rig = rig().await;
    let seq = result(&rig.ctx, &rig.agent, "oldest", false).await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "old",
        "toolResults/watch",
        json!({"request":{"sessionId":rig.agent.id(),"seq":seq}}),
    )
    .await;
    item(
        &mut socket,
        "old",
        json!({"available":true,"value":{"answer":"oldest"}}),
    )
    .await;
    for _ in 0..1000 {
        result(&rig.ctx, &rig.agent, "newer", false).await;
    }
    item(&mut socket, "old", json!({"available":false})).await;
}

#[tokio::test]
async fn attach_wait_without_a_timer_ends_without_any_items_and_cancel_is_harmless() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "wait",
        "userQuestions/attachWait",
        json!({"agentId":rig.agent.id(),"callId":"blocking-call"}),
    )
    .await;
    assert_eq!(
        next(&mut socket).await,
        json!({"type":"end","streamId":"wait"})
    );
    send(&mut socket, json!({"type":"cancel","streamId":"wait"})).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), socket.next())
            .await
            .is_err()
    );
    open(
        &mut socket,
        "again",
        "userQuestions/attachWait",
        json!({"agentId":rig.agent.id(),"callId":"other-call"}),
    )
    .await;
    assert_eq!(
        next(&mut socket).await,
        json!({"type":"end","streamId":"again"})
    );
}

#[tokio::test]
async fn attach_wait_refuses_nonlive_and_delegated_agents_with_ts_messages() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "missing",
        "userQuestions/attachWait",
        json!({"agentId":"missing","callId":"c"}),
    )
    .await;
    let frame = next(&mut socket).await;
    // The agent is a lookup parameter: an identity the host does not hold is
    // `session/not-found`, as the TypeScript host answers it.
    assert_eq!(
        frame["error"],
        json!({"code":"session/not-found","message":"session \"missing\" not found","details":{"sessionId":"missing"}})
    );
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let child = agents
        .create(
            None,
            AgentOptions::mock("m"),
            Some("/tmp".into()),
            None,
            None,
        )
        .unwrap();
    rig.ctx
        .require::<dsh_rs::plugins::subagent::Lineage>(dsh_rs::plugins::subagent::LINEAGE_SERVICE)
        .unwrap()
        .adopt_for_test(child.id(), rig.agent.id());
    open(
        &mut socket,
        "child",
        "userQuestions/attachWait",
        json!({"agentId":child.id(),"callId":"c"}),
    )
    .await;
    let frame = next(&mut socket).await;
    assert_eq!(
        frame["error"],
        json!({"code":"gateway/internal","message":"human interaction is unavailable while the calling agent is owned by another live agent; include the unresolved question or decision in the child agent's final result","details":{}})
    );
}

#[tokio::test]
async fn failed_results_do_not_evict_a_successful_result() {
    let rig = rig().await;
    let seq = result(&rig.ctx, &rig.agent, "retained", false).await;
    for _ in 0..1000 {
        result(&rig.ctx, &rig.agent, "error", true).await;
    }
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "retained",
        "toolResults/watch",
        json!({"request":{"sessionId":rig.agent.id(),"seq":seq}}),
    )
    .await;
    item(
        &mut socket,
        "retained",
        json!({"available":true,"value":{"answer":"retained"}}),
    )
    .await;
}

#[tokio::test]
async fn a_cold_watch_does_not_follow_a_session_created_later() {
    let rig = rig().await;
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "cold",
        "toolResults/watch",
        json!({"request":{"sessionId":"new-session","seq":0}}),
    )
    .await;
    item(&mut socket, "cold", json!({"available":false})).await;
    let agent = agents
        .create(
            Some("new-session".into()),
            AgentOptions::mock("m"),
            Some("/tmp".into()),
            None,
            None,
        )
        .unwrap();
    assert_eq!(result(&rig.ctx, &agent, "new", false).await, 0);
    assert!(
        tokio::time::timeout(Duration::from_millis(1200), socket.next())
            .await
            .is_err()
    );
    open(
        &mut socket,
        "fresh",
        "toolResults/watch",
        json!({"request":{"sessionId":"new-session","seq":0}}),
    )
    .await;
    item(
        &mut socket,
        "fresh",
        json!({"available":true,"value":{"answer":"new"}}),
    )
    .await;
}

#[tokio::test]
async fn durable_lineage_does_not_make_a_runtime_root_delegated() {
    let rig = rig().await;
    let agents = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap();
    let agent = agents
        .resume(
            "resumed-root".into(),
            Some(dsh_rs::types::SessionHeader {
                cwd: Some("/tmp".into()),
                parent_session: Some(rig.agent.id().into()),
                ..Default::default()
            }),
            vec![],
            AgentOptions::mock("m"),
        )
        .unwrap();
    let mut socket = connect(&rig).await;
    open(
        &mut socket,
        "root",
        "userQuestions/attachWait",
        json!({"agentId":agent.id(),"callId":"c"}),
    )
    .await;
    assert_eq!(
        next(&mut socket).await,
        json!({"type":"end","streamId":"root"})
    );
}
