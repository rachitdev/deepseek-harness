//! The queue of frames one attachment has not delivered yet.
//!
//! The TypeScript host gives every attachment a `TerminalFollower`: an ordered
//! queue bounded by the bytes its frames take. The output frames, the state
//! frames and the acknowledgements all go through that one queue, so the
//! client sees them in the order the terminal produced them. A follower whose
//! queue grows past the bound is failed, and the client is told to reconnect
//! for the current screen.

use std::collections::VecDeque;

use serde_json::Value;
use tokio::sync::Notify;

/// The message a follower that grew past its bound is failed with.
pub const BUFFER_EXCEEDED: &str = "Terminal output consumer exceeded its buffer; reconnect to recover the current screen";

#[derive(Default)]
struct Inner {
    queue: VecDeque<Value>,
    /// The bytes the queued frames take.
    bytes: usize,
    /// The failure that ended the follower.
    failure: Option<String>,
    /// The terminal closed: the queued frames are still delivered.
    finished: bool,
}

/// One attachment's frame queue.
pub struct Follower {
    /// A plain lock: a push must never wait for the attachment that reads the
    /// queue, so one slow client cannot stall the others.
    inner: std::sync::Mutex<Inner>,
    wake: Notify,
    /// The byte bound.
    cap: usize,
}

impl Follower {
    /// A follower whose queue may hold `cap` bytes of frames.
    pub fn new(cap: usize) -> Self {
        Follower { inner: std::sync::Mutex::new(Inner::default()), wake: Notify::new(), cap }
    }

    /// Add a frame. The follower fails when the queue would pass its bound.
    pub fn push(&self, frame: Value) {
        let bytes = serde_json::to_string(&frame).map(|text| text.len()).unwrap_or(0);
        let mut inner = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
        if inner.failure.is_some() || inner.finished {
            return;
        }
        if inner.bytes + bytes > self.cap {
            inner.failure = Some(BUFFER_EXCEEDED.to_string());
            inner.queue.clear();
            inner.bytes = 0;
            self.wake.notify_one();
            return;
        }
        inner.bytes += bytes;
        inner.queue.push_back(frame);
        drop(inner);
        self.wake.notify_one();
    }

    /// The next frame, waiting for one.
    ///
    /// `None` means the terminal finished and no frame is left. A failure is
    /// the follower's own: the client must reconnect for the current screen.
    pub async fn next(&self) -> Option<Result<Value, String>> {
        loop {
            {
                let mut inner = self.inner.lock().unwrap_or_else(|poison| poison.into_inner());
                if let Some(failure) = &inner.failure {
                    return Some(Err(failure.clone()));
                }
                if let Some(frame) = inner.queue.pop_front() {
                    inner.bytes = inner.bytes.saturating_sub(serde_json::to_string(&frame).map(|t| t.len()).unwrap_or(0));
                    return Some(Ok(frame));
                }
                if inner.finished {
                    return None;
                }
            }
            self.wake.notified().await;
        }
    }

    /// The terminal closed: the queued frames are delivered, then the queue ends.
    pub fn finish(&self) {
        self.inner.lock().unwrap_or_else(|poison| poison.into_inner()).finished = true;
        self.wake.notify_one();
    }

    /// Whether the follower was failed.
    pub fn failure(&self) -> Option<String> {
        self.inner.lock().unwrap_or_else(|poison| poison.into_inner()).failure.clone()
    }
}

/// Decodes UTF-8 across chunk boundaries.
///
/// The TypeScript host decodes the process output with a streaming decoder, so
/// a character split across two reads is one character, not two replacement
/// marks. This decoder keeps the incomplete tail and returns it with the next
/// chunk.
#[derive(Default)]
pub struct Utf8Decoder {
    tail: Vec<u8>,
}

impl Utf8Decoder {
    /// A decoder with no tail.
    pub fn new() -> Self {
        Utf8Decoder::default()
    }

    /// The text the chunk completes, and the tail it holds back.
    pub fn push(&mut self, data: &[u8]) -> String {
        let mut buffer = std::mem::take(&mut self.tail);
        buffer.extend_from_slice(data);
        let end = complete_prefix(&buffer);
        self.tail = buffer[end..].to_vec();
        String::from_utf8_lossy(&buffer[..end]).into_owned()
    }

