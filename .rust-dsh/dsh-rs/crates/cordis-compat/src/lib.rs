//! `cordis` compatibility layer for dsh-rs.
//!
//! dsh-rs was written against `cordis-rust` 0.2 (JSON-`Value` event payloads,
//! `Arc<dyn Plugin>`, infallible `ctx.plugin`). This crate keeps that call
//! surface and implements it on top of `jincdream/cordis-rs` (`cordis-rs-core`),
//! so the kernel is jincdream's and can be updated from GitHub independently.
//!
//! Supported: `provide`/`get`/`require`, `plugin`, `effect`, `on`/`on_sync`,
//! `emit`/`emit_sync`/`parallel`/`waterfall`, `logger().log_event`.
//! Not provided (unused by dsh-rs): `serial`, `bail`, `once`, `isolate`,
//! `intercept`, `accessor`, `stop`. Add them here, not in dsh-rs.

use std::any::Any;
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::Value;

use cordis_core as kernel;

/// Boxed `Send` future used throughout the legacy API.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Framework error. Free-form message plus the originating kernel error text.
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    /// Build an error from a message.
    pub fn msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<kernel::CordisError> for Error {
    fn from(error: kernel::CordisError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

/// Result alias used by plugin bodies and listeners.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Log levels (mirrors cordis-rust 0.2).
pub mod logger {
    /// Severity, ordered by increasing severity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LogLevel {
        Debug,
        Info,
        Warn,
        Error,
    }
}

/// Effect and disposer types.
pub mod fiber {
    use super::BoxFuture;

    /// Cleanup callback run once when the owning fiber unloads.
    pub type Disposer = Box<dyn FnOnce() -> BoxFuture<()> + Send>;

    /// Handle for a registered effect or listener.
    #[derive(Clone)]
    pub struct EffectGuard(pub(crate) cordis_core::EffectHandle);

    impl EffectGuard {
        /// Dispose the effect now.
        pub async fn dispose(&self) -> super::Result<()> {
            self.0.dispose().await.map_err(Into::into)
        }
    }
}

pub use fiber::{Disposer, EffectGuard};
pub use logger::LogLevel;

/// Plugin trait, injections and adapters.
pub mod plugin {
    use super::*;

    pub use super::BoxFuture;

    /// A declared service dependency.
    #[derive(Debug, Clone)]
    pub struct Injection {
        pub service: Cow<'static, str>,
        pub config: Option<Value>,
    }

    impl Injection {
        pub fn new(service: impl Into<Cow<'static, str>>) -> Self {
            Self {
                service: service.into(),
                config: None,
            }
        }

        pub fn with_config(service: impl Into<Cow<'static, str>>, config: Value) -> Self {
            Self {
                service: service.into(),
                config: Some(config),
            }
        }
    }

    impl From<&str> for Injection {
        fn from(value: &str) -> Self {
            Injection::new(value.to_string())
        }
    }

    /// Conversion of the accepted dependency-list shapes.
    pub trait IntoInjections {
        fn into_injections(self) -> Vec<Injection>;
    }

    impl IntoInjections for Vec<Injection> {
        fn into_injections(self) -> Vec<Injection> {
            self
        }
    }

    impl IntoInjections for Injection {
        fn into_injections(self) -> Vec<Injection> {
            vec![self]
        }
    }

    impl IntoInjections for &[&str] {
        fn into_injections(self) -> Vec<Injection> {
            self.iter()
                .map(|name| Injection::new(name.to_string()))
                .collect()
        }
    }

    /// The plugin trait.
    pub trait Plugin: Send + Sync + 'static {
        fn name(&self) -> Cow<'static, str> {
            Cow::Borrowed(std::any::type_name::<Self>())
        }

        fn inject(&self) -> Vec<Injection> {
            Vec::new()
        }

        fn resolve_config(&self, raw: &Value) -> Result<Value> {
            Ok(raw.clone())
        }

        fn apply(&self, ctx: Context, config: Value) -> BoxFuture<Result<()>>;
    }

    struct FnPlugin<C, F> {
        name: Cow<'static, str>,
        inject: Vec<Injection>,
        callback: F,
        _marker: std::marker::PhantomData<fn() -> C>,
    }

    impl<C, F, Fut> Plugin for FnPlugin<C, F>
    where
        C: DeserializeOwned + Send + 'static,
        F: Fn(Context, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        fn name(&self) -> Cow<'static, str> {
            self.name.clone()
        }

        fn inject(&self) -> Vec<Injection> {
            self.inject.clone()
        }

        fn resolve_config(&self, raw: &Value) -> Result<Value> {
            serde_json::from_value::<C>(raw.clone())
                .map_err(|e| Error::msg(format!("invalid config for plugin {}: {e}", self.name)))?;
            Ok(raw.clone())
        }

        fn apply(&self, ctx: Context, config: Value) -> BoxFuture<Result<()>> {
            match serde_json::from_value::<C>(config) {
                Ok(typed) => Box::pin((self.callback)(ctx, typed)),
                Err(e) => {
                    let msg = format!("invalid config for plugin {}: {e}", self.name);
                    Box::pin(async move { Err(Error::msg(msg)) })
                }
            }
        }
    }

    /// Adapt an async closure with a typed config into a plugin.
    pub fn plugin<C, F, Fut>(name: impl Into<Cow<'static, str>>, callback: F) -> Arc<dyn Plugin>
    where
        C: DeserializeOwned + Send + 'static,
        F: Fn(Context, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        plugin_with(name, Vec::<Injection>::new(), callback)
    }

    /// Like [`plugin`], also declaring required services.
    pub fn plugin_with<C, F, Fut>(
        name: impl Into<Cow<'static, str>>,
        inject: impl IntoInjections,
        callback: F,
    ) -> Arc<dyn Plugin>
    where
        C: DeserializeOwned + Send + 'static,
        F: Fn(Context, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Arc::new(FnPlugin {
            name: name.into(),
            inject: inject.into_injections(),
            callback,
            _marker: std::marker::PhantomData,
        })
    }
}

