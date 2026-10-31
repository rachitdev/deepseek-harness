//! Speech's no-provider contract through the HTTP gateway and WebSocket mux.

use std::sync::Arc;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
use cordis::Context;
use dsh_rs::gui::{Auth, Gateway, Router, RouterConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "speech-test-token";
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
}

fn rig() -> Rig {
    Rig {
        ctx: Context::new(),
        gateway: Arc::new(
            Gateway::new(
                Router::new(RouterConfig::default()),
                Auth {
                    token: TOKEN.into(),
                },
                None,
            )
            .with_heartbeat(Duration::from_secs(60)),
        ),
    }
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let body = serde_json::to_vec(&json!({ "type": "client-request", "rpcId": "r", "method": method, "payload": { "args": args } })).unwrap();
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    assert_eq!(reply.status, 200);
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

fn catalog(provider: &str, language: &str) -> Value {
    json!({ "providers": [], "selection": { "providerId": provider, "language": language },
        "maxAudioBytes": 4194304, "maxDurationSeconds": 120 })
}

fn wave(samples: usize) -> String {
    let mut bytes = vec![0u8; 44 + 2 * samples];
    bytes[..4].copy_from_slice(b"RIFF");
    bytes[4..8].copy_from_slice(&((36 + 2 * samples) as u32).to_le_bytes());
    bytes[8..16].copy_from_slice(b"WAVEfmt ");
    bytes[16..20].copy_from_slice(&16u32.to_le_bytes());
    bytes[20..24].copy_from_slice(&[1, 0, 1, 0]);
    bytes[24..28].copy_from_slice(&16000u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&32000u32.to_le_bytes());
    bytes[32..36].copy_from_slice(&[2, 0, 16, 0]);
    bytes[36..40].copy_from_slice(b"data");
    bytes[40..44].copy_from_slice(&((2 * samples) as u32).to_le_bytes());
    STANDARD.encode(bytes)
}

#[tokio::test]
async fn catalog_and_partial_configuration_share_the_settings_namespace() {
    let rig = rig();
    assert_eq!(
        call(&rig, "speech/catalog", json!({})).await,
        json!({ "ok": true, "value": catalog("", "auto") })
    );
    assert_eq!(
        call(
            &rig,
            "speech/configure",
            json!({ "patch": { "providerId": "offline", "language": "en" } })
        )
        .await,
        json!({ "ok": true })
    );
    assert_eq!(
        call(
            &rig,
            "speech/configure",
            json!({ "patch": { "language": "fr" } })
        )
        .await,
        json!({ "ok": true })
    );
    assert_eq!(
        call(&rig, "speech/catalog", json!({})).await["value"],
        catalog("offline", "fr")
    );
    let described = call(&rig, "settings/describe", json!({})).await;
    let ns = described["value"]["namespaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|ns| ns["ns"] == "speech")
        .unwrap();
    assert_eq!(
        ns["user"],
        json!({ "providerId": "offline", "language": "fr" })
    );
    assert_eq!(
        call(&rig, "speech/configure", json!({ "patch": {} })).await,
        json!({ "ok": true })
    );
    assert_eq!(
        call(
            &rig,
            "settings/update",
            json!({ "ns": "speech", "patch": { "language": "de" } })
        )
        .await["ok"],
        true
    );
    assert_eq!(
        call(&rig, "speech/catalog", json!({})).await["value"],
        catalog("offline", "de")
    );
}

#[tokio::test]
async fn unavailable_providers_keep_typescript_codes_messages_and_details() {
    let rig = rig();
    for endpoint in ["speech/prepare", "speech/cancelPreparation"] {
        assert_eq!(
            call(&rig, endpoint, json!({ "providerId": "ghost" })).await,
            json!({ "ok": false, "error": { "code": "gateway/internal", "message": "Speech provider is unavailable: ghost", "details": {} } })
        );
    }
    for request in [
        json!({ "audioBase64": wave(1) }),
        json!({ "audioBase64": wave(1), "providerId": "ghost" }),
    ] {
        let id = request["providerId"].as_str().unwrap_or("");
        let reason = format!("Speech provider is unavailable: {id}");
        assert_eq!(
            call(&rig, "speech/transcribe", json!({ "request": request })).await,
            json!({ "ok": false, "error": { "code": "speech/transcription-failed", "message": reason, "details": { "reason": reason } } })
        );
    }
    call(
        &rig,
        "speech/configure",
        json!({ "patch": { "providerId": "saved" } }),
    )
    .await;
    assert_eq!(
        call(
            &rig,
            "speech/transcribe",
            json!({ "request": { "audioBase64": wave(1) } })
        )
        .await["error"]["message"],
        "Speech provider is unavailable: saved"
    );
}

#[tokio::test]
async fn audio_validation_precedes_provider_resolution() {
    let rig = rig();
    for (audio, code, reason) in [
        (
            "Zg".to_string(),
            "speech/transcription-failed",
            "Audio must use canonical base64 encoding",
        ),
        (
            "".to_string(),
            "speech/transcription-failed",
            "Audio must be a canonical 16 kHz mono PCM16 WAV recording",
        ),
        (
            wave(16000 * 120 + 1),
            "speech/transcription-failed",
            "Audio exceeds 120 seconds",
        ),
        (
            "A".repeat(4194304usize.div_ceil(3) * 4 + 1),
            "speech/invalid-audio",
            "Audio is invalid or exceeds the configured byte limit",
        ),
    ] {
        let reply = call(
            &rig,
            "speech/transcribe",
            json!({ "request": { "audioBase64": audio, "providerId": "ghost" } }),
        )
        .await;
        assert_eq!(reply["error"]["code"], code);
        assert_eq!(reply["error"]["message"], reason);
    }
}

#[tokio::test]
async fn gateway_rejects_missing_unexpected_and_wrongly_typed_arguments() {
    let rig = rig();
    let missing = call(&rig, "speech/prepare", json!({ "extra": true })).await;
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    assert!(missing["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing \"providerId\"; unexpected \"extra\""));
    for (endpoint, args, field) in [
        ("speech/prepare", json!({ "providerId": 1 }), "providerId"),
        (
            "speech/configure",
            json!({ "patch": { "language": false } }),
            "patch",
        ),
        (
            "speech/transcribe",
            json!({ "request": { "audioBase64": 3 } }),
            "request",
        ),
        ("speech/transcribe", json!({ "request": {} }), "request"),
    ] {
        let result = call(&rig, endpoint, args).await;
        assert_eq!(result["error"]["code"], "gateway/input-invalid");
        assert_eq!(result["error"]["details"]["field"], field);
    }
}

async fn connect(rig: &Rig) -> Socket {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let gateway = rig.gateway.clone();
    let ctx = rig.ctx.clone();
    tokio::spawn(async move {
        dsh_rs::gui::server::serve(gateway, ctx, listener)
            .await
            .unwrap();
    });
    let mut request = format!("ws://{address}/api/remote.mux")
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    tokio_tungstenite::connect_async(request).await.unwrap().0
}

async fn send(socket: &mut Socket, frame: Value) {
    socket.send(Message::text(frame.to_string())).await.unwrap();
}

async fn next(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(2), socket.next())
            .await
            .expect("frame arrives")
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            return serde_json::from_str(text.as_str()).unwrap();
        }
    }
}

