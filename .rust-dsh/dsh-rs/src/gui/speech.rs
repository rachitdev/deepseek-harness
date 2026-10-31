//! Speech catalog and persisted selection for a host with no recognizers.
//!
//! Audio validation precedes provider resolution, as in the TypeScript remote
//! controller. Configuring preferences is allowed without an installed provider.

use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Map, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, Outcome};
use super::settings::{SettingsStore, SPEECH_NAMESPACE};
use super::wire::{codes, Failure, RpcResult};

const MAX_AUDIO_BYTES: usize = 4 * 1024 * 1024;
const MAX_DURATION_SECONDS: usize = 120;

/// A detached catalog with the saved selection and the controller's limits.
pub(super) fn catalog(settings: &SettingsStore) -> Value {
    let user = settings.user(SPEECH_NAMESPACE).unwrap_or_default();
    json!({
        "providers": [],
        "selection": {
            "providerId": user.get("providerId").and_then(Value::as_str).unwrap_or(""),
            "language": user.get("language").and_then(Value::as_str).unwrap_or("auto"),
        },
        "maxAudioBytes": MAX_AUDIO_BYTES,
        "maxDurationSeconds": MAX_DURATION_SECONDS,
    })
}

fn unavailable(id: &str) -> String {
    format!("Speech provider is unavailable: {id}")
}

fn internal(message: String) -> RpcResult {
    RpcResult::failed(Failure {
        code: codes::INTERNAL.into(),
        message,
        details: json!({}),
    })
}

/// Dispatch the five unary calls after the gateway has checked their arguments.
pub(super) fn dispatch(
    settings: &SettingsStore,
    endpoint: &str,
    args: &Value,
) -> Option<RpcResult> {
    if !matches!(
        endpoint,
        "speech/catalog"
            | "speech/configure"
            | "speech/prepare"
            | "speech/cancelPreparation"
            | "speech/transcribe"
    ) {
        return None;
    }
    if let Err(failure) = super::descriptor::check(endpoint, args) {
        return Some(RpcResult::failed(failure));
    }
    Some(match endpoint {
        "speech/catalog" => RpcResult::ok(catalog(settings)),
        "speech/configure" => {
            // Only declared preference fields belong to this namespace.
            let patch: Map<String, Value> = ["providerId", "language"]
                .into_iter()
                .filter_map(|key| {
                    args["patch"]
                        .get(key)
                        .map(|value| (key.into(), value.clone()))
                })
                .collect();
            match settings.update(SPEECH_NAMESPACE, &patch, None) {
                Ok(_) => RpcResult::Empty,
                Err(failure) => RpcResult::failed(failure),
            }
        }
        "speech/prepare" | "speech/cancelPreparation" => {
            internal(unavailable(args["providerId"].as_str().unwrap_or_default()))
        }
        _ => transcribe(settings, &args["request"]),
    })
}

fn transcription_failed(reason: &str) -> RpcResult {
    RpcResult::failed(Failure {
        code: "speech/transcription-failed".into(),
        message: reason.into(),
        details: json!({ "reason": reason }),
    })
}

fn transcribe(settings: &SettingsStore, request: &Value) -> RpcResult {
    let encoded = request["audioBase64"].as_str().unwrap_or_default();
    if encoded.encode_utf16().count() > MAX_AUDIO_BYTES.div_ceil(3) * 4 {
        return RpcResult::failed(Failure {
            code: "speech/invalid-audio".into(),
            message: "Audio is invalid or exceeds the configured byte limit".into(),
            details: json!({ "reason": "encoding-or-size" }),
        });
    }
    // Strict decoding is equivalent to Buffer's permissive decode followed by
    // the controller's canonical re-encoding check.
    let audio = match STANDARD.decode(encoded) {
        Ok(audio) if STANDARD.encode(&audio) == encoded => audio,
        _ => return transcription_failed("Audio must use canonical base64 encoding"),
    };
    if audio.len() > MAX_AUDIO_BYTES {
        return transcription_failed("Audio exceeds the configured byte limit");
    }
    if let Err(reason) = validate_wave(&audio) {
        return transcription_failed(reason);
    }
    let snapshot = catalog(settings);
    let id = request["providerId"]
        .as_str()
        .or_else(|| snapshot["selection"]["providerId"].as_str())
        .unwrap_or_default();
    transcription_failed(&unavailable(id))
}

