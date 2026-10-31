//! The HTTP and WebSocket halves of the GUI gateway.
//!
//! Unary routing is a pure function of `(method, path, headers, body)`, so every
//! transport rule — Host/Origin fence, authentication, envelope errors, content
//! types, path traversal — is testable without binding a port. The mux upgrade
//! at [`MUX_PATH`] is the one stateful route: it applies the same fence and
//! token check, then hands the socket to [`super::mux`]. A gateway admits one
//! mux socket at a time, as the TypeScript gateway does per browser Peer, and
//! answers a second upgrade with 409 until the first closes. [`serve`] moves
//! bytes and owns the upgrade.

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use cordis::Context;
use http::header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE};
use http::{HeaderMap, Method, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde_json::Value;

use super::mux::{Mux, MuxConfig};
use super::router::Router;
use super::wire::{codes, decode_request, Failure, RpcResult};

/// Path of the WebSocket carrying every logical stream.
pub const MUX_PATH: &str = "/api/remote.mux";

/// Path carrying unary RPC.
pub const API_PATH: &str = "/api";
/// Prefix reserved for the gateway; anything else is a static asset.
const API_PREFIX: &str = "/api/";

/// The raw-byte upload route, which the unary `/api/<endpoint>` form cannot carry.
const UPLOAD_PATH: &str = "/api/session/uploadFileBinary";
/// Cookie the browser client keeps its launch token in.
pub const TOKEN_COOKIE: &str = "dsh_token";

/// A response the gateway has already decided on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpReply {
    /// HTTP status code.
    pub status: u16,
    /// `Content-Type` to send.
    pub content_type: String,
    /// Response bytes.
    pub body: Vec<u8>,
}

impl HttpReply {
    fn json(status: u16, body: Vec<u8>) -> Self {
        HttpReply {
            status,
            content_type: "application/json; charset=utf-8".to_string(),
            body,
        }
    }

    pub(super) fn text(status: u16, body: impl Into<Vec<u8>>, content_type: &'static str) -> Self {
        HttpReply {
            status,
            content_type: content_type.to_string(),
            body: body.into(),
        }
    }

    /// A multipart answer: the binary parts, then the metadata that names them.
    fn multipart(
        status: u16,
        boundary: &str,
        attachments: &[super::wire::Attachment],
        metadata: Vec<u8>,
    ) -> Self {
        let mut body = Vec::new();
        for attachment in attachments {
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"{part}\"; filename=\"blob\"\r\nContent-Type: application/octet-stream\r\n\r\n",
                    part = attachment.part
                )
                .as_bytes(),
            );
            body.extend_from_slice(&attachment.bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(&metadata);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        HttpReply {
            status,
            content_type: format!("multipart/form-data; boundary={boundary}"),
            body,
        }
    }
}

/// How the gateway proves a caller is the one that launched it.
#[derive(Debug, Clone)]
pub struct Auth {
    /// Secret minted at launch, required on every request.
    pub token: String,
}

