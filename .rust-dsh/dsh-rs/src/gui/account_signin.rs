//! Platform sign-in protocol, injectable effects, and the bounded loopback callback lifetime.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::api::services::BoxFuture;
use crate::gui::account::Account;
use crate::gui::desktop::{CommandRunner, SystemRunner};

/// Safe classifications; errors never contain request bodies or transport diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignInError {
    /// No HTTP response was received.
    NoResponse,
    /// An unsuccessful HTTP status was received.
    Network,
    /// Invalid origin, callback, envelope, or payload.
    Protocol,
    /// The local attempt deadline elapsed.
    Expired,
    /// A credential could not be read or committed.
    Storage,
}
impl SignInError {
    /// The TypeScript error code.
    pub fn code(self) -> &'static str {
        match self {
            Self::NoResponse => "no-response",
            Self::Network => "network",
            Self::Protocol => "protocol",
            Self::Expired => "expired",
            Self::Storage => "storage",
        }
    }
}
impl std::fmt::Display for SignInError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "account: {}", self.code())
    }
}
impl std::error::Error for SignInError {}

/// Validate an explicit loopback HTTP origin, retaining even an explicit default port.
pub fn login_origin(value: &str) -> Result<String, SignInError> {
    let lower = value.to_ascii_lowercase();
    let authority = lower.strip_prefix("http://").ok_or(SignInError::Protocol)?;
    let authority = authority.strip_suffix('/').unwrap_or(authority);
    let (host, port) = authority.rsplit_once(':').ok_or(SignInError::Protocol)?;
    if !["localhost", "127.0.0.1", "[::1]"].contains(&host)
        || port.is_empty()
        || !port.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(SignInError::Protocol);
    }
    let port = port
        .parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or(SignInError::Protocol)?;
    Ok(format!("http://{host}:{port}"))
}

/// Validate the deployment origin; production credentials only go to HTTPS origins.
pub fn platform_origin(value: &str) -> Result<String, SignInError> {
    let url = reqwest::Url::parse(value).map_err(|_| SignInError::Protocol)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(SignInError::Protocol);
    }
    Ok(url.origin().ascii_serialization())
}

/// Successful HTTP response before envelope validation. Bodies are never formatted in diagnostics.
pub struct PlatformResponse {
    /// HTTP status.
    pub status: u16,
    /// Bounded response body.
    pub body: Vec<u8>,
}
/// Platform transport injection; an error here means no Response was received.
pub trait Platform: Send + Sync {
    /// POST one fixed protocol method with the captured client headers.
    fn request(
        &self,
        method: &str,
        body: Value,
        headers: BTreeMap<String, String>,
    ) -> BoxFuture<Result<PlatformResponse, SignInError>>;
}
/// Browser effect injection; implementations must not log the URL or opener diagnostics.
pub trait Browser: Send + Sync {
    /// Open one validated Platform authorization URL.
    fn open(&self, url: String) -> BoxFuture<Result<(), SignInError>>;
}
/// Provider-owned grant storage boundary; neither tokens nor verifier enter the account view.
pub trait GrantStore: Send + Sync {
    /// Read the opaque provider grant.
    fn read(&self) -> BoxFuture<Result<Option<Value>, SignInError>>;
    /// Atomically persist the provider grant.
    fn write(&self, grant: Value) -> BoxFuture<Result<(), SignInError>>;
    /// Delete the provider grant.
    fn delete(&self) -> BoxFuture<Result<(), SignInError>>;
    /// Read or create the stable device UUID.
    fn device(&self) -> BoxFuture<Result<String, SignInError>>;
}

/// Decode the TS response envelope without returning server-authored error messages.
pub fn envelope(response: PlatformResponse) -> Result<Value, SignInError> {
    if !(200..300).contains(&response.status) {
        return Err(SignInError::Network);
    }
    if response.body.len() > 65_536 {
        return Err(SignInError::Protocol);
    }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| SignInError::Protocol)?;
    if value["code"] != 0 || value["data"]["biz_code"] != 0 || !value["data"].is_object() {
        return Err(SignInError::Protocol);
    }
    Ok(value["data"]
        .get("biz_data")
        .cloned()
        .unwrap_or(Value::Null))
}

