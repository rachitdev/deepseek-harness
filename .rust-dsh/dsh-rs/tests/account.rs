//! `account/*` on a host with no DeepSeek account.

use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, BoxFuture, LlmAdapterApi, LlmService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::{AgentOptions, BoxStream, ContentBlock, GenerateOptions, LlmError, Message, StreamChunk};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as Frame;

const TOKEN: &str = "account-token-0123456789";

struct Hang;

impl LlmAdapterApi for Hang {
    fn name(&self) -> &'static str {
        "hang"
    }
    fn stream(&self, _options: GenerateOptions) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        Box::pin(async {
            std::future::pending::<()>().await;
            unreachable!()
        })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    address: std::net::SocketAddr,
}

async fn rig() -> Rig {
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
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.to_string() }, None));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (serve_ctx, serving) = (ctx.clone(), Arc::clone(&gateway));
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(serving, serve_ctx, listener).await.ok();
    });
    Rig { ctx, gateway, address }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method, "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    let (reply, _) = rig.gateway.route(&rig.ctx, &http::Method::POST, "/api", &headers, &body).await;
    serde_json::from_slice::<Value>(&reply.body).expect("JSON reply")["result"].clone()
}

fn client() -> Value {
    json!({ "version": "1", "locale": "en", "timezoneOffsetSeconds": 0 })
}

fn signed_out() -> Value {
    json!({
        "status": "signed-out", "attempt": null,
        "links": { "usageUrl": "https://platform.deepseek.com/usage", "topUpUrl": "https://platform.deepseek.com/top_up" },
    })
}

#[tokio::test]
async fn the_state_is_signed_out_with_the_platform_links() {
    let rig = rig().await;
    assert_eq!(call(&rig, "account/getState", json!({})).await, json!({ "ok": true, "value": signed_out() }));
}

#[tokio::test]
async fn sign_out_and_cancel_report_the_signed_out_state() {
    let rig = rig().await;
    let out = call(&rig, "account/signOut", json!({ "client": client() })).await;
    assert_eq!(out, json!({ "ok": true, "value": signed_out() }));
    for attempt in ["x", ""] {
        let cancel = call(&rig, "account/cancelSignIn", json!({ "attemptId": attempt })).await;
        assert_eq!(cancel, json!({ "ok": true, "value": signed_out() }), "{attempt:?}");
    }
}

#[tokio::test]
async fn account_data_is_null_and_an_acknowledgement_finds_nothing() {
    let rig = rig().await;
    for method in ["account/getProfile", "account/getBalance", "account/getUnnotifiedBonuses"] {
        let reply = call(&rig, method, json!({ "client": client() })).await;
        assert_eq!(reply, json!({ "ok": true, "value": null }), "{method}");
    }
    let ack = call(&rig, "account/ackBonusNotified", json!({ "accountId": "a", "orderId": "o", "client": client() })).await;
    assert_eq!(ack, json!({ "ok": true, "value": false }));
}

#[tokio::test]
async fn the_client_description_is_validated_at_the_gateway() {
    let rig = rig().await;
    for bad in [
        json!({ "version": "1", "locale": "en" }),
        json!({ "version": "1", "locale": "en", "timezoneOffsetSeconds": "0" }),
        json!({ "version": 1, "locale": "en", "timezoneOffsetSeconds": 0 }),
        json!({}),
        json!(null),
        json!("client"),
    ] {
        let reply = call(&rig, "account/getBalance", json!({ "client": bad.clone() })).await;
        assert_eq!(reply["error"]["code"], "gateway/input-invalid", "{bad}: {reply}");
        assert_eq!(reply["error"]["details"]["field"], "client");
    }
    // Extra keys in the description are ignored.
    let extra = json!({ "version": "1", "locale": "en", "timezoneOffsetSeconds": 0, "extra": true });
    assert_eq!(call(&rig, "account/getBalance", json!({ "client": extra })).await["ok"], true);
}

#[tokio::test]
async fn field_names_and_types_are_checked_before_any_answer() {
    let rig = rig().await;
    let wrong_name = call(&rig, "account/getBalance", json!({ "accountId": "a", "client": client() })).await;
    assert_eq!(wrong_name["error"]["code"], "gateway/arguments-invalid");
    assert!(wrong_name["error"]["message"].as_str().unwrap().contains("unexpected \"accountId\""));
    let both = call(&rig, "account/signOut", json!({ "accountId": "a" })).await;
    assert!(both["error"]["message"].as_str().unwrap().ends_with("missing \"client\"; unexpected \"accountId\""));
    let typed = call(&rig, "account/cancelSignIn", json!({ "attemptId": 5 })).await;
    assert_eq!(typed["error"]["code"], "gateway/input-invalid");
    assert_eq!(typed["error"]["details"]["field"], "attemptId");
}