    /// The text the last chunk left behind.
    pub fn finish(&mut self) -> String {
        let tail = std::mem::take(&mut self.tail);
        String::from_utf8_lossy(&tail).into_owned()
    }
}

/// The length of the prefix of `buffer` that ends on a character boundary.
fn complete_prefix(buffer: &[u8]) -> usize {
    let mut start = buffer.len();
    // Walk back over the continuation bytes to the character that starts them.
    while start > 0 && (buffer[start - 1] & 0b1100_0000) == 0b1000_0000 {
        start -= 1;
    }
    if start == 0 {
        return buffer.len();
    }
    let lead = buffer[start - 1];
    let width = if lead & 0b1111_1000 == 0b1111_0000 {
        4
    } else if lead & 0b1111_0000 == 0b1110_0000 {
        3
    } else if lead & 0b1110_0000 == 0b1100_0000 {
        2
    } else {
        1
    };
    if buffer.len() - (start - 1) < width {
        // The character is not complete yet.
        return start - 1;
    }
    buffer.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn frames_arrive_in_order() {
        let follower = Follower::new(64 * 1024);
        follower.push(json!({ "n": 1 }));
        follower.push(json!({ "n": 2 }));
        assert_eq!(follower.next().await.unwrap().unwrap(), json!({ "n": 1 }));
        assert_eq!(follower.next().await.unwrap().unwrap(), json!({ "n": 2 }));
    }

    #[tokio::test]
    async fn a_finished_follower_delivers_what_is_left_and_then_ends() {
        let follower = Follower::new(64 * 1024);
        follower.push(json!({ "n": 1 }));
        follower.finish();
        assert_eq!(follower.next().await.unwrap().unwrap(), json!({ "n": 1 }));
        assert!(follower.next().await.is_none());
    }

    #[tokio::test]
    async fn a_queue_that_grows_past_its_bound_fails_the_follower() {
        let follower = Follower::new(64);
        follower.push(json!({ "data": "x".repeat(40) }));
        follower.push(json!({ "data": "y".repeat(40) }));
        let failure = follower.next().await.unwrap().unwrap_err();
        assert_eq!(failure, BUFFER_EXCEEDED);
        assert_eq!(follower.failure().as_deref(), Some(BUFFER_EXCEEDED));
    }

    #[tokio::test]
    async fn a_frame_that_arrives_between_the_check_and_the_wait_is_not_lost() {
        // The consumer polls the queue and then waits; a frame that arrives in
        // that gap must still reach it.
        let follower = std::sync::Arc::new(Follower::new(1024));
        let reader = std::sync::Arc::clone(&follower);
        let consumer = tokio::spawn(async move {
            let mut seen = Vec::new();
            for _ in 0..50 {
                match reader.next().await {
                    Some(Ok(frame)) => seen.push(frame),
                    Some(Err(_)) | None => break,
                }
            }
            seen
        });
        for index in 0..50 {
            follower.push(json!({ "n": index }));
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        let seen = tokio::time::timeout(std::time::Duration::from_secs(5), consumer).await.expect("the consumer woke up");
        assert_eq!(seen.unwrap().len(), 50, "every frame reached the consumer");
    }

    #[test]
    fn a_character_split_across_chunks_is_one_character() {
        let text = "é😀".as_bytes().to_vec();
        let mut decoder = Utf8Decoder::new();
        let mut out = String::new();
        for byte in &text {
            out.push_str(&decoder.push(std::slice::from_ref(byte)));
        }
        out.push_str(&decoder.finish());
        assert_eq!(out, "é😀");
    }

    #[test]
    fn an_incomplete_character_is_held_back() {
        let mut decoder = Utf8Decoder::new();
        // The two-byte character is not complete yet.
        assert_eq!(decoder.push(b"a\xc3"), "a");
        assert_eq!(decoder.push(b"\xa9"), "é");
        assert_eq!(decoder.finish(), "");
    }

    #[test]
    fn a_complete_character_is_delivered_at_once() {
        let mut decoder = Utf8Decoder::new();
        assert_eq!(decoder.push("aé".as_bytes()), "aé");
        assert_eq!(decoder.finish(), "");
    }

    #[test]
    fn invalid_bytes_become_replacement_marks() {
        let mut decoder = Utf8Decoder::new();
        assert_eq!(decoder.push(&[0xff, 0xfe]), "\u{fffd}\u{fffd}");
    }
}