impl Auth {
    /// Whether these headers carry the token, from either carrier.
    fn admits(&self, headers: &HeaderMap) -> bool {
        if let Some(bearer) = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
        {
            if constant_time_eq(bearer.trim(), &self.token) {
                return true;
            }
        }
        if let Some(cookie) = headers.get(COOKIE).and_then(|value| value.to_str().ok()) {
            for pair in cookie.split(';') {
                let pair = pair.trim();
                if let Some(value) = pair
                    .strip_prefix(TOKEN_COOKIE)
                    .and_then(|rest| rest.strip_prefix('='))
                {
                    if constant_time_eq(value.trim(), &self.token) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Compare without leaking the length of the match through timing.
fn constant_time_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

/// Everything the gateway answers from.
pub struct Gateway {
    router: Router,
    auth: Auth,
    /// Directory served at `/`; `None` serves no assets.
    assets: Option<PathBuf>,
    /// The module-graph host that turns the static shell into a bootable page.
    boot: Option<super::boot::BootHost>,
    /// Extra `host[:port]` authorities accepted beyond loopback, e.g. the bind
    /// address when the gateway listens on a LAN interface.
    trusted_hosts: Vec<String>,
    /// Whether a mux socket is already open. The TS gateway lets one browser
    /// Peer own one socket; a second upgrade is refused with 409.
    socket_open: AtomicBool,
    /// Gap between WebSocket pings.
    heartbeat: Duration,
    /// Gap between checks for approval questions whose call stopped waiting.
    approval_sweep: Duration,
    workspace_poll: Duration,
}

impl Gateway {
    /// Build a gateway.
    pub fn new(router: Router, auth: Auth, assets: Option<PathBuf>) -> Self {
        Gateway {
            router,
            auth,
            assets,
            boot: None,
            trusted_hosts: Vec::new(),
            socket_open: AtomicBool::new(false),
            heartbeat: Duration::from_secs(5),
            approval_sweep: Duration::from_secs(1),
            workspace_poll: Duration::from_secs(1),
        }
    }

    /// Serve the client module graph: injects it into `index.html` and answers `/plugins`.
    pub fn with_boot(mut self, boot: super::boot::BootHost) -> Self {
        self.boot = Some(boot);
        self
    }

    /// Set the gap between WebSocket pings; a peer that misses two is dropped.
    pub fn with_heartbeat(mut self, heartbeat: Duration) -> Self {
        self.heartbeat = heartbeat;
        self
    }

    /// Set the gap between withdrawals of approval questions nobody awaits.
    /// How often `workspace/follow` re-reads which sessions each workspace owns.
    pub fn with_workspace_poll(mut self, poll: Duration) -> Self {
        self.workspace_poll = poll;
        self
    }

    pub fn with_approval_sweep(mut self, sweep: Duration) -> Self {
        self.approval_sweep = sweep;
        self
    }

    /// Why a WebSocket upgrade must be refused, if it must.
    pub fn upgrade_rejection(&self, headers: &HeaderMap) -> Option<(StatusCode, &'static str)> {
        if !self.trusts(headers) {
            return Some((StatusCode::FORBIDDEN, "forbidden"));
        }
        if !self.auth.admits(headers) {
            return Some((StatusCode::UNAUTHORIZED, "unauthorized"));
        }
        None
    }

    /// Accept requests addressed to these `host[:port]` authorities as well as
    /// loopback. Anything else is refused as a possible DNS-rebinding attempt.
    pub fn with_trusted_hosts(mut self, hosts: Vec<String>) -> Self {
        self.trusted_hosts = hosts;
        self
    }

    /// Whether the Host/Origin/Fetch-Metadata headers describe a request this
    /// gateway should answer at all. Applied before authentication, to every
    /// route including the WebSocket upgrade.
    pub fn trusts(&self, headers: &HeaderMap) -> bool {
        request_is_trusted(headers, &self.trusted_hosts)
    }

    /// The token cookie a launch response sets.
    pub fn token_cookie(&self) -> String {
        format!(
            "{TOKEN_COOKIE}={}; Path=/; HttpOnly; SameSite=Strict",
            self.auth.token
        )
    }

    /// Decide one request.
    pub async fn route(
        &self,
        ctx: &Context,
        method: &Method,
        path: &str,
        headers: &HeaderMap,
        body: &[u8],
    ) -> (HttpReply, HeaderMap) {
        self.route_with_query(ctx, method, path, None, headers, body)
            .await
    }

    /// Decide one request, given its URL query (the `/plugins` combo URLs
    /// carry the bundle list there).
    pub async fn route_with_query(
        &self,
        ctx: &Context,
        method: &Method,
        path: &str,
        query: Option<&str>,
        headers: &HeaderMap,
        body: &[u8],
    ) -> (HttpReply, HeaderMap) {
        let mut extra = HeaderMap::new();

        // OAuth is a cross-site browser navigation without the launch cookie.
        // Its fence is the registered loopback authority and one-use state, not
        // the API Origin/Fetch-Metadata fence or bearer authentication.
        if path == "/oauth/callback" {
            extra.insert(http::header::CACHE_CONTROL, "no-store".parse().unwrap());
            let route = if let Some(account) = &self.router.account {
                account.callback.lock().await.clone()
            } else {
                None
            };
            let status = if let Some((origin, route)) = route {
                let authority = origin.trim_start_matches("http://");
                if header_str(headers, "host")
                    .is_none_or(|host| !host.eq_ignore_ascii_case(authority))
                {
                    403
                } else {
                    let target = match query {
                        Some(query) => format!("{path}?{query}"),
                        None => path.to_string(),
                    };
                    match route.receive(method.as_str(), &target).await {
                        Err(status) => status,
                        Ok(receiver) => match receiver.await {
                            Ok(Some(url)) => {
                                if let Ok(url) = url.parse() {
                                    extra.insert(http::header::LOCATION, url);
                                }
                                302
                            }
                            _ => 204,
                        },
                    }
                }
            } else {
                410
            };
            return (
                HttpReply::text(status, Vec::new(), "text/plain; charset=utf-8"),
                extra,
            );
        }

        if !self.trusts(headers) {
            let failure = Failure::at(
                codes::FORBIDDEN,
                "the request's Host or Origin is not this gateway",
                path,
            );
            return (
                HttpReply::json(403, body_of(RpcResult::failed(failure))),
                extra,
            );
        }

        // The launch page hands the browser its token; it is the only
        // authenticated route that may answer without one.
        let unauthenticated = !self.auth.admits(headers);
        if unauthenticated && (path == "/" || path == "/index.html") {
            {
                let reply = match self.asset("index.html") {
                    Some(reply) => reply,
                    None => HttpReply::text(
                        404,
                        "no GUI bundle is installed",
                        "text/plain; charset=utf-8",
                    ),
                };
                if let Ok(value) = self.token_cookie().parse() {
                    extra.insert(SET_COOKIE, value);
                }
                return (reply, extra);
            }
        }

        if unauthenticated {
            let failure = Failure::at(codes::UNAUTHORIZED, "a launch token is required", path);
            return (
                HttpReply::json(401, body_of(RpcResult::failed(failure))),
                extra,
            );
        }

        if path == super::export::SESSION_LOG_EXPORT_PATH {
            return super::export::respond(&self.router, ctx, method, query).await;
        }
        if path == UPLOAD_PATH {
            return (
                self.raw_upload(ctx, method, query, headers, body).await,
                extra,
            );
        }
        if path == API_PATH || path.starts_with(API_PREFIX) {
            return (self.api(ctx, method, path, body).await, extra);
        }
        if method != Method::GET && method != Method::HEAD {
            return (
                HttpReply::text(405, "method not allowed", "text/plain; charset=utf-8"),
                extra,
            );
        }
        if path == "/open-in-app/apps" && method == Method::GET {
            return (
                HttpReply::json(200, super::catalog::open_in_app_apps()),
                extra,
            );
        }
        if let Some(boot) = &self.boot {
            if path.starts_with("/plugins/") {
                return self.plugin_reply(boot, method, path, query, extra);
            }
        }
        let name = if path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };
        let reply = match self.asset(name) {
            Some(reply) => reply,
            None => HttpReply::text(404, "not found", "text/plain; charset=utf-8"),
        };
        // HEAD answers with the same headers as GET and no body, as HTTP requires.
        let reply = if method == Method::HEAD {
            HttpReply {
                body: Vec::new(),
                ..reply
            }
        } else {
            reply
        };
        (reply, extra)
    }

    /// Readiness: 200 when a chat turn could start, 503 naming each failed check.
    ///
    /// Behind the fence and token like every other route, because the check
    /// names tell a caller which services and providers are loaded.
    fn health(&self, ctx: &Context, method: &Method) -> HttpReply {
        use crate::api::services::{AgentRegistryService, LlmService};
        if method != Method::GET {
            return HttpReply::text(405, "method not allowed", "text/plain; charset=utf-8");
        }
        let provider = &self.router.config.default_model.provider;
        let llm = ctx.get::<LlmService>(crate::api::LLM_SERVICE);
        let checks = serde_json::json!({
            "agents": ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE).is_some(),
            "llm": llm.is_some(),
            "defaultProvider": llm.is_some_and(|llm| llm.has_provider(provider)),
        });
        let ready = checks
            .as_object()
            .is_some_and(|c| c.values().all(|v| v == true));
        let body = serde_json::json!({
            "status": if ready { "ready" } else { "unavailable" },
            "checks": checks,
            "muxSocketOpen": self.socket_open.load(Ordering::Acquire),
        });
        HttpReply::json(
            if ready { 200 } else { 503 },
            serde_json::to_vec(&body).unwrap_or_default(),
        )
    }

    /// The largest request body this gateway reads: the biggest file upload as
    /// base64 plus its JSON wrapper. Anything longer is refused with 413.
    pub fn max_body_bytes(&self) -> usize {
        self.router
            .uploads()
            .max_body_bytes()
            .max(self.router.images().max_body_bytes())
    }

    /// `POST /api/session/uploadFileBinary?sessionId=…&name=…`: the body is the
    /// file. The browser sends a Blob here, not base64 through the RPC route.
    /// Request faults answer with a plain status; everything after them answers
    /// 200 with the `{ok, value | error}` JSON, as the TypeScript route does.
    async fn raw_upload(
        &self,
        ctx: &Context,
        method: &Method,
        query: Option<&str>,
        headers: &HeaderMap,
        body: &[u8],
    ) -> HttpReply {
        const TEXT: &str = "text/plain; charset=utf-8";
        // The TypeScript route is registered for POST alone, so any other
        // method reaches no handler and the gateway answers 404.
        if method != Method::POST {
            return HttpReply::text(404, "not found", TEXT);
        }
        let media_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(|value| value.trim().to_ascii_lowercase());
        if media_type.as_deref() != Some("application/octet-stream") {
            return HttpReply::text(415, "content type must be application/octet-stream", TEXT);
        }
        let mut session_id = None;
        let mut name = None;
        for (key, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            match key.as_ref() {
                "sessionId" => session_id = Some(value.into_owned()),
                "name" => name = Some(value.into_owned()),
                _ => {}
            }
        }
        let Some(session_id) = session_id.filter(|id| !id.is_empty()) else {
            return HttpReply::text(400, "sessionId is required", TEXT);
        };
        let result =
            match super::uploads::upload_stream(&self.router, ctx, &session_id, name, body).await {
                Ok(value) => RpcResult::ok(value),
                Err(failure) => RpcResult::failed(failure),
            };
        let json = serde_json::to_vec(&result).unwrap_or_default();
        HttpReply::json(200, json)
    }

    async fn api(&self, ctx: &Context, method: &Method, path: &str, body: &[u8]) -> HttpReply {
        if path == "/api/health" {
            return self.health(ctx, method);
        }
        if path == "/api/endpoints" {
            return HttpReply::json(
                200,
                serde_json::to_vec(&serde_json::json!({
                    "endpoints": Router::endpoints(),
                    "streams": super::mux::STREAM_ENDPOINTS,
                }))
                .unwrap_or_default(),
            );
        }
        // The shipped client posts to `/api/<endpoint>`; the bare `/api` form is
        // this gateway's earlier single-route spelling and stays accepted.
        let url_endpoint = match path.strip_prefix(API_PREFIX) {
            Some(endpoint) if valid_endpoint_path(endpoint) => Some(endpoint),
            Some(_) => {
                return HttpReply::json(
                    404,
                    body_of(RpcResult::failed(Failure::at(
                        codes::LOOKUP_NOT_FOUND,
                        format!("no route is registered for {path:?}"),
                        path,
                    ))),
                )
            }
            None => None,
        };
        if method != Method::POST {
            return HttpReply::json(
                405,
                body_of(RpcResult::failed(Failure::at(
                    codes::PROTOCOL,
                    "unary RPC is POST /api/<endpoint>",
                    path,
                ))),
            );
        }
        let request = match decode_request(body) {
            Ok(request) => request,
            Err(malformed) => {
                // Echo the correlation id when the envelope had one, so the
                // client can still fail the call it is waiting on.
                let response = match malformed.rpc_id {
                    Some(rpc_id) => super::wire::ServerResponse::new(
                        rpc_id,
                        RpcResult::failed(Failure::at(codes::PROTOCOL, malformed.reason, path)),
                    ),
                    None => super::wire::ServerResponse::new(
                        String::new(),
                        RpcResult::failed(Failure::at(codes::PROTOCOL, malformed.reason, path)),
                    ),
                };
                return HttpReply::json(400, response.to_json());
            }
        };
        if let Some(endpoint) = url_endpoint {
            if request.method != endpoint {
                let failure = Failure::at(
                    codes::BAD_REQUEST,
                    format!(
                        "method {:?} does not match endpoint {endpoint:?}",
                        request.method
                    ),
                    path,
                );
                return HttpReply::json(
                    200,
                    super::wire::ServerResponse::new(request.rpc_id, RpcResult::failed(failure))
                        .to_json(),
                );
            }
        }
        let result = self.router.dispatch(ctx, &request).await;
        let attachments = result.attachments().to_vec();
        if attachments.is_empty() {
            return HttpReply::json(
                200,
                super::wire::ServerResponse::new(request.rpc_id, result).to_json(),
            );
        }
        // A result with binary parts is answered as multipart: the parts first,
        // then the metadata that names them, as the TypeScript gateway does.
        let metadata =
            super::wire::ServerResponse::with_attachments(request.rpc_id, result).to_json();
        HttpReply::multipart(200, &multipart_boundary(), &attachments, metadata)
    }

    /// Read one asset under the asset root, refusing anything that escapes it.
    fn asset(&self, name: &str) -> Option<HttpReply> {
        let root = self.assets.as_ref()?;
        let relative = Path::new(name);
        // A traversal attempt is a missing asset, never a file outside the root.
        if relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return None;
        }
        let path = root.join(relative);
        let bytes = std::fs::read(&path).ok()?;
        let bytes = match (&self.boot, name) {
            (Some(boot), "index.html") => boot
                .render_index(&String::from_utf8_lossy(&bytes))
                .into_bytes(),
            _ => bytes,
        };
        Some(HttpReply::text(200, bytes, content_type_for(&path)))
    }

    /// Answer a `/plugins` request from the boot host; an unadvertised URL is 404.
    fn plugin_reply(
        &self,
        boot: &super::boot::BootHost,
        method: &Method,
        path: &str,
        query: Option<&str>,
        mut extra: HeaderMap,
    ) -> (HttpReply, HeaderMap) {
        if method != Method::GET && method != Method::HEAD {
            return (
                HttpReply::text(405, "method not allowed", "text/plain; charset=utf-8"),
                extra,
            );
        }
        let Some(served) = boot.serve(path, query) else {
            return (
                HttpReply::text(404, "not found", "text/plain; charset=utf-8"),
                extra,
            );
        };
        if let Ok(value) = super::boot::BootHost::cache_control().parse() {
            extra.insert(http::header::CACHE_CONTROL, value);
        }
        let body = if method == Method::HEAD {
            Vec::new()
        } else {
            served.body
        };
        (
            HttpReply::text(served.status, body, served.content_type),
            extra,
        )
    }
}

/// The boundary of a multipart answer.
fn multipart_boundary() -> String {
    let unique = crate::plugins::unique_id();
    format!("----formdata-undici-{unique}")
}

fn body_of(result: RpcResult) -> Vec<u8> {
    super::wire::ServerResponse::new(String::new(), result).to_json()
}

/// Content type by extension, covering the bundle the GUI ships.
fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ico" => "image/x-icon",
        "map" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Serve the gateway on an already-bound listener until the task is dropped.
///
/// Returns the bound address so a caller that asked for port 0 can report it.
pub async fn serve(
    gateway: Arc<Gateway>,
    ctx: Context,
    listener: tokio::net::TcpListener,
) -> std::io::Result<()> {
    loop {
        let (stream, _peer) = listener.accept().await?;
        let gateway = Arc::clone(&gateway);
        let ctx = ctx.clone();
        tokio::spawn(async move {
            let service = service_fn(move |request: Request<Incoming>| {
                let gateway = Arc::clone(&gateway);
                let ctx = ctx.clone();
                async move {
                    if request.uri().path() == MUX_PATH && is_websocket_upgrade(request.headers()) {
                        return Ok(upgrade(gateway, ctx, request));
                    }
                    let (parts, body) = request.into_parts();
                    let limit = gateway.max_body_bytes();
                    let bytes = match BodyExt::collect(http_body_util::Limited::new(body, limit))
                        .await
                    {
                        Ok(collected) => collected.to_bytes(),
                        Err(error)
                            if error
                                .downcast_ref::<http_body_util::LengthLimitError>()
                                .is_some() =>
                        {
                            let failure = Failure::at(
                                codes::BAD_REQUEST,
                                format!("the request body is longer than the {limit} byte limit"),
                                parts.uri.path(),
                            );
                            return Ok::<_, std::convert::Infallible>(
                                Response::builder()
                                    .status(StatusCode::PAYLOAD_TOO_LARGE)
                                    .header(CONTENT_TYPE, "application/json; charset=utf-8")
                                    .body(Full::new(Bytes::from(body_of(RpcResult::failed(
                                        failure,
                                    )))))
                                    .unwrap_or_default(),
                            );
                        }
                        Err(_) => {
                            return Ok::<_, std::convert::Infallible>(
                                Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Full::new(Bytes::new()))
                                    .unwrap_or_default(),
                            )
                        }
                    };
                    let path = parts.uri.path().to_string();
                    let query = parts.uri.query().map(str::to_string);
                    let method = parts.method.clone();
                    let started = std::time::Instant::now();
                    let (reply, extra) = gateway
                        .route_with_query(
                            &ctx,
                            &method,
                            &path,
                            query.as_deref(),
                            &parts.headers,
                            &bytes,
                        )
                        .await;
                    // One line per request: without it a failing client is a
                    // silent port with no evidence of where it stopped.
                    let level = if reply.status >= 400 {
                        cordis::logger::LogLevel::Warn
                    } else {
                        cordis::logger::LogLevel::Debug
                    };
                    ctx.logger().log_event(
                        level,
                        "gui".to_string(),
                        None,
                        format!(
                            "{} {path} -> {} ({} ms)",
                            method.as_str(),
                            reply.status,
                            started.elapsed().as_millis()
                        ),
                    );
                    let mut builder = Response::builder()
                        .status(StatusCode::from_u16(reply.status).unwrap_or(StatusCode::OK))
                        .header(CONTENT_TYPE, reply.content_type);
                    for (name, value) in extra.iter() {
                        builder = builder.header(name, value);
                    }
                    Ok(builder
                        .body(Full::new(Bytes::from(reply.body)))
                        .unwrap_or_default())
                }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .with_upgrades()
                .await;
        });
    }
}

/// Read a JSON body into a value, for tests and small probes.
#[allow(dead_code)]
fn parse_json(body: &[u8]) -> Option<Value> {
    serde_json::from_slice(body).ok()
}

/// The Host/Origin fence the TypeScript gateway applies to every request.
///
/// A rebound page reaches this socket carrying the attacker's domain in `Host`,
/// so `Host` must be loopback or an explicitly trusted authority. A browser
/// that labels the request cross-site, or attaches an `Origin` other than this
/// authority, is refused; an absent `Origin` is fine because `Host` already
/// bound the request.
fn request_is_trusted(headers: &HeaderMap, trusted_hosts: &[String]) -> bool {
    let Some(host) = header_str(headers, "host") else {
        return false;
    };
    let Some((hostname, port)) = split_authority(host, 80) else {
        return false;
    };
    let loopback = hostname == "localhost" || hostname == "::1" || hostname.starts_with("127.");
    let listed = trusted_hosts
        .iter()
        .any(|entry| split_authority(entry, 80).is_some_and(|(h, p)| h == hostname && p == port));
    if !loopback && !listed {
        return false;
    }
    if header_str(headers, "sec-fetch-site") == Some("cross-site") {
        return false;
    }
    match header_str(headers, "origin") {
        None => true,
        Some(origin) => origin_matches_host(origin, host),
    }
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Lower-cased hostname and port of `host[:port]`, with `default_port` when absent.
fn split_authority(authority: &str, default_port: u16) -> Option<(String, u16)> {
    let authority = authority.trim();
    if authority.is_empty() || authority.contains('/') || authority.contains('@') {
        return None;
    }
    let (hostname, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (inside, after) = rest.split_once(']')?;
        let port = match after.strip_prefix(':') {
            Some(digits) => digits.parse().ok()?,
            None if after.is_empty() => default_port,
            None => return None,
        };
        (inside.to_string(), port)
    } else if let Some((name, digits)) = authority.rsplit_once(':') {
        (name.to_string(), digits.parse().ok()?)
    } else {
        (authority.to_string(), default_port)
    };
    if hostname.is_empty() {
        return None;
    }
    Some((hostname.to_ascii_lowercase(), port))
}

/// Whether `origin` is exactly the `http(s)` authority named by `host`.
fn origin_matches_host(origin: &str, host: &str) -> bool {
    let (scheme_port, rest) = if let Some(rest) = origin.strip_prefix("http://") {
        (80, rest)
    } else if let Some(rest) = origin.strip_prefix("https://") {
        (443, rest)
    } else {
        return false; // "null", file:, and unknown schemes are opaque origins
    };
    let authority = rest.trim_end_matches('/');
    match (
        split_authority(authority, scheme_port),
        split_authority(host, scheme_port),
    ) {
        (Some(from_origin), Some(from_host)) => from_origin == from_host,
        _ => false,
    }
}

fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    let has = |name: &str, token: &str| {
        header_str(headers, name).is_some_and(|value| {
            value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case(token))
        })
    };
    has("connection", "upgrade") && has("upgrade", "websocket")
}

/// Answer a mux upgrade: refuse it with a plain status, or switch protocols and
/// hand the socket to [`pump`].
fn upgrade(
    gateway: Arc<Gateway>,
    ctx: Context,
    mut request: Request<Incoming>,
) -> Response<Full<Bytes>> {
    let refuse = |status: StatusCode, text: &'static str| {
        Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from_static(text.as_bytes())))
            .unwrap_or_default()
    };
    if let Some((status, text)) = gateway.upgrade_rejection(request.headers()) {
        return refuse(status, text);
    }
    let Some(key) = request
        .headers()
        .get("sec-websocket-key")
        .filter(|key| is_valid_websocket_key(key.as_bytes()))
        .cloned()
    else {
        return refuse(
            StatusCode::BAD_REQUEST,
            "missing or malformed Sec-WebSocket-Key",
        );
    };
    if header_str(request.headers(), "sec-websocket-version") != Some("13") {
        return refuse(StatusCode::BAD_REQUEST, "unsupported WebSocket version");
    }
    if gateway.socket_open.swap(true, Ordering::AcqRel) {
        return refuse(StatusCode::CONFLICT, "conflict");
    }
    let accept = tokio_tungstenite::tungstenite::handshake::derive_accept_key(key.as_bytes());
    let upgraded = hyper::upgrade::on(&mut request);
    let owner = Arc::clone(&gateway);
    tokio::spawn(async move {
        if let Ok(io) = upgraded.await {
            pump(gateway, ctx, TokioIo::new(io)).await;
        }
        owner.socket_open.store(false, Ordering::Release);
    });
    Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("connection", "Upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-accept", accept)
        .body(Full::new(Bytes::new()))
        .unwrap_or_default()
}

