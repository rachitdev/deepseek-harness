//! Polling generations for `workspaceFiles/changes`.
//!
//! A generation snapshots before `ready` and compares only direct children for
//! directories. Polling coalesces invalidations and cannot observe changes that
//! are reverted between samples; there is no `fs/observed` feed in this build.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, Outcome};
use super::router::boundary;
use super::wire::{codes, Failure};
use crate::api::services::{AgentRegistryService, SessionPersistenceApi};

/// Context service containing the GUI's file-watch sampling configuration.
pub const FILE_CHANGES_SERVICE: &str = "gui.file-changes";

/// Polling settings, provided by the GUI plugin and independently by tests.
#[derive(Debug, Clone)]
pub struct FileChangesConfig {
    /// Milliseconds between samples; zero is clamped to one millisecond.
    pub poll_interval_ms: u64,
}

impl Default for FileChangesConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 300,
        }
    }
}

const ENDPOINT: &str = "workspaceFiles/changes";

/// Serve one watch until cancellation or a filesystem refusal.
pub(super) async fn changes(
    ctx: &Context,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    let (lifetime, effect) = match super::tool_results::lifetime(ctx).await {
        Ok(pair) => pair,
        Err(error) => {
            return Outcome::Failed(Failure::at(codes::INTERNAL, error.to_string(), ENDPOINT))
        }
    };
    let outcome = tokio::select! {
        biased;
        _ = lifetime.cancelled() => Outcome::Ended,
        _ = cancel.cancelled() => Outcome::Cancelled,
        outcome = changes_inner(ctx, args, stream_id, out, cancel) => outcome,
    };
    let _ = effect.dispose().await;
    outcome
}

async fn changes_inner(
    ctx: &Context,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    if cancel.is_cancelled() {
        return Outcome::Cancelled;
    }
    if let Err(failure) = super::descriptor::check(ENDPOINT, args) {
        return Outcome::Failed(failure);
    }
    let Some(session_id) = args.get("workspaceFileScopeId").and_then(Value::as_str) else {
        return Outcome::Failed(boundary(ENDPOINT, "workspaceFileScope"));
    };
    let Some(path) = args.get("path").and_then(Value::as_str) else {
        return Outcome::Failed(boundary(ENDPOINT, "path"));
    };
    let (context, session, shown) = (ctx.clone(), session_id.to_string(), path.to_string());
    let initial = tokio::task::spawn_blocking(move || {
        let root = workspace_root(&context, &session).ok_or_else(|| Failure {
            code: codes::LOOKUP_NOT_FOUND.to_string(),
            // The scope is a lookup the gateway resolves; an identity it cannot
            // resolve is the gateway's own refusal, with the wire field's name.
            message: format!(
                "typert gateway: {ENDPOINT}: lookup provider \"workspaceFileScope\" did not resolve the requested identity"
            ),
            details: json!({ "endpoint": ENDPOINT, "field": "workspaceFileScopeId" }),
        })?;
        let root = fs::canonicalize(&root).map_err(|error| io_failure(&shown, &error))?;
        let target = prospective(&root.join(&shown)).map_err(|error| io_failure(&shown, &error))?;
        let previous = sample(&root, &target, &shown)?;
        Ok::<_, Failure>((root, target, previous))
    })
    .await;
    let (root, target, mut previous) = match initial {
        Ok(Ok(initial)) => initial,
        Ok(Err(failure)) => return Outcome::Failed(failure),
        Err(error) => {
            return Outcome::Failed(Failure::at(codes::INTERNAL, error.to_string(), ENDPOINT))
        }
    };
    let interval = ctx
        .get::<FileChangesConfig>(FILE_CHANGES_SERVICE)
        .map(|config| config.poll_interval_ms)
        .unwrap_or(300);
    if deliver(out, stream_id, json!({ "kind": "ready" }), cancel)
        .await
        .is_err()
    {
        return Outcome::Cancelled;
    }
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Outcome::Cancelled,
            _ = tokio::time::sleep(Duration::from_millis(interval.max(1))) => {}
        }
        let (sample_root, sample_target, shown) = (root.clone(), target.clone(), path.to_string());
        let sampling =
            tokio::task::spawn_blocking(move || sample(&sample_root, &sample_target, &shown));
        let current = match sampling.await {
            Ok(Ok(sample)) => sample,
            Ok(Err(failure)) => return Outcome::Failed(failure),
            Err(error) => {
                return Outcome::Failed(Failure::at(codes::INTERNAL, error.to_string(), ENDPOINT))
            }
        };
        let fingerprint = current.as_ref().map(|sample| &sample.fingerprint);
        if fingerprint != previous.as_ref().map(|sample| &sample.fingerprint) {
            let change = match &current {
                Some(sample) => json!({ "absolutePath": target, "version": sample.version }),
                None => json!({ "absolutePath": target, "absent": true }),
            };
            if deliver(
                out,
                stream_id,
                json!({ "kind": "change", "change": change }),
                cancel,
            )
            .await
            .is_err()
            {
                return Outcome::Cancelled;
            }
        }
        previous = current;
    }
}

