//! `pluginRegistryProbe/fastest`: which public npm registry answers first.
//!
//! The plugin manager dialog asks once to pick its starting registry. The two
//! public registries are pinged at once and the first successful answer wins;
//! the slower request is dropped, which cancels it. Readers that arrive while a
//! probe runs share it. A winner is remembered for the life of the process. An
//! unreachable pair is not remembered, because a host that started offline
//! should find a registry once it is online; the reply is then `null`, as the
//! TypeScript host answers, and every failed ping is logged with its reason.

use std::sync::Arc;
use std::time::Duration;

use futures_util::future::{select_all, FutureExt, Shared};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use super::router::{check_descriptor, Router};
use super::wire::RpcResult;
use crate::api::services::BoxFuture;

/// The registries compared.
const REGISTRIES: [&str; 2] = [
    "https://registry.npmjs.org/",
    "https://registry.npmmirror.com/",
];

/// Pings a registry.
pub trait Pinger: Send + Sync {
    /// Ping `registry` (a URL ending in `/`); the error says why it did not answer.
    fn ping(&self, registry: &str) -> BoxFuture<Result<(), String>>;
}

/// Pings with an HTTPS request to the registry's `-/ping` route.
pub struct HttpPinger {
    client: reqwest::Client,
}

impl HttpPinger {
    /// A pinger whose requests give up after `timeout`.
    pub fn new(timeout: Duration) -> Result<Self, String> {
        reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map(|client| HttpPinger { client })
            .map_err(|e| format!("cannot build the registry probe client: {e}"))
    }
}

impl Pinger for HttpPinger {
    fn ping(&self, registry: &str) -> BoxFuture<Result<(), String>> {
        let request = self.client.get(format!("{registry}-/ping"));
        Box::pin(async move {
            let response = request.send().await.map_err(|e| e.to_string())?;
            match response.status() {
                status if status.is_success() => Ok(()),
                status => Err(format!("answered {status}")),
            }
        })
    }
}

/// What one probe found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probed {
    /// The first registry that answered, if any did.
    pub winner: Option<&'static str>,
    /// Each registry that failed before the probe finished, with the reason.
    pub failures: Vec<(&'static str, String)>,
}

type Flight = Shared<BoxFuture<Probed>>;

#[derive(Default)]
struct State {
    winner: Option<&'static str>,
    flight: Option<Flight>,
}

/// The fastest-registry probe and its remembered winner.
pub struct RegistryProbe {
    pinger: Arc<dyn Pinger>,
    state: Mutex<State>,
}

impl RegistryProbe {
    /// A probe that asks `pinger`.
    pub fn new(pinger: Arc<dyn Pinger>) -> Self {
        RegistryProbe {
            pinger,
            state: Mutex::new(State::default()),
        }
    }

    /// The probe against the real registries, each request bounded by `timeout`.
    pub fn system(timeout: Duration) -> Result<Self, String> {
        Ok(Self::new(Arc::new(HttpPinger::new(timeout)?)))
    }

    fn race(&self) -> Flight {
        let pings: Vec<_> = REGISTRIES
            .iter()
            .map(|registry| {
                let ping = self.pinger.ping(registry);
                Box::pin(async move { (*registry, ping.await) })
            })
            .collect();
        let race: BoxFuture<Probed> = Box::pin(async move {
            let mut pending = pings;
            let mut failures = Vec::new();
            while !pending.is_empty() {
                let ((registry, outcome), _, rest) = select_all(pending).await;
                pending = rest;
                match outcome {
                    Ok(()) => {
                        return Probed {
                            winner: Some(registry),
                            failures,
                        }
                    }
                    Err(reason) => failures.push((registry, reason)),
                }
            }
            Probed {
                winner: None,
                failures,
            }
        });
        race.shared()
    }

    /// The registry that answered first, or none. Readers that arrive during a
    /// probe share its result; a later call after a failed probe tries again.
    pub async fn fastest(&self) -> Probed {
        let flight = {
            let mut state = self.state.lock().await;
            if let Some(winner) = state.winner {
                return Probed {
                    winner: Some(winner),
                    failures: Vec::new(),
                };
            }
            state.flight.get_or_insert_with(|| self.race()).clone()
        };
        let probed = flight.await;
        let mut state = self.state.lock().await;
        state.flight = None;
        if probed.winner.is_some() {
            state.winner = probed.winner;
        }
        probed
    }
}

