//! The two terminal streams: `terminal/attach` and `terminal/retain`.
//!
//! `terminal/attach` opens a logical stream and carries the terminal's output
//! and the client's input. The most recent attachment holds the input; the
//! others watch and are told they are read-only. The frames and their order
//! follow the TypeScript host's `BrowserTerminal`.

use std::sync::Arc;

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, MuxConfig, Outcome};
use super::uplink::Uplink;
use super::wire::{codes, Failure};
use crate::terminal::follower::Follower;
use crate::terminal::service::{Control, Terminal, TerminalService};

/// Serve `terminal/attach` and `terminal/retain`.
pub(super) async fn dispatch(
    ctx: &Context,
    config: &MuxConfig,
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
    uplink: &Arc<Uplink>,
) -> Outcome {
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Outcome::Failed(failure);
    }
    let Some(service) = config.terminal.clone() else {
        return Outcome::Failed(Failure::at(
            codes::SERVICE_UNAVAILABLE,
            "the terminal service is not wired into this build",
            endpoint,
        ));
    };
    match endpoint {
        "terminal/attach" => attach(ctx, &service, config, endpoint, args, stream_id, out, cancel, uplink).await,
        _ => retain(&service, args, stream_id, out, cancel).await,
    }
}

/// `terminal/attach`: the terminal's frames, and the client's input.
async fn attach(
    ctx: &Context,
    service: &Arc<TerminalService>,
    config: &MuxConfig,
    endpoint: &str,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
    uplink: &Arc<Uplink>,
) -> Outcome {
    let agent = args.get("agentId").and_then(Value::as_str).unwrap_or_default();
    let id = args.get("id").and_then(Value::as_str).unwrap_or_default();
    if let Err(failure) = super::router::require_session(ctx, endpoint, agent) {
        return Outcome::Failed(failure);
    }
    let terminal = match service.terminal(agent, id).await {
        Ok(terminal) => terminal,
        Err(failure) => return Outcome::Failed(failure),
    };
    let cap = config.terminal.as_ref().map(|service| service.config.max_buffered_bytes).unwrap_or(2 * 1024 * 1024);
    let (attachment, follower, snapshot) = service.attach(&terminal, cap, uplink.window(stream_id)).await;
    // The snapshot is the first frame, before the queue is drained.
    if deliver(out, stream_id, snapshot, cancel).await.is_err() {
        service.detach(&terminal, attachment).await;
        return Outcome::Cancelled;
    }

    let mut expected: u64 = 0;
    let mut input_open = true;
    let outcome = loop {
        let next_input = async {
            match input_open {
                true => uplink.next().await,
                false => std::future::pending().await,
            }
        };
        tokio::select! {
            _ = cancel.cancelled() => break Outcome::Cancelled,
            _ = terminal.lifetime().cancelled() => break Outcome::Ended,
            frame = follower.next() => {
                match frame {
                    // The terminal closed: the frames it holds are delivered first.
                    None => break Outcome::Ended,
                    Some(Err(message)) => break Outcome::Failed(Failure {
                        code: codes::INTERNAL.to_string(),
                        message,
                        details: json!({}),
                    }),
                    Some(Ok(frame)) => {
                        if deliver(out, stream_id, frame, cancel).await.is_err() {
                            break Outcome::Cancelled;
                        }
                    }
                }
            }
            item = next_input => {
                match item {
                    Ok(Some(value)) => {
                        match apply_input(service, &terminal, &follower, attachment, &value, &mut expected).await {
                            // The acknowledgement goes through the same queue as the
                            // frames, so the client sees them in the order the
                            // terminal produced them.
                            Ok(()) => {}
                            Err(failure) => break Outcome::Failed(failure),
                        }
                    }
                    // The client half-closed its uplink: the terminal keeps streaming.
                    Ok(None) => input_open = false,
                    Err(failure) => break Outcome::Failed(failure),
                }
            }
        }
    };
    service.detach(&terminal, attachment).await;
    let _ = config;
    outcome
}

