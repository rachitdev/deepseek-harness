//! The plugins this process started, for the plugin inventory.
//!
//! The kernel offers no listing of the plugins it runs, and a profile is only
//! turned into plugins here, so the bundle records each one it starts. The
//! inventory reads the record on every call and each row reports the fiber's
//! current state, so it never shows a state the kernel has left.
//!
//! A row keeps a state receiver, not the fiber handle: a handle owns the whole
//! runtime, and the roster is itself a service inside that runtime, so holding
//! handles would keep the runtime alive after everything else let go of it.

use std::sync::{Arc, Mutex, MutexGuard};

use cordis::{Context, FiberHandle, FiberState, Plugin};
use serde_json::Value;
use tokio::sync::watch;

/// The service name the roster is provided under.
pub const ROSTER_SERVICE: &str = "pluginRoster";

/// Name of the plugin that provides the roster. It is machinery of this
/// bundle, not a plugin an operator configured, so it is not recorded.
pub const ROSTER_PLUGIN: &str = "plugin-roster";

/// Where a row reads its state from.
#[derive(Clone)]
enum Source {
    /// Not started: the profile switched the plugin off.
    Off,
    /// A running fiber's lifecycle.
    Live(watch::Receiver<FiberState>),
    /// A fiber that has no lifecycle to follow, with the state it ended in.
    Ended(FiberState),
}

/// One plugin as the inventory shows it.
#[derive(Clone)]
pub struct RosterRow {
    /// The plugin's name, which is also its entry id: a profile names each
    /// plugin once.
    pub name: String,
    source: Source,
}

impl RosterRow {
    /// Whether the plugin was started.
    pub fn enabled(&self) -> bool {
        !matches!(self.source, Source::Off)
    }

    /// The fiber's current state; `None` for a plugin that was never started.
    pub fn state(&self) -> Option<FiberState> {
        match &self.source {
            Source::Off => None,
            Source::Live(receiver) => Some(*receiver.borrow()),
            Source::Ended(state) => Some(*state),
        }
    }
}

/// Every plugin the bundle started or switched off, in first-seen order, one
/// row per name. Recording a name again replaces its row, so installing a
/// plugin again or switching it off later leaves one current row.
#[derive(Default)]
pub struct PluginRoster {
    rows: Mutex<Vec<RosterRow>>,
}

impl PluginRoster {
    fn lock(&self) -> MutexGuard<'_, Vec<RosterRow>> {
        // A row is replaced or appended whole, so a panic elsewhere cannot
        // leave one half written.
        self.rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn record(&self, name: String, source: Source) {
        let mut rows = self.lock();
        let row = RosterRow { name, source };
        match rows.iter_mut().find(|existing| existing.name == row.name) {
            Some(existing) => *existing = row,
            None => rows.push(row),
        }
    }

    /// Record a started plugin. The roster's own provider is not recorded.
    pub fn started(&self, handle: &FiberHandle) {
        let name = handle.name();
        if name == ROSTER_PLUGIN {
            return;
        }
        let source = match handle.watch_state() {
            Some(receiver) => Source::Live(receiver),
            None => Source::Ended(handle.state()),
        };
        self.record(name, source);
    }

    /// Record a plugin the profile switched off.
    pub fn skipped(&self, name: &str) {
        self.record(name.to_string(), Source::Off);
    }

    /// The rows in first-seen order.
    pub fn rows(&self) -> Vec<RosterRow> {
        self.lock().clone()
    }
}

/// The plugin that provides `roster` as the `pluginRoster` service.
pub fn roster_plugin(roster: Arc<PluginRoster>) -> Arc<dyn Plugin> {
    cordis::plugin::plugin(ROSTER_PLUGIN, move |ctx, _config: Value| {
        let roster = roster.clone();
        async move {
            ctx.provide(ROSTER_SERVICE, roster).await?;
            Ok(())
        }
    })
}

/// The roster a bundle installed on `ctx`, if any.
pub fn roster_of(ctx: &Context) -> Option<Arc<PluginRoster>> {
    ctx.get::<Arc<PluginRoster>>(ROSTER_SERVICE)
        .map(|service| (*service).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_has_one_row_and_a_later_record_replaces_it() {
        let roster = PluginRoster::default();
        roster.skipped("goal");
        roster.skipped("web");
        roster.record("goal".to_string(), Source::Ended(FiberState::Failed));
        let rows = roster.rows();
        let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["goal", "web"], "first-seen order is kept");
        assert_eq!(rows[0].state(), Some(FiberState::Failed));
        assert!(rows[0].enabled());
        assert!(!rows[1].enabled());
        assert_eq!(rows[1].state(), None);
    }

    #[test]
    fn a_live_row_follows_the_fiber_it_watches() {
        let (sender, receiver) = watch::channel(FiberState::Loading);
        let roster = PluginRoster::default();
        roster.record("web".to_string(), Source::Live(receiver));
        assert_eq!(roster.rows()[0].state(), Some(FiberState::Loading));
        sender.send_replace(FiberState::Disposed);
        assert_eq!(roster.rows()[0].state(), Some(FiberState::Disposed));
    }
}
