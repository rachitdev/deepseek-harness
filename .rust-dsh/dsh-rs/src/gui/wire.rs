//! The GUI's unary wire format, byte-compatible with the TypeScript client.
//!
//! Every unary call is one JSON envelope in and one out, over `POST /api`:
//!
//! ```text
//! -> { "type": "client-request", "rpcId": "…", "method": "session/list", "payload": { … } }
//! <- { "type": "server-response", "rpcId": "…", "result": { "ok": true, "value": … } }
//! <- { "type": "server-response", "rpcId": "…", "result": { "ok": false,
//!      "error": { "code": "gateway/lookup-not-found", "message": "…", "details": { … } } } }
//! ```
//!
//! `rpcId` is echoed unchanged so a client can correlate concurrent calls; the
//! browser client rejects a response whose id it did not mint. Both directions
//! are validated here, at the transport boundary, and nowhere deeper.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Longest accepted correlation id; anything longer is a malformed envelope.
const MAX_RPC_ID: usize = 256;

/// Failure codes the gateway itself produces. Field-level endpoint failures use
/// each endpoint's own vocabulary and pass through unchanged.
pub mod codes {
    /// The envelope could not be decoded, or a field was missing or of the wrong type.
    pub const PROTOCOL: &str = "gateway/protocol";
    /// The request carried no valid launch token. Transport-level: the upstream
    /// vocabulary has no auth code because its gateway runs in-process.
    pub const UNAUTHORIZED: &str = "gateway/unauthorized";
    /// The request came from a page on another origin. Transport-level, like
    /// [`UNAUTHORIZED`].
    pub const FORBIDDEN: &str = "gateway/forbidden";
    /// The envelope was well formed but contradicted its URL: the `method` names
    /// a different endpoint than the path the client posted to.
    pub const BAD_REQUEST: &str = "gateway/bad-request";
    /// The payload did not match the endpoint's parameters.
    pub const INPUT_INVALID: &str = "gateway/input-invalid";
    /// The call's argument fields do not match the endpoint's descriptor.
    pub const ARGUMENTS_INVALID: &str = "gateway/arguments-invalid";
    /// No endpoint is registered under that name.
    pub const LOOKUP_NOT_FOUND: &str = "gateway/lookup-not-found";
    /// The endpoint exists but the service it needs is not wired in this build.
    pub const SERVICE_UNAVAILABLE: &str = "gateway/service-unavailable";
    /// The endpoint's own code raised an unexpected failure.
    pub const INTERNAL: &str = "gateway/internal";
    /// A client sent more uplink frames than the stream's window allows.
    pub const UPLINK_OVERFLOW: &str = "gateway/uplink-overflow";
}

/// One decoded `client-request`.
#[derive(Debug, Clone, Deserialize)]
pub struct ClientRequest {
    /// Correlation id, echoed on the response.
    #[serde(rename = "rpcId")]
    pub rpc_id: String,
    /// Canonical `<namespace>/<method>` endpoint name.
    pub method: String,
    /// Endpoint parameters; `null` when the endpoint takes none.
    #[serde(default)]
    pub payload: Value,
}

/// A failure in the TypeScript client's shape.
#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    /// Stable code such as `gateway/lookup-not-found`.
    pub code: String,
    /// Human-readable, already safe to display.
    pub message: String,
    /// Structured context; always an object, often carrying `endpoint`.
    pub details: Value,
}

impl Failure {
    /// Build a failure, attaching `endpoint` to the details.
    pub fn at(code: &str, message: impl Into<String>, endpoint: &str) -> Self {
        Failure {
            code: code.to_string(),
            message: message.into(),
            details: json!({ "endpoint": endpoint }),
        }
    }
}

/// The `result` field of a response envelope.
///
/// Serialized by hand so the `ok` discriminator cannot disagree with the
/// variant. A struct field would let a caller build `{ok: false, value: …}`,
/// leaving the client unable to tell success from failure.
#[derive(Debug, Clone)]
pub enum RpcResult {
    /// The endpoint returned a value.
    Ok(Value),
    /// The endpoint returned a value with binary parts.
    ///
    /// The bytes are carried in the reply's multipart parts, and the value's
    /// own field for them is null: the TypeScript gateway answers a binary
    /// result that way, and the client reads the parts.
    WithAttachments { value: Value, attachments: Vec<Attachment> },
    /// The endpoint returned no value. The reply is `{"ok": true}` with no
    /// `value` key, which is what the TypeScript gateway sends for `undefined`.
    Empty,
    /// The endpoint failed.
    Failed(Failure),
}