async fn open(socket: &mut Socket) {
    send(socket, json!({ "type": "open", "streamId": "s", "endpoint": "speech/follow", "payload": { "args": {} } })).await;
}

#[tokio::test]
async fn follow_sends_initial_and_changed_catalogs_and_releases_cancelled_streams() {
    let rig = rig();
    let mut socket = connect(&rig).await;
    open(&mut socket).await;
    assert_eq!(
        next(&mut socket).await,
        json!({ "type": "item", "streamId": "s", "value": catalog("", "auto") })
    );
    assert_eq!(
        call(
            &rig,
            "speech/configure",
            json!({ "patch": { "language": "en" } })
        )
        .await,
        json!({ "ok": true })
    );
    assert_eq!(next(&mut socket).await["value"], catalog("", "en"));
    assert_eq!(
        call(
            &rig,
            "settings/update",
            json!({ "ns": "speech", "patch": { "providerId": "external" } })
        )
        .await["ok"],
        true
    );
    assert_eq!(next(&mut socket).await["value"], catalog("external", "en"));
    send(&mut socket, json!({ "type": "cancel", "streamId": "s" })).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    call(
        &rig,
        "speech/configure",
        json!({ "patch": { "language": "fr" } }),
    )
    .await;
    assert!(
        tokio::time::timeout(Duration::from_millis(200), socket.next())
            .await
            .is_err()
    );
    open(&mut socket).await;
    assert_eq!(next(&mut socket).await["value"], catalog("external", "fr"));
    socket.close(None).await.unwrap();
}