pub use plugin::{plugin, plugin_with, Injection, IntoInjections, Plugin};

/// Waterfall continuation. `run` continues the chain with an updated payload;
/// a listener that never calls it vetoes the rest of the chain.
#[derive(Clone)]
pub struct Next {
    invocation: kernel::EventInvocation,
}

impl Next {
    pub async fn run(self, payload: Value) -> Result<Value> {
        let Some(future) = self.invocation.next(args_of(payload.clone())) else {
            return Ok(payload);
        };
        outcome_value(future.await?)
    }
}

tokio::task_local! {
    /// `true` while a waterfall is dispatching on this task: a listener's
    /// return value is then the chain result. In `serial`/`bail` dispatch
    /// (`false`) only a truthy value stops the chain. Listeners run inline on
    /// the dispatcher's task, so the scope covers them.
    static IN_WATERFALL: bool;
}

fn args_of(value: Value) -> kernel::EventArgs {
    let shared: kernel::SharedValue = Arc::new(value);
    vec![shared]
}

fn outcome_value(outcome: kernel::EventOutcome) -> Result<Value> {
    Ok(match outcome {
        kernel::EventOutcome::Bail(value) => value
            .downcast_ref::<Value>()
            .cloned()
            .unwrap_or(Value::Null),
        kernel::EventOutcome::Continue => Value::Null,
    })
}

/// Whether a value stops serial/bail dispatch: anything but `null`/`false`.
pub fn is_bailed(value: &Value) -> bool {
    !matches!(value, Value::Null | Value::Bool(false))
}

/// Listener signature: dispatching context, JSON payload, waterfall `Next`.
pub type Listener = Arc<dyn Fn(Context, Value, Next) -> BoxFuture<Result<Value>> + Send + Sync>;

/// Logger view (structured events only).
pub struct Logger {
    inner: kernel::Logger,
}

impl Logger {
    pub fn log_event(&self, level: LogLevel, event: String, code: Option<String>, message: String) {
        let prefix = match code {
            Some(code) => format!("[event={event} code={code}]"),
            None => format!("[event={event}]"),
        };
        let line = format!("{prefix} {message}");
        let _ = match level {
            LogLevel::Debug => self.inner.debug(line, vec![]),
            LogLevel::Info => self.inner.info(line, vec![]),
            LogLevel::Warn => self.inner.warn(line, vec![]),
            LogLevel::Error => self.inner.error(line, vec![]),
        };
    }
}

/// Handle to one plugin fiber.
#[derive(Clone)]
pub struct FiberHandle {
    state: Result<kernel::FiberHandle, Error>,
    name: String,
}

pub use kernel::FiberState;

impl FiberHandle {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// A receiver of this plugin's lifecycle state that does not keep the
    /// runtime alive, unlike the handle itself. `None` when the plugin never
    /// started or its fiber is gone; [`FiberHandle::state`] then says why.
    pub fn watch_state(&self) -> Option<tokio::sync::watch::Receiver<FiberState>> {
        self.state.as_ref().ok()?.subscribe_state()
    }

    pub fn state(&self) -> FiberState {
        match &self.state {
            Ok(handle) => handle
                .snapshot()
                .map(|s| s.state)
                .unwrap_or(FiberState::Disposed),
            Err(_) => FiberState::Failed,
        }
    }