/// Real platform transport with redirects refused and a per-request deadline.
pub struct HttpPlatform {
    origin: String,
    client: reqwest::Client,
}
impl HttpPlatform {
    /// Construct an HTTPS-only production transport.
    pub fn new(origin: &str) -> Result<Self, SignInError> {
        Ok(Self {
            origin: platform_origin(origin)?,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|_| SignInError::Protocol)?,
        })
    }
}
impl Platform for HttpPlatform {
    fn request(
        &self,
        method: &str,
        body: Value,
        headers: BTreeMap<String, String>,
    ) -> BoxFuture<Result<PlatformResponse, SignInError>> {
        let client = self.client.clone();
        let url = format!("{}/auth-api/v0/dsh/{method}", self.origin);
        Box::pin(async move {
            let mut request = client.post(url).json(&body);
            for (name, value) in headers {
                request = request.header(name, value);
            }
            let mut response = request.send().await.map_err(|_| SignInError::NoResponse)?;
            let status = response.status().as_u16();
            if (300..400).contains(&status) {
                return Err(SignInError::NoResponse);
            }
            if !(200..300).contains(&status) {
                return Err(SignInError::Network);
            }
            let mut body = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| SignInError::Protocol)? {
                if body.len() + chunk.len() > 65_536 {
                    return Err(SignInError::Protocol);
                }
                body.extend_from_slice(&chunk);
            }
            Ok(PlatformResponse { status, body })
        })
    }
}

/// OS opener using the desktop module's bounded, non-capturing command runner.
pub struct SystemBrowser;
impl Browser for SystemBrowser {
    fn open(&self, url: String) -> BoxFuture<Result<(), SignInError>> {
        Box::pin(async move {
            let program = if cfg!(target_os = "macos") {
                "open"
            } else if cfg!(target_os = "linux") {
                "xdg-open"
            } else {
                return Err(SignInError::Protocol);
            };
            SystemRunner::new(Duration::from_secs(30))
                .launch(program, vec![url])
                .await
                .map_err(|_| SignInError::Protocol)
        })
    }
}

/// A callback whose HTTP response remains pending until commit settles.
pub struct Callback {
    pub(crate) code: String,
    pub(crate) response: oneshot::Sender<Option<String>>,
}

/// Check cardinality, method, path and state before admitting any exchange.
pub fn callback_code(method: &str, target: &str, state: &str) -> Result<String, u16> {
    let url = reqwest::Url::parse(&format!("http://127.0.0.1{target}")).map_err(|_| 400u16)?;
    let codes: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "code")
        .map(|(_, v)| v.into_owned())
        .collect();
    let states: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "state")
        .map(|(_, v)| v.into_owned())
        .collect();
    // Hashing both values keeps the comparison independent of the first differing byte.
    let valid_state = states.len() == 1 && constant_time_state(&states[0], state);
    if method != "GET"
        || url.path() != "/oauth/callback"
        || !valid_state
        || codes.len() != 1
        || codes[0].is_empty()
    {
        return Err(400);
    }
    Ok(codes[0].clone())
}

fn constant_time_state(left: &str, right: &str) -> bool {
    let left = Sha256::digest(left.as_bytes());
    let right = Sha256::digest(right.as_bytes());
    let mut difference = 0u8;
    for i in 0..32 {
        // Volatile reads prevent the optimizer replacing this fixed-length scan
        // with an early-exit memcmp. Every digest byte is always visited.
        difference |=
            unsafe { std::ptr::read_volatile(&left[i]) ^ std::ptr::read_volatile(&right[i]) };
    }
    difference == 0
}

/// Callback route, also used directly by tests without opening a socket.
pub struct CallbackRoute {
    state: String,
    sender: mpsc::Sender<Callback>,
    claimed: std::sync::atomic::AtomicBool,
    account: Arc<Account>,
    id: String,
}
impl CallbackRoute {
    /// Admit exactly one valid callback while this attempt waits for its browser.
    pub async fn receive(
        &self,
        method: &str,
        target: &str,
    ) -> Result<oneshot::Receiver<Option<String>>, u16> {
        let code = callback_code(method, target, &self.state)?;
        if !self.account.waiting(&self.id).await
            || self.claimed.swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(410);
        }
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Callback { code, response })
            .await
            .map_err(|_| 410u16)?;
        Ok(receiver)
    }
}