/// Apply one client item and answer its acknowledgement.
///
/// The acknowledgement goes into the attachment's queue, so it reaches the
/// client after the frames the command produced.
async fn apply_input(
    service: &Arc<TerminalService>,
    terminal: &Arc<Terminal>,
    follower: &Arc<Follower>,
    attachment: u64,
    value: &Value,
    expected: &mut u64,
) -> Result<(), Failure> {
    let sequence = decode_input(value)?;
    if sequence != (*expected + 1) as f64 {
        return Err(Failure {
            code: codes::INTERNAL.to_string(),
            message: "Terminal input sequence has a gap".to_string(),
            details: json!({}),
        });
    }
    // The counter follows the items the host accepted.
    *expected += 1;
    // The process state decides first: a terminal that has ended refuses every
    // input, whoever holds the input.
    let info = terminal.info().await;
    let ended = info["state"] != "running" || terminal.pty.exited();
    if ended {
        follower.push(json!({ "type": "ack", "sequence": sequence, "refusal": "not-running" }));
        return Ok(());
    }
    // Another attachment holds the input: the client is told, not refused.
    if service.control(terminal, attachment).await == Control::ReadOnly {
        follower.push(json!({ "type": "ack", "sequence": sequence, "refusal": "read-only" }));
        return Ok(());
    }
    match value["type"].as_str().unwrap_or_default() {
        "write" => {
            let data = value["data"].as_str().unwrap_or_default();
            if data.len() > service.config.max_input_bytes {
                return Err(Failure {
                    code: codes::INTERNAL.to_string(),
                    message: "Terminal input exceeds the configured limit".to_string(),
                    details: json!({}),
                });
            }
            // The acknowledgement is queued before the write, so the echo of
            // the input cannot overtake it. The write itself does not hold the
            // frame lock: a child that is not reading its input must not stop
            // the terminal from reading its output.
            follower.push(json!({ "type": "ack", "sequence": *expected }));
            terminal.pty.write(data.as_bytes().to_vec()).await.map_err(|error| Failure {
                code: codes::INTERNAL.to_string(),
                message: error.to_string(),
                details: json!({}),
            })?;
            // The terminal was used, so an idle observation does not survive it.
            terminal.touch();
        }
        "resize" => {
            let cols = value["cols"].as_f64().filter(|n| n.is_finite() && n.fract() == 0.0).map(|n| n as u16);
            let rows = value["rows"].as_f64().filter(|n| n.is_finite() && n.fract() == 0.0).map(|n| n as u16);
            let (Some(cols), Some(rows)) = (cols, rows) else {
                return Err(dimensions_error());
            };
            if cols < 2 || cols > service.config.max_cols || rows < 1 || rows > service.config.max_rows {
                return Err(dimensions_error());
            }
            terminal.pty.resize(cols, rows).map_err(|error| Failure {
                code: codes::INTERNAL.to_string(),
                message: error.to_string(),
                details: json!({}),
            })?;
            terminal.screen.lock().await.resize(cols, rows);
            terminal.info.lock().await["cols"] = json!(cols);
            terminal.info.lock().await["rows"] = json!(rows);
            // The state reaches the client before the acknowledgement, as in
            // the TypeScript host.
            terminal.broadcast_state_to(None).await;
            follower.push(json!({ "type": "ack", "sequence": *expected }));
        }
        _ => {
            return Err(Failure {
                code: codes::INTERNAL.to_string(),
                message: "Terminal input sequence has a gap".to_string(),
                details: json!({}),
            })
        }
    }
    Ok(())
}

fn dimensions_error() -> Failure {
    Failure {
        code: codes::INTERNAL.to_string(),
        message: "Terminal dimensions exceed the configured limits".to_string(),
        details: json!({}),
    }
}

/// The client's sequence number.
///
/// The TypeScript host decodes an uplink item with a zod object schema, which
/// strips the keys it does not declare and accepts any number as a sequence.
/// An item that does not fit the schema at all is refused here; a sequence
/// that is not the next one is the host's "sequence has a gap" failure.
fn decode_input(value: &Value) -> Result<f64, Failure> {
    let refused = || Failure {
        code: codes::INPUT_INVALID.to_string(),
        message: "typert gateway: terminal/attach: wire field \"uplink\" failed boundary validation".to_string(),
        details: json!({ "endpoint": "terminal/attach", "field": "uplink" }),
    };
    let Some(object) = value.as_object() else {
        return Err(refused());
    };
    let allowed: &[&str] = match object.get("type").and_then(Value::as_str) {
        Some("write") => &["type", "sequence", "data"],
        Some("resize") => &["type", "sequence", "cols", "rows"],
        _ => return Err(refused()),
    };
    // The undeclared keys are dropped, as the TypeScript codec drops them.
    let mut item = serde_json::Map::new();
    for key in allowed {
        if let Some(field) = object.get(*key) {
            item.insert((*key).to_string(), field.clone());
        }
    }
    let Some(sequence) = item.get("sequence").and_then(Value::as_f64).filter(|n| n.is_finite()) else {
        return Err(refused());
    };
    match item["type"].as_str() {
        Some("write") => {
            if !item["data"].is_string() {
                return Err(refused());
            }
        }
        Some("resize") => {
            if !item["cols"].is_number() || !item["rows"].is_number() {
                return Err(refused());
            }
        }
        _ => return Err(refused()),
    }
    Ok(sequence)
}

/// `terminal/retain`: hold a terminal open for a window, without taking its input.
async fn retain(
    service: &Arc<TerminalService>,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let session = args.get("sessionId").and_then(Value::as_str).unwrap_or_default();
    let id = args.get("id").and_then(Value::as_str).unwrap_or_default();
    match service.terminal(session, id).await {
        Ok(_) => {}
        Err(_) => {
            return Outcome::Failed(Failure {
                code: "terminal/unavailable".to_string(),
                message: "Terminal is closing or unavailable".to_string(),
                details: json!({}),
            })
        }
    }
    let Some(terminal) = service.terminal(session, id).await.ok() else {
        return Outcome::Failed(Failure {
            code: "terminal/unavailable".to_string(),
            message: "Terminal is closing or unavailable".to_string(),
            details: json!({}),
        });
    };
    // The hold keeps the terminal from being reclaimed while the window watches it.
    terminal.hold();
    if deliver(out, stream_id, json!({ "type": "retained" }), cancel).await.is_err() {
        terminal.release();
        return Outcome::Cancelled;
    }
    let outcome = tokio::select! {
        _ = cancel.cancelled() => Outcome::Cancelled,
        // The terminal closed: the hold ends and the stream ends with it.
        _ = terminal.lifetime().cancelled() => Outcome::Ended,
    };
    terminal.release();
    outcome
}