impl RpcResult {
    /// A successful result.
    pub fn ok(value: Value) -> Self {
        RpcResult::Ok(value)
    }

    /// A failed result.
    pub fn failed(failure: Failure) -> Self {
        RpcResult::Failed(failure)
    }

    /// A result whose value carries binary parts.
    pub fn with_attachments(value: Value, attachments: Vec<Attachment>) -> Self {
        RpcResult::WithAttachments { value, attachments }
    }

    /// The binary parts of the result, when it has any.
    pub fn attachments(&self) -> &[Attachment] {
        match self {
            RpcResult::WithAttachments { attachments, .. } => attachments,
            _ => &[],
        }
    }
}

/// One binary part of a multipart answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// The path in the value the bytes belong to, e.g. `["data"]`.
    pub path: Vec<String>,
    /// The codec that reads the part.
    pub codec: String,
    /// The multipart part's name, e.g. `bytes-0`.
    pub part: String,
    /// The bytes themselves.
    pub bytes: Vec<u8>,
}

impl Attachment {
    /// The reference the envelope carries for the part.
    pub fn reference(&self) -> Value {
        json!({ "path": self.path, "codec": self.codec, "part": self.part })
    }
}

impl Serialize for RpcResult {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut out = serializer.serialize_struct("RpcResult", 2)?;
        match self {
            RpcResult::Ok(value) => {
                out.serialize_field("ok", &true)?;
                out.serialize_field("value", value)?;
            }
            RpcResult::WithAttachments { value, .. } => {
                out.serialize_field("ok", &true)?;
                out.serialize_field("value", value)?;
            }
            RpcResult::Empty => {
                out.serialize_field("ok", &true)?;
            }
            RpcResult::Failed(failure) => {
                out.serialize_field("ok", &false)?;
                out.serialize_field("error", failure)?;
            }
        }
        out.end()
    }
}

/// One `server-response`, ready to serialize.
#[derive(Debug, Clone, Serialize)]
pub struct ServerResponse {
    /// Always `server-response`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The request's correlation id, unchanged.
    #[serde(rename = "rpcId")]
    pub rpc_id: String,
    /// The outcome.
    pub result: RpcResult,
    /// The binary parts of the answer, when it has any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Value>>,
}

impl ServerResponse {
    /// Wrap a result for the request it answers.
    pub fn new(rpc_id: impl Into<String>, result: RpcResult) -> Self {
        ServerResponse {
            kind: "server-response",
            rpc_id: rpc_id.into(),
            result,
            attachments: None,
        }
    }

    /// A response whose result carries binary parts.
    pub fn with_attachments(rpc_id: impl Into<String>, result: RpcResult) -> Self {
        let attachments: Vec<Value> = result.attachments().iter().map(Attachment::reference).collect();
        ServerResponse {
            kind: "server-response",
            rpc_id: rpc_id.into(),
            result,
            attachments: (!attachments.is_empty()).then_some(attachments),
        }
    }

    /// Serialize to the bytes written on the wire.
    ///
    /// Encoding these owned parts cannot fail in practice; if it ever does, the
    /// replacement keeps this response's `rpcId` so the client still resolves
    /// the call it is waiting on instead of hanging.
    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|error| {
            let fallback = ServerResponse::new(
                self.rpc_id.clone(),
                RpcResult::failed(Failure::at(
                    codes::INTERNAL,
                    format!("response encoding failed: {error}"),
                    "",
                )),
            );
            serde_json::to_vec(&fallback).unwrap_or_default()
        })
    }
}

/// Why a request body could not be turned into a [`ClientRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformed {
    /// Field-level reason, safe to return to the client.
    pub reason: String,
    /// The correlation id when the envelope carried a usable one, so the client
    /// can still surface the failure against its pending call.
    pub rpc_id: Option<String>,
}

/// Decode one request body.
///
/// The `type` tag must be `client-request`: a `server-response` posted back to
/// the server is a protocol error, not an unknown method.
pub fn decode_request(body: &[u8]) -> Result<ClientRequest, Malformed> {
    let value: Value = serde_json::from_slice(body).map_err(|error| Malformed {
        reason: format!("body is not JSON: {error}"),
        rpc_id: None,
    })?;
    decode_value(&value)
}