/// Serve `pluginRegistryProbe/fastest`, or `None` for any other method.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &cordis::Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    if method != "pluginRegistryProbe/fastest" {
        return None;
    }
    Some(match check_descriptor(method, &[], payload) {
        Ok(()) => {
            let probed = router.registry_probe.fastest().await;
            for (registry, reason) in &probed.failures {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Warn,
                    "gui".to_string(),
                    None,
                    format!("{method}: {registry} did not answer: {reason}"),
                );
            }
            RpcResult::ok(match probed.winner {
                Some(registry) => json!(registry),
                None => Value::Null,
            })
        }
        Err(failure) => RpcResult::failed(failure),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Answers each registry after its own delay.
    struct Scripted {
        calls: AtomicUsize,
        plans: [(Duration, bool); 2],
    }

    impl Pinger for Scripted {
        fn ping(&self, registry: &str) -> BoxFuture<Result<(), String>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (delay, ok) = self.plans[usize::from(registry == REGISTRIES[1])];
            Box::pin(async move {
                tokio::time::sleep(delay).await;
                if ok {
                    Ok(())
                } else {
                    Err("connection refused".to_string())
                }
            })
        }
    }

    fn probe(plans: [(Duration, bool); 2]) -> (RegistryProbe, Arc<Scripted>) {
        let pinger = Arc::new(Scripted {
            calls: AtomicUsize::new(0),
            plans,
        });
        (RegistryProbe::new(pinger.clone()), pinger)
    }

    const FAST: Duration = Duration::from_millis(5);
    const SLOW: Duration = Duration::from_millis(400);

    #[tokio::test]
    async fn the_first_successful_answer_wins_without_waiting_for_the_other() {
        let (probe, _) = probe([(SLOW, true), (FAST, true)]);
        let started = std::time::Instant::now();
        assert_eq!(probe.fastest().await.winner, Some(REGISTRIES[1]));
        assert!(
            started.elapsed() < SLOW,
            "the slower request is dropped, not awaited"
        );
    }

    #[tokio::test]
    async fn a_failure_that_comes_first_does_not_win_but_is_reported() {
        let (probe, _) = probe([(FAST, false), (SLOW, true)]);
        let probed = probe.fastest().await;
        assert_eq!(probed.winner, Some(REGISTRIES[1]));
        assert_eq!(
            probed.failures,
            [(REGISTRIES[0], "connection refused".to_string())]
        );
    }

    #[tokio::test]
    async fn a_winner_is_remembered_and_readers_share_one_probe() {
        let (probe, pinger) = probe([(FAST, true), (SLOW, true)]);
        let (a, b) = tokio::join!(probe.fastest(), probe.fastest());
        assert_eq!(
            (a.winner, b.winner),
            (Some(REGISTRIES[0]), Some(REGISTRIES[0]))
        );
        assert_eq!(probe.fastest().await.winner, Some(REGISTRIES[0]));
        assert_eq!(
            pinger.calls.load(Ordering::SeqCst),
            2,
            "one probe of two registries served all three readers"
        );
    }

    #[tokio::test]
    async fn concurrent_readers_of_an_offline_pair_share_one_failed_probe() {
        let (probe, pinger) = probe([(FAST, false), (FAST, false)]);
        let (a, b, c) = tokio::join!(probe.fastest(), probe.fastest(), probe.fastest());
        assert_eq!((a.winner, b.winner, c.winner), (None, None, None));
        assert_eq!(a.failures.len(), 2, "both reasons are kept");
        assert_eq!(pinger.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_offline_result_is_not_remembered() {
        let (probe, pinger) = probe([(FAST, false), (FAST, false)]);
        assert_eq!(probe.fastest().await.winner, None);
        assert_eq!(probe.fastest().await.winner, None);
        assert_eq!(
            pinger.calls.load(Ordering::SeqCst),
            4,
            "an offline start must not be remembered"
        );
    }
}