    /// Wait until the fiber settles; surface its startup error.
    ///
    /// `Pending` is a settled state only when some declared service is
    /// missing. The kernel marks a fiber started slightly before it takes the
    /// lifecycle lock, so a ready-but-not-yet-loading fiber is polled briefly.
    pub async fn join(&self) -> Result<()> {
        let handle = match &self.state {
            Ok(handle) => handle,
            Err(error) => return Err(error.clone()),
        };
        handle.wait().await?;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let Some(snapshot) = handle.snapshot() else {
                return Ok(());
            };
            match snapshot.state {
                FiberState::Failed => {
                    return Err(snapshot
                        .error
                        .map(Error::from)
                        .unwrap_or_else(|| Error::msg("plugin failed")));
                }
                FiberState::Loading | FiberState::Unloading => {}
                FiberState::Pending => {
                    let ready = match handle.context() {
                        Some(ctx) => handle
                            .dependencies()
                            .iter()
                            .all(|dep| ctx.get_shared(dep).is_some()),
                        None => false,
                    };
                    if !ready {
                        return Ok(());
                    }
                }
                FiberState::Active | FiberState::Disposed => return Ok(()),
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(Error::msg(format!(
                    "plugin {} did not settle within 10s",
                    self.name
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
    }

    /// Unload the plugin and run its disposers.
    pub async fn dispose(&self) {
        if let Ok(handle) = &self.state {
            if let Err(error) = handle.dispose().await {
                eprintln!("cordis-compat: disposing plugin {} failed: {error}", self.name);
            }
        }
    }
}

/// Legacy-compatible view of a kernel context.
#[derive(Clone)]
pub struct Context {
    inner: kernel::Context,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

fn into_kernel_disposer(disposer: Disposer) -> kernel::Disposer {
    let cell = Arc::new(std::sync::Mutex::new(Some(disposer)));
    Arc::new(move || {
        let taken = cell.lock().ok().and_then(|mut guard| guard.take());
        Box::pin(async move {
            if let Some(run) = taken {
                run().await;
            }
            Ok(())
        })
    })
}

impl Context {
    /// Create a fresh root runtime.
    pub fn new() -> Self {
        Self {
            inner: kernel::Context::new(),
        }
    }

    fn wrap(inner: kernel::Context) -> Self {
        Self { inner }
    }

    /// Provide a service (disposed with the owning fiber).
    pub async fn provide<V>(&self, name: &str, value: V) -> Result<()>
    where
        V: Any + Send + Sync,
    {
        self.inner.provide(name.to_string(), value).await?;
        Ok(())
    }

    /// Read a service, if provided and active.
    pub fn get<V: Any + Send + Sync>(&self, name: &str) -> Option<Arc<V>> {
        self.inner
            .get::<V>(name)
            .map(|service| service.into_inner())
    }

    /// Read a service or fail.
    pub fn require<V: Any + Send + Sync>(&self, name: &str) -> Result<Arc<V>> {
        self.get::<V>(name)
            .ok_or_else(|| Error::msg(format!("cannot get property \"{name}\" without inject")))
    }

    /// The structured logger.
    pub fn logger(&self) -> Logger {
        Logger {
            inner: self.inner.logger(),
        }
    }

    /// Run `setup` as a fiber-owned effect; its disposer runs on unload.
    pub async fn effect<Fut>(&self, label: impl Into<String>, setup: Fut) -> Result<EffectGuard>
    where
        Fut: Future<Output = Result<Option<Disposer>>> + Send + 'static,
    {
        let handle = self
            .inner
            .effect(label, move |_scope| async move {
                let disposer = setup
                    .await
                    .map_err(|e| kernel::CordisError::Effect(Arc::new(e)))?;
                Ok(disposer.map(into_kernel_disposer))
            })
            .await?;
        Ok(EffectGuard(handle))
    }

    /// Start a plugin; returns its fiber handle before activation settles.
    pub fn plugin(&self, plugin: Arc<dyn Plugin>, config: Option<Value>) -> FiberHandle {
        let name = plugin.name().into_owned();
        let mut builder = {
            let body = Arc::clone(&plugin);
            kernel::Plugin::builder(name.clone(), move |ctx, config| {
                let body = Arc::clone(&body);
                async move {
                    body.apply(Context::wrap(ctx), config).await.map_err(|e| {
                        kernel::CordisError::Plugin(body.name().into_owned(), Arc::new(e))
                    })?;
                    Ok(None)
                }
            })
        };
        for injection in plugin.inject() {
            builder = match injection.config {
                Some(config) => builder.inject_with(injection.service.into_owned(), config),
                None => builder.inject(injection.service.into_owned()),
            };
        }
        let validator = Arc::clone(&plugin);
        builder = builder.validate_config(move |raw| {
            validator
                .resolve_config(&raw)
                .map_err(|e| kernel::CordisError::InvalidConfig(e.to_string()))
        });
        let state = self
            .inner
            .start_plugin(builder.build(), config.unwrap_or(Value::Null))
            .map_err(Error::from);
        FiberHandle { state, name }
    }

    /// Register a listener disposed with the owning fiber.
    pub async fn on<L>(&self, name: &str, listener: L) -> Result<EffectGuard>
    where
        L: Fn(Context, Value, Next) -> BoxFuture<Result<Value>> + Send + Sync + 'static,
    {
        let listener: Listener = Arc::new(listener);
        let owner = self.clone();
        let callback: kernel::EventCallback = Arc::new(move |invocation| {
            let waterfall = IN_WATERFALL.try_with(|w| *w).unwrap_or(true);
            let payload = invocation.arg::<Value>(0).cloned().unwrap_or(Value::Null);
            let next = Next { invocation };
            let future = listener(owner.clone(), payload, next);
            Box::pin(async move {
                match future.await {
                    Ok(value) if waterfall || is_bailed(&value) => {
                        Ok(kernel::EventOutcome::Bail(Arc::new(value)))
                    }
                    Ok(_) => Ok(kernel::EventOutcome::Continue),
                    Err(error) => Err(kernel::CordisError::Listener(Arc::new(error))),
                }
            })
        });
        let handle = self
            .inner
            .on(name, callback, kernel::EventOptions::default())?;
        Ok(EffectGuard(handle))
    }

    /// Alias of [`Context::on`]; the kernel has one dispatch path.
    pub async fn on_sync<L>(&self, name: &str, listener: L) -> Result<EffectGuard>
    where
        L: Fn(Context, Value, Next) -> BoxFuture<Result<Value>> + Send + Sync + 'static,
    {
        self.on(name, listener).await
    }

    /// Fire-and-forget dispatch; listener failures are dropped.
    pub fn emit(&self, name: &str, payload: Value) {
        let inner = self.inner.clone();
        let name = name.to_string();
        let work = async move {
            let _ = inner.emit(&name, args_of(payload)).await;
        };
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(work);
            }
            Err(_) => futures::executor::block_on(work),
        }
    }

    /// Dispatch and await every listener; failures surface to the caller.
    pub async fn emit_sync(&self, name: &str, payload: Value) -> Result<()> {
        self.inner.emit(name, args_of(payload)).await?;
        Ok(())
    }

    /// Concurrent dispatch; aggregates listener failures.
    pub async fn parallel(&self, name: &str, payload: Value) -> Result<Value> {
        self.inner
            .parallel(name, args_of(payload.clone()))
            .await
            .map_err(|e| Error::msg(e.to_string()))?;
        Ok(payload)
    }

    /// Concurrent dispatch bounded by `timeout`; unfinished listeners are abandoned.
    pub async fn parallel_timeout(
        &self,
        name: &str,
        payload: Value,
        timeout: std::time::Duration,
    ) -> Result<Value> {
        match tokio::time::timeout(timeout, self.parallel(name, payload)).await {
            Ok(result) => result,
            Err(_) => Err(Error::msg(format!("parallel dispatch of {name} timed out"))),
        }
    }

    /// Sequential dispatch until the first truthy listener value (returned).
    pub async fn serial(&self, name: &str, payload: Value) -> Result<Value> {
        let outcome = IN_WATERFALL
            .scope(false, self.inner.serial(name, args_of(payload)))
            .await?;
        outcome_value(outcome)
    }

    /// First-bail-wins dispatch (alias of [`Context::serial`]).
    pub async fn bail(&self, name: &str, payload: Value) -> Result<Value> {
        self.serial(name, payload).await
    }

    /// Compose listeners around a fallback continuation.
    pub async fn waterfall<F>(&self, name: &str, payload: Value, fallback: F) -> Result<Value>
    where
        F: Fn(Value) -> BoxFuture<Result<Value>> + Send + Sync + 'static,
    {
        let fallback = Arc::new(fallback);
        let terminal: kernel::Next = Arc::new(move |args: kernel::EventArgs| {
            let fallback = Arc::clone(&fallback);
            let value = args
                .first()
                .and_then(|arg| arg.downcast_ref::<Value>())
                .cloned()
                .unwrap_or(Value::Null);
            Box::pin(async move {
                match fallback(value).await {
                    Ok(result) => Ok(kernel::EventOutcome::Bail(Arc::new(result))),
                    Err(error) => Err(kernel::CordisError::Listener(Arc::new(error))),
                }
            })
        });
        let outcome = IN_WATERFALL
            .scope(true, self.inner.waterfall(name, args_of(payload), terminal))
            .await?;
        outcome_value(outcome)
    }
}