/// Decode an already-parsed request value.
pub fn decode_value(value: &Value) -> Result<ClientRequest, Malformed> {
    let rpc_id = value
        .get("rpcId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty() && id.len() <= MAX_RPC_ID)
        .map(str::to_string);

    let malformed = |reason: &str| Malformed {
        reason: reason.to_string(),
        rpc_id: rpc_id.clone(),
    };

    let Some(object) = value.as_object() else {
        return Err(malformed("the envelope must be a JSON object"));
    };
    match object.get("type").and_then(Value::as_str) {
        Some("client-request") => {}
        Some(other) => {
            return Err(Malformed {
                reason: format!("expected type \"client-request\", got {other:?}"),
                rpc_id,
            })
        }
        None => return Err(malformed("the envelope has no \"type\"")),
    }
    if rpc_id.is_none() {
        return Err(Malformed {
            reason: format!("rpcId must be a non-empty string of at most {MAX_RPC_ID} characters"),
            rpc_id: None,
        });
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Err(malformed("\"method\" must be a string"));
    };
    if method.is_empty() {
        return Err(malformed("\"method\" must not be empty"));
    }
    Ok(ClientRequest {
        rpc_id: rpc_id.unwrap_or_default(),
        method: method.to_string(),
        payload: object.get("payload").cloned().unwrap_or(Value::Null),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(text: &str) -> Result<ClientRequest, Malformed> {
        decode_request(text.as_bytes())
    }

    #[test]
    fn a_well_formed_request_decodes() {
        let request = decode(
            r#"{"type":"client-request","rpcId":"r1","method":"session/list","payload":{"cursor":"c"}}"#,
        )
        .expect("valid");
        assert_eq!(request.rpc_id, "r1");
        assert_eq!(request.method, "session/list");
        assert_eq!(request.payload, json!({ "cursor": "c" }));
    }

    #[test]
    fn a_missing_payload_is_null_not_an_error() {
        let request = decode(r#"{"type":"client-request","rpcId":"r","method":"session/list"}"#)
            .expect("valid");
        assert_eq!(request.payload, Value::Null);
    }

    #[test]
    fn the_wrong_tag_is_refused_and_names_what_it_got() {
        let error = decode(r#"{"type":"server-response","rpcId":"r","method":"m"}"#).unwrap_err();
        assert!(error.reason.contains("server-response"), "{}", error.reason);
        assert_eq!(
            error.rpc_id.as_deref(),
            Some("r"),
            "the id is still recoverable"
        );
    }

    #[test]
    fn a_missing_or_empty_rpc_id_is_refused() {
        for body in [
            r#"{"type":"client-request","method":"m"}"#,
            r#"{"type":"client-request","rpcId":"","method":"m"}"#,
            r#"{"type":"client-request","rpcId":7,"method":"m"}"#,
        ] {
            let error = decode(body).expect_err(body);
            assert!(error.reason.contains("rpcId"), "{}", error.reason);
            assert_eq!(error.rpc_id, None);
        }
    }

    #[test]
    fn an_over_long_rpc_id_is_refused() {
        let body = format!(
            r#"{{"type":"client-request","rpcId":"{}","method":"m"}}"#,
            "x".repeat(MAX_RPC_ID + 1)
        );
        assert!(decode(&body).is_err());
    }

    #[test]
    fn a_non_json_body_is_refused_without_a_correlation_id() {
        let error = decode("not json").unwrap_err();
        assert!(error.reason.contains("not JSON"));
        assert_eq!(error.rpc_id, None);
    }

    #[test]
    fn responses_use_the_tagged_envelope_the_client_expects() {
        let ok = ServerResponse::new("r1", RpcResult::ok(json!({ "sessionId": "s1" })));
        assert_eq!(
            serde_json::from_slice::<Value>(&ok.to_json()).unwrap(),
            json!({
                "type": "server-response",
                "rpcId": "r1",
                "result": { "ok": true, "value": { "sessionId": "s1" } }
            })
        );

        let failed = ServerResponse::new(
            "r2",
            RpcResult::failed(Failure::at(
                codes::LOOKUP_NOT_FOUND,
                "no such endpoint",
                "x/y",
            )),
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&failed.to_json()).unwrap(),
            json!({
                "type": "server-response",
                "rpcId": "r2",
                "result": {
                    "ok": false,
                    "error": {
                        "code": "gateway/lookup-not-found",
                        "message": "no such endpoint",
                        "details": { "endpoint": "x/y" }
                    }
                }
            })
        );
    }

    #[test]
    fn every_failure_carries_an_object_details_field() {
        let failure = Failure::at(codes::PROTOCOL, "bad", "session/list");
        let value = serde_json::to_value(&failure).unwrap();
        assert!(
            value["details"].is_object(),
            "details must always be an object"
        );
    }
}