#[tokio::test]
async fn a_run_on_the_account_provider_is_a_running_account_task_and_no_other_is() {
    let rig = rig().await;
    assert_eq!(call(&rig, "account/hasRunningAccountTasks", json!({})).await, json!({ "ok": true, "value": false }));

    let llm = rig.ctx.require::<LlmService>("llm").unwrap();
    llm.register_adapter(&["deepseek-account"], Arc::new(Hang)).unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(&["mock"], Arc::new(Hang)).unwrap();
    let agents = rig.ctx.require::<AgentRegistryService>("agents").unwrap();

    let other = agents.create(Some("plain".into()), AgentOptions::mock("m"), Some("/tmp".into()), None, None).unwrap();
    other.followup(Message::user("u", vec![ContentBlock::text("go")]));
    wait_for_request(&*other).await;
    assert_eq!(
        call(&rig, "account/hasRunningAccountTasks", json!({})).await["value"], false,
        "a running turn on another provider is not an account task"
    );

    let options = AgentOptions { provider: "deepseek-account".into(), model: "m".into(), max_tokens: None };
    let account = agents.create(Some("acct".into()), options, Some("/tmp".into()), None, None).unwrap();
    account.followup(Message::user("u", vec![ContentBlock::text("go")]));
    wait_for_request(&*account).await;
    assert_eq!(call(&rig, "account/hasRunningAccountTasks", json!({})).await["value"], true);
    account.cancel(dsh_rs::types::AgentCancelCause::User, false);
    other.cancel(dsh_rs::types::AgentCancelCause::User, false);
}

async fn wait_for_request(agent: &dyn AgentView) {
    for _ in 0..300 {
        if agent.session().request_header().is_some() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the agent never made a request");
}

#[tokio::test]
async fn start_sign_in_is_not_served() {
    let rig = rig().await;
    let reply = call(&rig, "account/startSignIn", json!({
        "client": client(), "callbackOrigin": "http://127.0.0.1:3081", "loginSource": "web",
    }))
    .await;
    assert_eq!(reply["error"]["code"], "gateway/lookup-not-found", "{reply}");
}

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(rig: &Rig) -> Socket {
    let mut request = format!("ws://{}/api/remote.mux", rig.address).into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    tokio_tungstenite::connect_async(request).await.expect("upgrade").0
}

async fn next_frame(socket: &mut Socket, wait: Duration) -> Option<Value> {
    loop {
        match tokio::time::timeout(wait, socket.next()).await {
            Err(_) => return None,
            Ok(None) => panic!("the socket closed"),
            Ok(Some(message)) => {
                if let Frame::Text(text) = message.expect("no transport error") {
                    let frame: Value = serde_json::from_str(text.as_str()).unwrap();
                    if frame["type"] != "heartbeat" {
                        return Some(frame);
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn the_watch_stream_sends_the_signed_out_state_and_the_expiry_stream_stays_silent() {
    let rig = rig().await;
    let mut socket = connect(&rig).await;
    let open = |id: &str, endpoint: &str, args: Value| {
        json!({ "type": "open", "streamId": id, "endpoint": endpoint, "payload": { "args": args } }).to_string()
    };
    socket.send(Frame::text(open("w", "account/watch", json!({})))).await.unwrap();
    let frame = next_frame(&mut socket, Duration::from_secs(3)).await.expect("a baseline frame");
    assert_eq!(frame["value"], signed_out());

    socket.send(Frame::text(open("e", "account/watchExpiry", json!({})))).await.unwrap();
    assert_eq!(next_frame(&mut socket, Duration::from_millis(400)).await, None, "nothing expires");

    socket.send(Frame::text(open("bad", "account/watchExpiry", json!({ "x": 1 })))).await.unwrap();
    let refusal = next_frame(&mut socket, Duration::from_secs(3)).await.expect("a refusal");
    assert_eq!(refusal["type"], "error", "{refusal}");
    assert_eq!(refusal["error"]["code"], "gateway/input-invalid");
}
