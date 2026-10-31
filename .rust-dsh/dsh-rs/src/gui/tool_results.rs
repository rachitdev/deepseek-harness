//! Process-local executor values. Durable model-facing content is never a value.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Value};
use tokio::sync::{mpsc, Notify};
use tokio_util::sync::CancellationToken;

use super::mux::{deliver, Outcome};
use super::wire::{codes, Failure};
use crate::api::events::{self, AgentDisposedPayload, SessionDisposedPayload};
use crate::api::services::{AgentRegistryService, AgentView};

const SERVICE: &str = "gui.tool-results";
static INSTALL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct SessionValues {
    owner: Weak<dyn AgentView>,
    values: BTreeMap<u64, Value>,
}

#[derive(Default)]
struct LiveValues {
    sessions: Mutex<HashMap<String, SessionValues>>,
    notify: Notify,
}

/// Mount before execution starts. `toolResults/value` must carry the executor's
/// value and committed sequence, not a reconstruction of the model-facing log.
/// The mux also calls this lazily for standalone gateways.
pub(super) async fn observe(ctx: &Context) -> Result<(), cordis::Error> {
    let _install = INSTALL.lock().await;
    if ctx.get::<Arc<LiveValues>>(SERVICE).is_some() {
        return Ok(());
    }
    let state = Arc::new(LiveValues::default());
    ctx.provide(SERVICE, state.clone()).await?;
    let values = state.clone();
    ctx.on_sync("toolResults/value", move |ctx, payload, _| {
        let state = values.clone();
        Box::pin(async move {
            if let (Some(id), Some(seq), Some(value)) = (
                payload["session"].as_str(),
                payload["seq"].as_u64(),
                payload.get("value"),
            ) {
                if let Some(agent) = ctx
                    .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                    .and_then(|agents| agents.get(id))
                {
                    let mut sessions = state.sessions.lock().unwrap();
                    let entry = sessions
                        .entry(id.to_string())
                        .or_insert_with(|| SessionValues {
                            owner: Arc::downgrade(&agent),
                            values: BTreeMap::new(),
                        });
                    if !entry
                        .owner
                        .upgrade()
                        .is_some_and(|old| Arc::ptr_eq(&old, &agent))
                    {
                        entry.owner = Arc::downgrade(&agent);
                        entry.values.clear();
                    }
                    entry.values.insert(seq, value.clone());
                    while entry.values.len() > 1000 {
                        entry.values.pop_first();
                    }
                    state.notify.notify_waiters();
                }
            }
            Ok(Value::Null)
        })
    })
    .await?;
    let disposed_values = state.clone();
    events::on::<AgentDisposedPayload, _>(ctx, move |_, payload| {
        let state = disposed_values.clone();
        Box::pin(async move {
            state.sessions.lock().unwrap().remove(&payload.agent);
            state.notify.notify_waiters();
            Ok(Value::Null)
        })
    })
    .await?;
    events::on::<SessionDisposedPayload, _>(ctx, move |_, payload| {
        let state = state.clone();
        Box::pin(async move {
            state.sessions.lock().unwrap().remove(&payload.session);
            state.notify.notify_waiters();
            Ok(Value::Null)
        })
    })
    .await?;
    Ok(())
}

/// Register owner disposal independently of caller cancellation.
pub(super) async fn lifetime(
    ctx: &Context,
) -> Result<(CancellationToken, cordis::fiber::EffectGuard), cordis::Error> {
    let lifetime = CancellationToken::new();
    let disposed = lifetime.clone();
    let effect = ctx
        .effect("GUI stream lifetime", async move {
            let cleanup: cordis::Disposer = Box::new(move || {
                disposed.cancel();
                Box::pin(async {})
            });
            Ok(Some(cleanup))
        })
        .await?;
    Ok((lifetime, effect))
}

pub(super) async fn watch(
    ctx: &Context,
    args: &Value,
    stream_id: &str,
    out: &mpsc::Sender<String>,
    cancel: &CancellationToken,
) -> Outcome {
    const ENDPOINT: &str = "toolResults/watch";
    if cancel.is_cancelled() {
        return Outcome::Cancelled;
    }
    if let Err(failure) = super::descriptor::check(ENDPOINT, args) {
        return Outcome::Failed(failure);
    }
    let id = args["request"]["sessionId"].as_str().unwrap();
    let seq = args["request"]["seq"].as_f64().unwrap();
    // Never rebind a cold watch, or a disposed instance, to a new agent.
    let owner = ctx
        .get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(id));
    if let Err(error) = observe(ctx).await {
        return Outcome::Failed(Failure::at(codes::INTERNAL, error.to_string(), ENDPOINT));
    }
    let state = ctx.get::<Arc<LiveValues>>(SERVICE).unwrap();
    let (lifetime, effect) = match lifetime(ctx).await {
        Ok(pair) => pair,
        Err(error) => {
            return Outcome::Failed(Failure::at(codes::INTERNAL, error.to_string(), ENDPOINT))
        }
    };
    let work = async {
        let mut last = None;
        loop {
            let notified = state.notify.notified();
            let live = owner.as_ref().is_some_and(|owner| {
                ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
                    .and_then(|agents| agents.get(id))
                    .is_some_and(|current| Arc::ptr_eq(owner, &current))
            });
            let value = if live && seq >= 0.0 && seq.fract() == 0.0 {
                let sessions = state.sessions.lock().unwrap();
                sessions
                    .get(id)
                    .filter(|entry| {
                        entry
                            .owner
                            .upgrade()
                            .zip(owner.as_ref())
                            .is_some_and(|(a, b)| Arc::ptr_eq(&a, b))
                    })
                    .and_then(|entry| entry.values.get(&(seq as u64)))
                    .map(|value| json!({"available": true, "value": value}))
                    .unwrap_or_else(|| json!({"available": false}))
            } else {
                json!({"available": false})
            };
            if last.as_ref() != Some(&value) {
                if deliver(out, stream_id, value.clone(), cancel)
                    .await
                    .is_err()
                {
                    return Outcome::Cancelled;
                }
                last = Some(value);
            }
            tokio::select! {
                _ = cancel.cancelled() => return Outcome::Cancelled,
                _ = notified => {},
                _ = tokio::time::sleep(Duration::from_secs(1)) => {},
            }
        }
    };
    let outcome = tokio::select! {
        biased;
        _ = lifetime.cancelled() => Outcome::Ended,
        _ = cancel.cancelled() => Outcome::Cancelled,
        outcome = work => outcome,
    };
    let _ = effect.dispose().await;
    outcome
}

#[cfg(test)]
pub(super) async fn test_owner() -> (Context, cordis::FiberHandle) {
    let root = Context::new();
    let (send, mut receive) = mpsc::channel(1);
    let plugin = cordis::plugin::plugin("stream-owner-test", move |ctx, _: Value| {
        let send = send.clone();
        async move {
            send.send(ctx).await.unwrap();
            Ok(())
        }
    });
    let owner = root.plugin(plugin, Some(json!({})));
    owner.join().await.unwrap();
    (receive.recv().await.unwrap(), owner)
}
