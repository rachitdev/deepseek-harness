//! `llm/discoverModels`, through the gateway, against a throwaway local server.

use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::types::AgentOptions;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const TOKEN: &str = "discovery-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
}

async fn rig() -> Rig {
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
    });
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

/// What the throwaway server answers and what it was asked.
struct Server {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
}

/// Serve `reply` (a status line and body) to every connection, keeping each request's text.
async fn serve(status: &'static str, body: Vec<u8>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = requests.clone();
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (seen, body) = (seen.clone(), body.clone());
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let read = stream.read(&mut buffer).await.unwrap_or(0);
                seen.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buffer[..read]).to_string());
                let head = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes()).await;
                let _ = stream.write_all(&body).await;
            });
        }
    });
    Server { base, requests }
}

fn discover(request: Value) -> Value {
    json!({ "settingsNs": "llm-pi-ai", "request": request })
}

fn rejected(reply: &Value) -> (&str, &str) {
    assert_eq!(
        reply["error"]["code"], "llm/model-discovery-rejected",
        "{reply}"
    );
    (
        reply["error"]["message"].as_str().unwrap(),
        reply["error"]["details"]["settingsNs"].as_str().unwrap(),
    )
}

#[tokio::test]
async fn an_openai_endpoint_is_listed_with_a_bearer_key_deduplicated_in_order() {
    let rig = rig().await;
    let server = serve(
        "200 OK",
        br#"{"data":[{"id":"b","name":"B","context_length":8192},{"id":"a","max_tokens":512},{"id":"b","name":"again"}]}"#.to_vec(),
    )
    .await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": format!("{}/v1/", server.base), "apiKey": "  sk-test \n" })),
    )
    .await;
    assert_eq!(
        reply,
        json!({ "ok": true, "value": [
            { "id": "b", "name": "B", "contextWindow": 8192 },
            { "id": "a", "name": "a", "maxTokens": 512 },
        ]})
    );
    let requests = server.requests.lock().unwrap();
    let text = requests[0].to_lowercase();
    assert!(text.starts_with("get /v1/models http/1.1"), "{text}");
    assert!(text.contains("authorization: bearer sk-test\r\n"), "{text}");
    assert!(text.contains("user-agent: deepseek-harness/"), "{text}");
}

#[tokio::test]
async fn an_anthropic_endpoint_is_listed_with_its_own_headers_and_path() {
    let rig = rig().await;
    let server = serve(
        "200 OK",
        br#"{"data":[{"id":"claude-x","display_name":"Claude X"}]}"#.to_vec(),
    )
    .await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": format!("{}/v1", server.base), "api": "anthropic-messages", "apiKey": "k" })),
    )
    .await;
    assert_eq!(
        reply["value"],
        json!([{ "id": "claude-x", "name": "Claude X" }]),
        "{reply}"
    );
    let text = server.requests.lock().unwrap()[0].to_lowercase();
    assert!(text.starts_with("get /v1/models?limit=1000 "), "{text}");
    assert!(
        text.contains("x-api-key: k\r\n") && text.contains("anthropic-version: 2023-06-01\r\n"),
        "{text}"
    );
    assert!(!text.contains("authorization:"), "{text}");
}

#[tokio::test]
async fn an_unauthenticated_probe_sends_no_credentials() {
    let rig = rig().await;
    let server = serve("200 OK", br#"{"data":[]}"#.to_vec()).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": server.base })),
    )
    .await;
    assert_eq!(reply, json!({ "ok": true, "value": [] }));
    let text = server.requests.lock().unwrap()[0].to_lowercase();
    assert!(
        !text.contains("authorization:") && !text.contains("x-api-key:"),
        "{text}"
    );
}

#[tokio::test]
async fn an_endpoint_that_refuses_the_key_says_so_and_the_failure_carries_the_base_url() {
    let rig = rig().await;
    let server = serve("401 Unauthorized", b"{}".to_vec()).await;
    let url = format!("{}/v1", server.base);
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": url.clone() })),
    )
    .await;
    let (message, ns) = rejected(&reply);
    assert_eq!(
        message,
        format!("{url}/models answered 401; check the API key")
    );
    assert_eq!(ns, "llm-pi-ai");
    assert_eq!(reply["error"]["details"]["baseURL"], url.as_str());
    let server = serve("500 Internal Server Error", b"{}".to_vec()).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": server.base.clone() })),
    )
    .await;
    assert_eq!(
        rejected(&reply).0,
        format!("{}/models answered 500", server.base)
    );
}

#[tokio::test]
async fn a_redirect_is_refused_and_never_followed_so_a_key_stays_with_its_host() {
    let rig = rig().await;
    let elsewhere = serve("200 OK", br#"{"data":[{"id":"leak"}]}"#.to_vec()).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let target = format!("{}/stolen", elsewhere.base);
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let target = target.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let _ = stream.read(&mut buffer).await;
                let head = format!("HTTP/1.1 302 Found\r\nlocation: {target}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
                let _ = stream.write_all(head.as_bytes()).await;
            });
        }
    });
    for api in ["openai-completions", "anthropic-messages"] {
        let reply = call(
            &rig,
            "llm/discoverModels",
            discover(json!({ "baseURL": base.clone(), "api": api, "apiKey": "secret" })),
        )
        .await;
        assert!(
            rejected(&reply)
                .0
                .contains("answered 302; redirects are not followed"),
            "{reply}"
        );
    }
    assert!(
        elsewhere.requests.lock().unwrap().is_empty(),
        "the redirect target must never be asked"
    );
}