/// A command for the socket's single writer task.
enum Control {
    Ping,
    /// Send this close frame, then stop.
    Close(super::mux::MuxClose),
    /// The peer sent Close: flush tungstenite's queued reply, then stop.
    FlushClose,
}

/// Longest the closing handshake may take before the socket is dropped anyway.
const CLOSE_GRACE: Duration = Duration::from_secs(2);

/// Move frames between the socket and a [`Mux`] until either side stops.
///
/// Reading, writing and the heartbeat run apart: one task owns the sink and
/// serialises every write, while this task keeps reading and counting missed
/// pings. A peer that stops reading therefore blocks only the writer; its
/// cancels still land and the heartbeat still drops it.
async fn pump<S>(gateway: Arc<Gateway>, ctx: Context, io: S)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame, Role};
    use tokio_tungstenite::tungstenite::Message;

    let socket = tokio_tungstenite::WebSocketStream::from_raw_socket(io, Role::Server, None).await;
    let (mut sink, mut source) = socket.split();
    let (out, mut outbound) = tokio::sync::mpsc::channel::<String>(64);
    let (control, mut commands) = tokio::sync::mpsc::unbounded_channel::<Control>();
    // Set by the writer when a write had to wait for the peer and then finished:
    // a congested link can hold the Pong behind data, so that counts as life.
    let progressed = Arc::new(AtomicBool::new(false));

    let writer_progress = Arc::clone(&progressed);
    let writer = tokio::spawn(async move {
        loop {
            let message = tokio::select! {
                biased;
                command = commands.recv() => match command {
                    Some(Control::Ping) => Message::Ping(Bytes::new()),
                    Some(Control::Close(refusal)) => {
                        let frame = CloseFrame {
                            code: CloseCode::from(refusal.code),
                            reason: refusal.reason.into(),
                        };
                        let _ = tokio::time::timeout(
                            CLOSE_GRACE,
                            sink.send(Message::Close(Some(frame))),
                        )
                        .await;
                        return;
                    }
                    Some(Control::FlushClose) => {
                        let _ = tokio::time::timeout(CLOSE_GRACE, sink.flush()).await;
                        return;
                    }
                    None => return,
                },
                frame = outbound.recv() => match frame {
                    Some(text) => Message::text(text),
                    None => return,
                },
            };
            let mut write = std::pin::pin!(sink.send(message));
            match futures_util::poll!(write.as_mut()) {
                std::task::Poll::Ready(Ok(())) => {}
                std::task::Poll::Ready(Err(_)) => return,
                std::task::Poll::Pending => {
                    if write.await.is_err() {
                        return;
                    }
                    writer_progress.store(true, Ordering::Release);
                }
            }
        }
    });

    let mux = Mux::new(
        ctx,
        MuxConfig {
            home: std::env::var("HOME").unwrap_or_default(),
            max_page_events: gateway.router.config.max_page_events,
            agent_presets: gateway.router.config.agent_presets.clone(),
            image_limits: gateway.router.images().limits().clone(),
            events: gateway.router.events(),
            approval_sweep: gateway.approval_sweep,
            workspaces: gateway.router.workspaces(),
            workspace_poll: gateway.workspace_poll,
            default_model: gateway.router.config.default_model.clone(),
            terminal: gateway.router.terminal(),
            product_analytics: gateway.router.product_analytics(),
            settings: Arc::clone(&gateway.router.settings),
        },
        out,
    );
    let mut ticker = tokio::time::interval(gateway.heartbeat);
    // After a long stall, wait a full interval rather than firing the missed
    // ticks back to back, which would spend the allowance in one burst.
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    ticker.tick().await; // the first tick is immediate
    let mut unanswered = 0u32;
    let mut graceful = false;
    loop {
        tokio::select! {
            message = source.next() => match message {
                Some(Ok(Message::Text(text))) => {
                    unanswered = 0;
                    if let Err(refusal) = mux.receive(text.as_str()) {
                        let _ = control.send(Control::Close(refusal));
                        graceful = true;
                        break;
                    }
                }
                Some(Ok(Message::Binary(_))) => {
                    let _ = control.send(Control::Close(super::mux::MuxClose {
                        code: 1003,
                        reason: "text messages required",
                    }));
                    graceful = true;
                    break;
                }
                Some(Ok(Message::Pong(_) | Message::Ping(_))) => unanswered = 0,
                Some(Ok(Message::Close(_))) => {
                    let _ = control.send(Control::FlushClose);
                    graceful = true;
                    break;
                }
                None | Some(Err(_)) => break,
                Some(Ok(Message::Frame(_))) => {}
            },
            _ = ticker.tick() => {
                if progressed.swap(false, Ordering::AcqRel) {
                    unanswered = 0;
                }
                if unanswered >= 2 {
                    break;
                }
                unanswered += 1;
                let _ = control.send(Control::Ping);
            }
        }
    }
    mux.shutdown();
    if graceful {
        // The writer finishes the close handshake; do not cut it short.
        let _ = tokio::time::timeout(CLOSE_GRACE, writer).await;
    } else {
        writer.abort();
    }
}

/// RFC 6455 §4.1: the key is the base64 of exactly 16 bytes, i.e. 22 base64
/// characters, two of padding, and zero trailing bits in the last character.
fn is_valid_websocket_key(key: &[u8]) -> bool {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    key.len() == 24
        && key.ends_with(b"==")
        && key[..22].iter().all(|byte| ALPHABET.contains(byte))
        // 16 bytes leave 4 spare bits, so the final sextet's low 4 bits are zero.
        && b"AQgw".contains(&key[21])
}

/// TS `endpointFromPath`: every segment is non-empty, not `.`/`..`, and made of
/// `[A-Za-z0-9_$.-]`.
fn valid_endpoint_path(endpoint: &str) -> bool {
    endpoint.split('/').all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'.' | b'-'))
    })
}
