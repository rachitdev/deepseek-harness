//! The uplink inbox of one logical stream: the items a client sends while a
//! stream is open.
//!
//! The TypeScript gateway (`packages/api/gateway/src/stream-server.ts`) keeps
//! one `UplinkInbox` per logical stream. The inbox is bounded by the bytes the
//! client's frames take, and two rules end the stream when a client breaks
//! them:
//!
//! - an item after the client half-closed the uplink is a `gateway/protocol`
//!   violation;
//! - an item that would push the buffered bytes past the window is a
//!   `gateway/uplink-overflow` violation.
//!
//! Items for a stream the connection no longer owns are dropped, as in the
//! TypeScript host: a finished stream leaves the table while the client's
//! in-flight frames are still arriving.

use std::collections::VecDeque;

use serde_json::Value;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use super::wire::{codes, Failure};

/// The default window: the bytes one logical stream may hold for the host.
pub const DEFAULT_UPLINK_BYTES: usize = 262_144;

/// One queued client item and the frame bytes it took.
struct Entry {
    value: Value,
    bytes: usize,
}

#[derive(Default)]
struct State {
    queue: VecDeque<Entry>,
    bytes: usize,
    ended: bool,
    closed: bool,
    failure: Option<Failure>,
}

impl State {
    /// The stream failed: the waiting items are dropped, as in the TypeScript host.
    fn fail(&mut self, failure: Failure) {
        if self.failure.is_some() {
            return;
        }
        self.failure = Some(failure);
        self.queue.clear();
        self.bytes = 0;
    }
}

/// The client items of one logical stream.
pub struct Uplink {
    state: std::sync::Mutex<State>,
    wake: Notify,
    max_bytes: usize,
    endpoint: String,
    /// The stream's cancellation: a violation ends the stream, as in the
    /// TypeScript host, where the violation aborts the stream's controller.
    cancel: std::sync::Arc<CancellationToken>,
}

impl Uplink {
    /// An inbox for `endpoint`, bounded by `max_bytes` of client frame bytes.
    pub fn new(endpoint: impl Into<String>, max_bytes: usize, cancel: std::sync::Arc<CancellationToken>) -> Self {
        Uplink {
            state: std::sync::Mutex::new(State::default()),
            wake: Notify::new(),
            max_bytes,
            endpoint: endpoint.into(),
            cancel,
        }
    }

    /// The window the client is told about: the bound and the per-frame overhead.
    pub fn window(&self, stream_id: &str) -> serde_json::Value {
        serde_json::json!({
            "maxBytes": self.max_bytes,
            "frameOverheadBytes": frame_overhead(stream_id),
        })
    }

    /// Take one client item, waiting for the next.
    ///
    /// `None` means the client half-closed the uplink and no item is left.
    /// A protocol violation is reported as a failure.
    pub async fn next(&self) -> Result<Option<Value>, Failure> {
        loop {
            {
                let mut state = self.state.lock().unwrap();
                if let Some(entry) = state.queue.pop_front() {
                    state.bytes -= entry.bytes;
                    return Ok(Some(entry.value));
                }
                if let Some(failure) = &state.failure {
                    return Err(failure.clone());
                }
                if state.ended || state.closed {
                    return Ok(None);
                }
            }
            self.wake.notified().await;
        }
    }

    /// Add one client item; `frame_bytes` is the size of the frame it arrived in.
    ///
    /// A violation is recorded and ends the stream.
    pub fn push(&self, value: Value, frame_bytes: usize) {
        let mut state = self.state.lock().unwrap();
        if state.failure.is_some() || state.closed {
            return;
        }
        if state.ended {
            state.fail(self.violation(
                codes::PROTOCOL,
                "api gateway: Remote stream uplink item after end",
            ));
            self.wake.notify_one();
            self.cancel.cancel();
            return;
        }
        if state.bytes + frame_bytes > self.max_bytes {
            state.fail(self.violation(
                codes::UPLINK_OVERFLOW,
                format!("api gateway: Remote stream uplink exceeded {} buffered bytes", self.max_bytes),
            ));
            self.wake.notify_one();
            self.cancel.cancel();
            return;
        }
        state.bytes += frame_bytes;
        state.queue.push_back(Entry { value, bytes: frame_bytes });
        self.wake.notify_one();
    }

    /// The client half-closed the uplink.
    pub fn end(&self) {
        let mut state = self.state.lock().unwrap();
        state.ended = true;
        self.wake.notify_one();
    }

    /// The stream is over: no further item is accepted.
    pub fn close(&self) {
        let mut state = self.state.lock().unwrap();
        state.closed = true;
        state.queue.clear();
        state.bytes = 0;
        self.wake.notify_one();
    }

