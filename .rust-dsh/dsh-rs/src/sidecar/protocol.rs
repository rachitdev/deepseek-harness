//! The sidecar's wire protocol.
//!
//! One request per frame, one answer per frame. A frame is a 4-byte
//! big-endian length followed by that many bytes of UTF-8 JSON. The length is
//! the frame's payload, not its header, and a frame longer than
//! [`MAX_FRAME_BYTES`] is a protocol error.
//!
//! The messages are the ones `docs/specs/node-sidecar.md` defines: the startup
//! exchange (`hello`, `accepted`, `handshake`), the ordinary calls (`call`,
//! `result`, `failure`) and the cancellation (`cancel`, `cancelled`).

use serde_json::{json, Value};

/// The protocol version this build speaks.
pub const PROTOCOL_VERSION: u32 = 1;

/// The largest frame the protocol allows.
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// The kind of a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// The sidecar greets the host and carries the token.
    Hello { token: String },
    /// The host accepted the token.
    Accepted { id: String },
    /// The host asks what the sidecar is.
    Handshake,
    /// The sidecar answers what it is.
    HandshakeResult { id: String, protocol: u32, inventory: String, packages: Value },
    /// A call to one of the sidecar's endpoints.
    Call { id: String, method: String, args: Value },
    /// The answer to a call.
    Result { id: String, value: Value },
    /// The failure of a call.
    Failure { id: String, error: Value },
    /// The host cancels a call.
    Cancel { id: String },
    /// The sidecar acknowledges a cancellation.
    Cancelled { id: String },
    /// A frame this build does not know.
    Unknown { kind: String },
}

impl Message {
    /// Decode a frame's payload.
    pub fn decode(payload: &[u8]) -> Result<Message, String> {
        let value: Value = serde_json::from_slice(payload).map_err(|error| format!("the frame is not JSON: {error}"))?;
        let kind = value["kind"].as_str().unwrap_or_default();
        let id = || value["id"].as_str().unwrap_or_default().to_string();
        Ok(match kind {
            "hello" => Message::Hello { token: value["token"].as_str().unwrap_or_default().to_string() },
            "accepted" => Message::Accepted { id: id() },
            "handshake" => Message::Handshake,
            "handshakeResult" => Message::HandshakeResult {
                id: id(),
                protocol: value["protocol"].as_u64().unwrap_or_default() as u32,
                inventory: value["inventory"].as_str().unwrap_or_default().to_string(),
                packages: value["packages"].clone(),
            },
            "call" => Message::Call {
                id: id(),
                method: value["method"].as_str().unwrap_or_default().to_string(),
                args: value["args"].clone(),
            },
            "result" => Message::Result { id: id(), value: value["value"].clone() },
            "failure" => Message::Failure { id: id(), error: value["error"].clone() },
            "cancel" => Message::Cancel { id: id() },
            "cancelled" => Message::Cancelled { id: id() },
            other => Message::Unknown { kind: other.to_string() },
        })
    }

    /// The frame's payload.
    pub fn encode(&self) -> Vec<u8> {
        let value = match self {
            Message::Hello { token } => json!({ "v": PROTOCOL_VERSION, "kind": "hello", "token": token }),
            Message::Accepted { id } => json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "accepted" }),
            Message::Handshake => json!({ "v": PROTOCOL_VERSION, "kind": "handshake" }),
            Message::HandshakeResult { id, protocol, inventory, packages } => json!({
                "v": PROTOCOL_VERSION, "id": id, "kind": "handshakeResult", "protocol": protocol,
                "inventory": inventory, "packages": packages,
            }),
            Message::Call { id, method, args } => {
                json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "call", "method": method, "args": args })
            }
            Message::Result { id, value } => json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "result", "value": value }),
            Message::Failure { id, error } => json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "failure", "error": error }),
            Message::Cancel { id } => json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "cancel" }),
            Message::Cancelled { id } => json!({ "v": PROTOCOL_VERSION, "id": id, "kind": "cancelled" }),
            Message::Unknown { kind } => json!({ "v": PROTOCOL_VERSION, "kind": kind }),
        };
        serde_json::to_vec(&value).unwrap_or_default()
    }
}

/// One framed message: its length and its payload.
pub fn frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 4);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

/// Read one frame from `bytes`, returning the payload and the rest.
///
/// `None` means the frame is incomplete; `Err` means it is longer than the
/// protocol allows.
pub fn take_frame(bytes: &[u8]) -> Result<Option<(Vec<u8>, &[u8])>, String> {
    if bytes.len() < 4 {
        return Ok(None);
    }
    let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(format!("the frame is {length} bytes, over the protocol's limit"));
    }
    if bytes.len() < 4 + length {
        return Ok(None);
    }
    Ok(Some((bytes[4..4 + length].to_vec(), &bytes[4 + length..])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_startup_frames_carry_their_id() {
        let accepted = Message::Accepted { id: "h1".into() };
        assert_eq!(Message::decode(&accepted.encode()).unwrap(), accepted);
        let handshake = Message::HandshakeResult {
            id: "h2".into(),
            protocol: 1,
            inventory: "abc".into(),
            packages: json!({}),
        };
        assert_eq!(Message::decode(&handshake.encode()).unwrap(), handshake);
    }

    #[test]
    fn a_message_survives_the_round_trip() {
        let message = Message::Call { id: "7".into(), method: "pluginManager/listBundles".into(), args: json!({ "x": 1 }) };
        let payload = message.encode();
        assert_eq!(Message::decode(&payload).unwrap(), message);
    }

    #[test]
    fn a_frame_carries_its_length() {
        let payload = Message::Accepted { id: String::new() }.encode();
        let framed = frame(&payload);
        assert_eq!(&framed[..4], &(payload.len() as u32).to_be_bytes());
        let (taken, rest) = take_frame(&framed).unwrap().unwrap();
        assert_eq!(taken, payload);
        assert!(rest.is_empty());
        assert_eq!(Message::decode(&taken).unwrap(), Message::Accepted { id: String::new() });
    }

    #[test]
    fn an_incomplete_frame_is_not_taken() {
        let payload = Message::Accepted { id: String::new() }.encode();
        let framed = frame(&payload);
        assert!(take_frame(&framed[..framed.len() - 1]).unwrap().is_none());
        assert!(take_frame(&framed[..3]).unwrap().is_none());
    }

    #[test]
    fn an_oversized_frame_is_refused() {
        let mut framed = frame(b"x");
        framed[..4].copy_from_slice(&(MAX_FRAME_BYTES as u32 + 1).to_be_bytes());
        assert!(take_frame(&framed).is_err());
    }

    #[test]
    fn an_unknown_kind_is_kept_as_it_is() {
        let message = Message::decode(br#"{"kind":"somethingElse"}"#).unwrap();
        assert_eq!(message, Message::Unknown { kind: "somethingElse".into() });
    }
}