/// Callback listener injection; production binds only the requested loopback address.
pub trait CallbackHost: Send + Sync {
    /// Serve the route until cancellation; readiness must precede initialization.
    fn listen(
        &self,
        origin: String,
        route: Arc<CallbackRoute>,
        stop: CancellationToken,
    ) -> BoxFuture<Result<(), SignInError>>;
}
/// Registers on the account; the existing gateway serves the callback.
pub struct LoopbackHost;
impl CallbackHost for LoopbackHost {
    fn listen(
        &self,
        origin: String,
        route: Arc<CallbackRoute>,
        stop: CancellationToken,
    ) -> BoxFuture<Result<(), SignInError>> {
        Box::pin(async move {
            let origin = login_origin(&origin)?;
            let account = route.account.clone();
            let id = route.id.clone();
            *account.callback.lock().await = Some((origin, route));
            tokio::spawn(async move {
                stop.cancelled().await;
                let mut slot = account.callback.lock().await;
                if slot.as_ref().is_some_and(|(_, route)| route.id == id) {
                    *slot = None;
                }
            });
            Ok(())
        })
    }
}

fn random_secret() -> Result<String, SignInError> {
    let mut bytes = [0u8; 32];
    // Read OS randomness directly: UUID version/variant bits are not random PKCE bytes.
    use std::io::Read;
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .is_err()
    {
        return Err(SignInError::Protocol);
    }
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}
fn browser_url(value: &Value, origin: &str, path: &str) -> Result<String, SignInError> {
    let url = reqwest::Url::parse(value.as_str().ok_or(SignInError::Protocol)?)
        .map_err(|_| SignInError::Protocol)?;
    if url.origin().ascii_serialization() != origin
        || url.path() != path
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(SignInError::Protocol);
    }
    Ok(url.to_string())
}

/// Captured arguments of one attempt.
pub struct SignInRequest {
    /// UI identity used for headers.
    pub client: Value,
    /// Explicit loopback HTTP origin.
    pub callback_origin: String,
    /// Initiating UI (`web` or `desktop`).
    pub login_source: String,
}