fn validate_wave(audio: &[u8]) -> Result<(), &'static str> {
    let u16_at = |i| u16::from_le_bytes([audio[i], audio[i + 1]]);
    let u32_at = |i| u32::from_le_bytes([audio[i], audio[i + 1], audio[i + 2], audio[i + 3]]);
    // Node's ASCII decoding masks the high bit of every byte.
    let tag = |i: usize, expected: &[u8]| {
        audio[i..i + 4]
            .iter()
            .map(|b| b & 0x7f)
            .eq(expected.iter().copied())
    };
    if audio.len() < 46
        || !tag(0, b"RIFF")
        || !tag(8, b"WAVE")
        || !tag(12, b"fmt ")
        || u32_at(16) != 16
        || u16_at(20) != 1
        || u16_at(22) != 1
        || u32_at(24) != 16000
        || u32_at(28) != 32000
        || u16_at(32) != 2
        || u16_at(34) != 16
        || !tag(36, b"data")
        || u32_at(4) as usize != audio.len() - 8
        || u32_at(40) as usize != audio.len() - 44
        || (audio.len() - 44) % 2 != 0
    {
        return Err("Audio must be a canonical 16 kHz mono PCM16 WAV recording");
    }
    if audio.len() - 44 > MAX_DURATION_SECONDS * 32000 {
        return Err("Audio exceeds 120 seconds");
    }
    Ok(())
}

/// Context-aware entry point; the mux must pass the speech owner's context.
#[allow(dead_code)]
pub(super) async fn follow_owned(
    ctx: &cordis::Context,
    settings: &SettingsStore,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let (lifetime, effect) = match super::tool_results::lifetime(ctx).await {
        Ok(pair) => pair,
        Err(error) => {
            return Outcome::Failed(Failure::at(
                codes::INTERNAL,
                error.to_string(),
                "speech/follow",
            ))
        }
    };
    let outcome = tokio::select! {
        biased;
        _ = lifetime.cancelled() => Outcome::Ended,
        _ = cancel.cancelled() => Outcome::Cancelled,
        outcome = follow(settings, args, stream_id, out, cancel) => outcome,
    };
    let _ = effect.dispose().await;
    outcome
}

/// Send the current catalog, then changed snapshots until the client cancels.
/// Settings are sampled so writes through any settings endpoint are visible;
/// slow readers coalesce intermediate changes, like TypeScript's observer.
pub(super) async fn follow(
    settings: &SettingsStore,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if let Err(failure) = super::descriptor::check("speech/follow", args) {
        return Outcome::Failed(failure);
    }
    let mut previous = catalog(settings);
    if deliver(out, stream_id, previous.clone(), cancel)
        .await
        .is_err()
    {
        return Outcome::Cancelled;
    }
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Outcome::Cancelled,
            _ = tick.tick() => {
                let next = catalog(settings);
                if next != previous {
                    if deliver(out, stream_id, next.clone(), cancel).await.is_err() {
                        return Outcome::Cancelled;
                    }
                    previous = next;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn owner_disposal_ends_follow_even_when_output_is_full() {
        let (ctx, owner) = super::super::tool_results::test_owner().await;
        let (out, mut frames) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            let settings = SettingsStore::new(None).unwrap();
            follow_owned(
                &ctx,
                &settings,
                &json!({}),
                "speech",
                &out,
                &CancellationToken::new(),
            )
            .await
        });
        frames.recv().await.unwrap();
        owner.dispose().await;
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(2), task)
                .await
                .unwrap()
                .unwrap(),
            Outcome::Ended
        ));
    }
}
