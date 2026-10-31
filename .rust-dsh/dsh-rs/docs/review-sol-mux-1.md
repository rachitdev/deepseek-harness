gpt-6.1-sol
## Findings

### 1. A blocked socket write stops reads, cancellation, and heartbeat enforcement
**File/function:** `server.rs::pump`  
**Severity:** High — availability/resource retention

Both outbound data and Ping writes are awaited **inside** the selected branch:

```rust
if sink.send(Message::text(text)).await.is_err() {
    break;
}
```

While that write is blocked, the loop cannot poll `source.next()` or the heartbeat ticker.

**Concrete failure:** A peer opens streams and then stops reading. Once TCP backpressure blocks a write, the server cannot process a subsequently received cancel/close or enforce the heartbeat timeout. The connection can remain stuck, retaining its streams and the exclusive `gateway.socket_open` slot.

The TS implementation serializes writes without blocking its receive callbacks.

**Fix:** Drive the writer separately from the reader and heartbeat supervisor. Give outstanding writes a cancellation/deadline path so a dead peer cannot retain the generation indefinitely. Keep one serialized writer to preserve frame order.

---

### 2. Cancellation is not checked during startup or outbound delivery
**File/functions:** `mux.rs::events_stream`, `follow`, `send_item`  
**Severity:** Medium — cancellation race and retained work

`Notify` preserves a cancellation notification, but the stream only consumes it at particular idle points:

- `$events` sends `ready` before checking cancellation.
- `follow` subscribes, constructs and sends the snapshot, and drains all pending events before checking cancellation.
- A blocked `send_item` does not observe cancellation.

**Concrete reproduction:** Send `open`, immediately followed by `cancel`, before the spawned task runs. The task can still send a new `ready` or snapshot after the cancellation was handled. With a full output channel, a cancelled stream can remain blocked on delivery instead of disposing its subscription.

An already queued frame cannot necessarily be recalled; this bug is that **new delivery and startup work proceed without consulting cancellation**.

**Fix:** Use sticky cancellation state, such as `CancellationToken`, and check it before startup and each new delivery. Select cancellation against subscription setup and blocked sends, with cancellation taking precedence where both are ready. Ensure cancellation exits through subscription cleanup.

---

### 3. Removing a stream before enqueueing its terminal frame permits generation mixing
**File/function:** `mux.rs::Mux::open`, spawned task  
**Severity:** Medium — ordering race/protocol deviation

The stream is removed before its terminal frame is constructed and enqueued:

```rust
streams.lock().unwrap().remove(&stream_id);
// ...
out.send(frame.to_string()).await;
```

That creates an interval in which another `open` using the same ID is accepted, although the previous generation's terminal frame is not yet in the output queue.

On a multithreaded runtime, a possible execution is:

1. Old stream removes its entry.
2. New `open` with the same ID is accepted.
3. New stream enqueues an item.
4. Old stream enqueues its error/end.

The client then sees a terminal frame for the old generation after an item for the new one, with no way to distinguish generations. The TS implementation retains ownership until its pump, including terminal delivery, settles.

**Fix:** Keep the stream registered through terminal delivery. Ideally have the serialized writer acknowledge socket-write completion, not merely channel insertion. Do not allow a new generation to claim the ID while the old terminal is still pending.

---

### 4. Heartbeat accounting omits the TS reference's congested-write activity
**File/function:** `server.rs::pump`, outbound branch and ticker branch  
**Severity:** Medium — false disconnections

Only incoming frames reset `unanswered`. A downlink write that actually waits for buffer space and subsequently completes does not count as activity.

This is explicitly different from the TS contract: a delayed write completion counts because congestion can hold the peer's Pong behind data.

Additionally, `tokio::time::interval` defaults to burst catch-up. After a long awaited write, multiple overdue ticks can execute in quick succession and exhaust the unanswered count without allowing a fresh heartbeat interval.

**Fix:** Have the writer report genuinely backpressured write completions to the heartbeat supervisor and treat those as activity. Do not count immediate local-buffer acceptance as peer activity. Use elapsed-time deadlines or an appropriate missed-tick policy rather than counting burst catch-up ticks as separate unanswered heartbeat periods.

This remains necessary even after fixing the blocking-loop problem in finding 1.

---

### 5. Receiving a WebSocket Close drops the transport without flushing the Close reply
**File/function:** `server.rs::pump`, `Message::Close` branch  
**Severity:** Medium — WebSocket protocol deviation

```rust
Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
```

Tungstenite queues a Close response when it receives a Close frame, but that response must still be driven through a write/flush. This branch exits, leaves `close` as `None`, and drops the socket without doing so.

**Effect:** An ordinary peer-initiated close may end as a transport shutdown without the required WebSocket closing handshake.

**Fix:** Distinguish peer Close from EOF/errors. Flush the automatically queued Close response, or otherwise drive Tungstenite's closing handshake, with a bounded timeout. Avoid sending a second Close after the automatic response.

---

### 6. The upgrade accepts malformed `Sec-WebSocket-Key` values
**File/function:** `server.rs::upgrade`  
**Severity:** Low — handshake protocol validation

The code checks only that the header exists, then hashes its raw bytes:

```rust
derive_accept_key(key.as_bytes())
```

`derive_accept_key` computes the response; it does not validate that the supplied key is valid Base64 decoding to exactly 16 bytes. Consequently, keys such as `x` can receive a `101` response.

**Fix:** Validate the key before reserving `socket_open`, or use a complete WebSocket handshake validator rather than manually implementing only part of the handshake.

## Categories with no additional findings

- **Session event ordering/gaps:** No concrete bug found in the shown subscribe-before-snapshot and log-draining approach, assuming the session log itself is append-ordered and notifications occur after append.
- **Independent leaks:** No additional proven leak beyond the stalled-write and cancellation retention issues above.
- **Authentication/authorization or data-exposure security:** No concrete finding from this excerpt. `upgrade_rejection` and the relevant authorization logic are not supplied.
- **Other parser parity:** Cannot establish additional deviations without the TS `parseRemoteStreamClientMessage` implementation.

The deliberately excluded uplink, host-event forwarding, and session projection/assistant/subagent limitations are not included as findings.