/// Run one attempt. The caller owns cancellation and the upper attempt deadline.
pub(crate) async fn run(
    account: Arc<Account>,
    id: &str,
    request: SignInRequest,
    stop: CancellationToken,
) -> Result<(), SignInError> {
    let verifier = random_secret()?;
    let state = random_secret()?;
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    let (sender, mut receiver) = mpsc::channel(1);
    let route = Arc::new(CallbackRoute {
        state: state.clone(),
        sender,
        claimed: false.into(),
        account: account.clone(),
        id: id.to_string(),
    });
    account
        .callbacks
        .listen(request.callback_origin.clone(), route, stop.clone())
        .await?;
    let locale = if request.client["locale"]
        .as_str()
        .unwrap_or("")
        .split(['-', '_'])
        .next()
        .is_some_and(|s| s.eq_ignore_ascii_case("zh"))
    {
        "zh_CN"
    } else {
        "en_US"
    };
    let headers = BTreeMap::from([
        ("x-client-bundle-id".into(), String::new()),
        ("x-client-platform".into(), account.client_platform.clone()),
        (
            "x-client-version".into(),
            request.client["version"].as_str().unwrap_or("").into(),
        ),
        ("x-client-locale".into(), locale.into()),
        (
            "x-client-timezone-offset".into(),
            request.client["timezoneOffsetSeconds"].to_string(),
        ),
    ]);
    let redirect_uri = format!("{}/oauth/callback", request.callback_origin);
    let init_request = account.platform.request("auth_init", json!({"code_challenge": challenge, "code_challenge_method": "S256", "state": state, "redirect_uri": redirect_uri, "locale": locale, "login_source": request.login_source}), headers.clone());
    // Before initialization returns there is no authorization id to cancel.
    let init = tokio::select! {
        biased;
        _ = stop.cancelled() => return Err(SignInError::Expired),
        result = tokio::time::timeout(Duration::from_millis(account.deadline(id).await.saturating_sub(now_ms())), init_request) => envelope(result.map_err(|_| SignInError::Expired)??)?,
    };
    let authorize_id = init["authorize_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(SignInError::Protocol)?
        .to_string();
    let outcome = async {
        let authorize_url = browser_url(&init["authorize_url"], &account.origin, "/dsh/authorize")?;
        let ttl = init["expires_in"]
            .as_f64()
            .filter(|s| s.is_finite() && *s > 0.0)
            .ok_or(SignInError::Protocol)?;
        let ttl = Duration::from_secs_f64(ttl.min(600.0));
        let expires = account
            .deadline(id)
            .await
            .min(now_ms() + ttl.as_millis() as u64);
        account
            .update(
                id,
                "waiting-browser",
                Some(json!({"authorizeUrl": authorize_url, "expiresAt": expires})),
            )
            .await;
        let exchange = async {
            account.browser.open(authorize_url).await?;
            let callback = tokio::time::timeout(
                Duration::from_millis(expires.saturating_sub(now_ms())),
                receiver.recv(),
            )
            .await
            .map_err(|_| SignInError::Expired)?
            .ok_or(SignInError::Protocol)?;
            account.update(id, "exchanging", None).await;
            let device_id = account.store.device().await?;
            let result = envelope(account.platform.request("auth_exchange", json!({"code": callback.code, "code_verifier": verifier, "redirect_uri": redirect_uri, "device_id": device_id, "device_model": device_model(), "os_version": os_version()}), headers.clone()).await?)?;
            let token = result["token"]
                .as_str()
                .filter(|s| !s.is_empty() && s.bytes().all(|b| (0x21..=0x7e).contains(&b)))
                .ok_or(SignInError::Protocol)?;
            let completion = browser_url(
                &result["authorized_url"],
                &account.origin,
                "/dsh/authorized",
            )?;
            let mut completion =
                reqwest::Url::parse(&completion).map_err(|_| SignInError::Protocol)?;
            let pairs: Vec<_> = completion
                .query_pairs()
                .filter(|(k, _)| k != "login_source")
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
            completion
                .query_pairs_mut()
                .clear()
                .extend_pairs(pairs)
                .append_pair("login_source", &request.login_source);
            if now_ms() >= expires {
                return Err(SignInError::Expired);
            }
            account
                .commit(
                    id,
                    json!({"version": 1, "token": token, "issuer": account.origin}),
                )
                .await?;
            let _ = callback.response.send(Some(completion.into()));
            Ok(())
        };
        tokio::time::timeout(
            Duration::from_millis(expires.saturating_sub(now_ms())),
            exchange,
        )
        .await
        .unwrap_or(Err(SignInError::Expired))
    };
    let result = tokio::select! {
        biased;
        _ = stop.cancelled() => Err(SignInError::Expired),
        result = tokio::time::timeout(Duration::from_millis(account.deadline(id).await.saturating_sub(now_ms())), outcome) => result.unwrap_or(Err(SignInError::Expired)),
    };
    if stop.is_cancelled() || result == Err(SignInError::Expired) {
        let _ = tokio::time::timeout(
            Duration::from_secs(30),
            account.platform.request(
                "auth_cancel",
                json!({"authorize_id":authorize_id,"code_verifier":verifier}),
                headers,
            ),
        )
        .await;
    }
    result
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn device_model() -> String {
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        std::env::consts::OS
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{platform}-{arch}")
}
fn os_version() -> String {
    #[cfg(unix)]
    {
        let mut info = std::mem::MaybeUninit::<libc::utsname>::uninit();
        // SAFETY: uname initializes the supplied structure on success.
        if unsafe { libc::uname(info.as_mut_ptr()) } == 0 {
            let info = unsafe { info.assume_init() };
            let release =
                unsafe { std::ffi::CStr::from_ptr(info.release.as_ptr()) }.to_string_lossy();
            let platform = if cfg!(target_os = "macos") {
                "darwin"
            } else {
                std::env::consts::OS
            };
            return format!("{platform} {release}");
        }
    }
    std::env::consts::OS.to_string()
}