    /// The violation that ended the stream, when there was one.
    pub fn failure(&self) -> Option<Failure> {
        self.state.lock().unwrap().failure.clone()
    }

    fn violation(&self, code: &str, message: impl Into<String>) -> Failure {
        Failure {
            code: code.to_string(),
            message: message.into(),
            details: serde_json::json!({ "endpoint": self.endpoint }),
        }
    }
}

/// The bytes a client frame takes beyond its value: the envelope around it.
///
/// The TypeScript host computes it as the length of the item envelope with a
/// `null` value, less the four bytes `null` takes.
pub fn frame_overhead(stream_id: &str) -> usize {
    let envelope = serde_json::json!({ "type": "item", "streamId": stream_id, "value": Value::Null });
    envelope.to_string().len().saturating_sub(4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn items_arrive_in_order() {
        let uplink = Uplink::new("terminal/attach", DEFAULT_UPLINK_BYTES, std::sync::Arc::new(CancellationToken::new()));
        uplink.push(json!({ "n": 1 }), 10);
        uplink.push(json!({ "n": 2 }), 10);
        assert_eq!(uplink.next().await.unwrap(), Some(json!({ "n": 1 })));
        assert_eq!(uplink.next().await.unwrap(), Some(json!({ "n": 2 })));
        uplink.end();
        assert_eq!(uplink.next().await.unwrap(), None);
    }

    #[tokio::test]
    async fn an_item_after_the_end_is_a_protocol_violation() {
        let uplink = Uplink::new("terminal/attach", DEFAULT_UPLINK_BYTES, std::sync::Arc::new(CancellationToken::new()));
        uplink.end();
        uplink.push(json!({}), 10);
        let failure = uplink.next().await.unwrap_err();
        assert_eq!(failure.code, codes::PROTOCOL);
        assert_eq!(failure.message, "api gateway: Remote stream uplink item after end");
        assert_eq!(failure.details, json!({ "endpoint": "terminal/attach" }));
    }

    #[tokio::test]
    async fn too_many_buffered_bytes_overflows_the_window() {
        let uplink = Uplink::new("terminal/attach", 100, std::sync::Arc::new(CancellationToken::new()));
        uplink.push(json!({ "a": 1 }), 60);
        uplink.push(json!({ "b": 2 }), 60);
        let failure = uplink.next().await.unwrap_err();
        assert_eq!(failure.code, codes::UPLINK_OVERFLOW);
        assert_eq!(failure.message, "api gateway: Remote stream uplink exceeded 100 buffered bytes");
        assert_eq!(failure.details, json!({ "endpoint": "terminal/attach" }));
    }

    #[tokio::test]
    async fn a_closed_inbox_takes_no_item_and_waits_for_nothing() {
        let uplink = Uplink::new("terminal/attach", DEFAULT_UPLINK_BYTES, std::sync::Arc::new(CancellationToken::new()));
        uplink.push(json!({ "n": 1 }), 10);
        uplink.close();
        uplink.push(json!({ "n": 2 }), 10);
        assert_eq!(uplink.next().await.unwrap(), None);
        assert!(uplink.failure().is_none());
    }

    #[tokio::test]
    async fn a_violation_cancels_the_stream_it_belongs_to() {
        let cancel = std::sync::Arc::new(CancellationToken::new());
        let uplink = Uplink::new("terminal/attach", 100, std::sync::Arc::clone(&cancel));
        uplink.push(json!({ "a": 1 }), 60);
        uplink.push(json!({ "b": 2 }), 60);
        assert!(cancel.is_cancelled(), "the overflowing stream is ended");
        let failure = uplink.next().await.unwrap_err();
        assert_eq!(failure.code, codes::UPLINK_OVERFLOW);

        let cancel = std::sync::Arc::new(CancellationToken::new());
        let uplink = Uplink::new("terminal/attach", 100, std::sync::Arc::clone(&cancel));
        uplink.end();
        uplink.push(json!({}), 10);
        assert!(cancel.is_cancelled(), "an item after the end ends the stream");
    }

    #[test]
    fn the_window_reports_the_bound_and_the_frame_overhead() {
        let uplink = Uplink::new("terminal/attach", DEFAULT_UPLINK_BYTES, std::sync::Arc::new(CancellationToken::new()));
        let window = uplink.window("a1");
        assert_eq!(window["maxBytes"], 262_144);
        assert_eq!(window["frameOverheadBytes"], frame_overhead("a1"));
        // The envelope around a null value, less the four bytes `null` takes.
        assert_eq!(frame_overhead("a"), 39);
        assert_eq!(frame_overhead("a1"), 40);
        assert_eq!(frame_overhead("at1"), 41);
    }
}