/// A missing entry has no fingerprint; directory versions are not fingerprints.
struct Sample {
    fingerprint: Value,
    version: String,
}

fn sample(root: &Path, target: &Path, shown: &str) -> Result<Option<Sample>, Failure> {
    let metadata = match fs::metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_failure(shown, &error)),
    };
    let version = super::files::version_of(&metadata);
    let fingerprint = if metadata.is_dir() {
        // Recheck confinement after replacement, including a new symlink.
        let resolved = fs::canonicalize(target).map_err(|error| io_failure(shown, &error))?;
        if !resolved.starts_with(root) {
            return Err(Failure {
                code: "workspace-file/outside-workspace".to_string(),
                message: "Directory is outside the workspace".to_string(),
                details: json!({ "path": shown }),
            });
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(target).map_err(|error| io_failure(shown, &error))? {
            let entry = entry.map_err(|error| io_failure(shown, &error))?;
            let metadata = match fs::metadata(entry.path()) {
                Ok(metadata) => Some(metadata),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(io_failure(shown, &error)),
            };
            entries.push((
                entry.file_name(),
                metadata.map(|metadata| {
                    (
                        metadata.is_dir(),
                        metadata.is_file(),
                        super::files::version_of(&metadata),
                    )
                }),
            ));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        json!(entries
            .into_iter()
            .map(|(name, version)| (name.to_string_lossy().into_owned(), version))
            .collect::<Vec<_>>())
    } else if metadata.is_file() {
        json!(version)
    } else {
        return Err(unsupported(
            shown,
            "Filesystem cannot watch this target type",
        ));
    };
    Ok(Some(Sample {
        fingerprint,
        version,
    }))
}

// Canonicalize the nearest existing ancestor, retaining a missing suffix.
fn prospective(path: &Path) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or(error)?;
            let base = prospective(parent)?;
            Ok(base.join(path.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "watch path has no file name")
            })?))
        }
        Err(error) => Err(error),
    }
}

// The file module's scope and resolution helpers are private. Keep the same
// live-session-first/header fallback without broadening their visibility.
fn workspace_root(ctx: &Context, session_id: &str) -> Option<PathBuf> {
    if let Some(agent) = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(session_id))
    {
        return agent.session().header_cwd().map(PathBuf::from);
    }
    ctx.get::<Arc<dyn SessionPersistenceApi>>(crate::api::services::SESSION_PERSISTENCE_SERVICE)?
        .header(session_id)
        .ok()
        .flatten()?
        .cwd
        .map(PathBuf::from)
}

fn not_found(path: &str) -> Failure {
    Failure {
        code: "workspace-file/not-found".to_string(),
        message: format!("no entry at \"{path}\""),
        details: json!({ "path": path }),
    }
}

fn unsupported(path: &str, message: &str) -> Failure {
    Failure {
        code: "workspace-file/watch-unsupported".to_string(),
        message: message.to_string(),
        details: json!({ "path": path }),
    }
}

fn io_failure(path: &str, error: &io::Error) -> Failure {
    if error.kind() == io::ErrorKind::NotFound {
        not_found(path)
    } else {
        unsupported(path, &error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn owner_disposal_ends_file_watch() {
        use crate::types::AgentOptions;
        let (ctx, owner) = super::super::tool_results::test_owner().await;
        crate::bundle::install_profile(&ctx, &json!({}))
            .await
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let agent = ctx
            .require::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
            .unwrap()
            .create(
                None,
                AgentOptions::mock("m"),
                Some(directory.path().to_string_lossy().into_owned()),
                None,
                None,
            )
            .unwrap();
        let (out, mut frames) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            changes(
                &ctx,
                &json!({"workspaceFileScopeId":agent.id(), "path":"future.txt"}),
                "files",
                &out,
                &CancellationToken::new(),
            )
            .await
        });
        let ready: Value = serde_json::from_str(&frames.recv().await.unwrap()).unwrap();
        assert_eq!(ready["value"]["kind"], "ready");
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