#[tokio::test]
async fn an_unreachable_or_unreadable_endpoint_is_rejected_not_listed_empty() {
    let rig = rig().await;
    let closed = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": base.clone() })),
    )
    .await;
    assert_eq!(rejected(&reply).0, format!("could not reach {base}/models"));
    let html = serve("200 OK", b"<html></html>".to_vec()).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": html.base.clone() })),
    )
    .await;
    assert_eq!(
        rejected(&reply).0,
        format!("{}/models did not answer with JSON", html.base)
    );
    let odd = serve("200 OK", b"{\"hello\":1}".to_vec()).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": odd.base })),
    )
    .await;
    assert!(
        rejected(&reply)
            .0
            .contains("enter this provider's models by hand"),
        "{reply}"
    );
}

#[tokio::test]
async fn an_oversized_reply_is_refused_rather_than_truncated() {
    let rig = rig().await;
    let server = serve("200 OK", vec![b' '; 5 * 1024 * 1024]).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": server.base.clone() })),
    )
    .await;
    assert!(
        rejected(&reply)
            .0
            .contains("answered with more than 4194304 bytes"),
        "{reply}"
    );
}

#[tokio::test]
async fn a_reply_of_exactly_the_ceiling_is_read_and_one_byte_more_is_refused() {
    let rig = rig().await;
    let ceiling = 4 * 1024 * 1024;
    let padded = |len: usize| {
        let mut body = br#"{"data":[{"id":"m"}]}"#.to_vec();
        body.resize(len, b' ');
        body
    };
    let at = serve("200 OK", padded(ceiling)).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": at.base.clone() })),
    )
    .await;
    assert_eq!(
        reply["value"],
        json!([{ "id": "m", "name": "m" }]),
        "{reply}"
    );
    let over = serve("200 OK", padded(ceiling + 1)).await;
    let reply = call(
        &rig,
        "llm/discoverModels",
        discover(json!({ "baseURL": over.base.clone() })),
    )
    .await;
    assert!(
        rejected(&reply).0.contains("more than 4194304 bytes"),
        "{reply}"
    );
}

#[tokio::test]
async fn requests_the_typescript_host_refuses_before_any_network_call_are_refused_here() {
    let rig = rig().await;
    let server = serve("200 OK", br#"{"data":[]}"#.to_vec()).await;
    let base = server.base.clone();
    for (request, needle) in [
        (json!({}), "needs a provider route or a baseURL"),
        (
            json!({ "provider": "" }),
            "needs a provider route or a baseURL",
        ),
        (
            json!({ "provider": "openai" }),
            "can only come from its endpoint",
        ),
        (
            json!({ "baseURL": base.clone(), "api": "azure-openai-responses" }),
            "has no model listing this build can read",
        ),
        (
            json!({ "baseURL": base.clone(), "apiKey": "" }),
            "API key is blank",
        ),
        (
            json!({ "baseURL": base.clone(), "apiKey": "caf\u{e9}" }),
            "no HTTP header can carry",
        ),
    ] {
        let reply = call(&rig, "llm/discoverModels", discover(request.clone())).await;
        assert!(rejected(&reply).0.contains(needle), "{request}: {reply}");
    }
    assert!(
        server.requests.lock().unwrap().is_empty(),
        "none of those may reach the network"
    );
}

#[tokio::test]
async fn only_the_pi_ai_namespace_has_a_discovery() {
    let rig = rig().await;
    for ns in ["", "nope", "llm"] {
        let reply = call(
            &rig,
            "llm/discoverModels",
            json!({ "settingsNs": ns, "request": { "provider": "x" } }),
        )
        .await;
        assert_eq!(
            rejected(&reply),
            (
                format!("no model discovery is registered for {ns:?}").as_str(),
                ns
            )
        );
    }
}

#[tokio::test]
async fn malformed_arguments_fail_at_the_gateway_like_typescript() {
    let rig = rig().await;
    for (args, code, field) in [
        (
            json!({ "settingsNs": 5, "request": {} }),
            "gateway/input-invalid",
            Some("settingsNs"),
        ),
        (
            json!({ "settingsNs": "x", "request": null }),
            "gateway/input-invalid",
            Some("request"),
        ),
        (
            json!({ "settingsNs": "x", "request": [] }),
            "gateway/input-invalid",
            Some("request"),
        ),
        (
            json!({ "settingsNs": "x", "request": { "provider": 5 } }),
            "gateway/input-invalid",
            Some("request"),
        ),
        (
            json!({ "settingsNs": "x", "request": { "apiKey": 1 } }),
            "gateway/input-invalid",
            Some("request"),
        ),
        (
            json!({ "settingsNs": "x", "request": { "baseURL": null } }),
            "gateway/input-invalid",
            Some("request"),
        ),
        (
            json!({ "settingsNs": "x" }),
            "gateway/arguments-invalid",
            None,
        ),
        (
            json!({ "settingsNs": "x", "request": {}, "extra": 1 }),
            "gateway/arguments-invalid",
            None,
        ),
    ] {
        let reply = call(&rig, "llm/discoverModels", args.clone()).await;
        assert_eq!(reply["error"]["code"], code, "{args}: {reply}");
        if let Some(field) = field {
            assert_eq!(reply["error"]["details"]["field"], field, "{args}");
        }
    }
}
