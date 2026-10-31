//! Opening workspace paths on the desktop that runs this host.
//!
//! `session/openWorkspacePath` hands a path to the operating system (the
//! default application, a chosen application, or the file manager), and
//! `session/workspacePathApplications` lists the applications registered for
//! it. Both refuse a path the host cannot verify: after lexical resolution the
//! path must equal its canonical form, so a symlinked spelling such as
//! `/tmp` is refused where `/private/tmp` is accepted, exactly as the
//! TypeScript host does.
//!
//! Supported desktops are macOS (`open`, `osascript`) and graphical Linux
//! (`xdg-open`). Linux has no application query here, so its list is empty.
//! WSL and Windows are not supported: they report that opening is
//! unavailable instead of opening the wrong thing.
//!
//! Verification checks a pathname and the opener resolves that pathname again,
//! as in the TypeScript host; a process that can rewrite the directories on
//! the path between the two steps can redirect the open. Neither `open` nor
//! `xdg-open` accepts a verified handle, so this is a limit of the platform
//! openers, not of the check.
//!
//! Every native command goes through a [`CommandRunner`], so tests never start
//! a real application, and each command has a deadline: a hung `osascript` must
//! not hang the request. A command is killed when its request is dropped.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Value};

use super::router::{boundary, check_descriptor, Router};
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::BoxFuture;

/// The AppKit query macOS runs for the applications registered for a file. The
/// path arrives as an argument, never as script source.
const MAC_APPLICATIONS: &str = r#"
ObjC.import('AppKit');
function run(argv) {
  var workspace = $.NSWorkspace.sharedWorkspace;
  var file = $.NSURL.fileURLWithPath(argv[0]);
  var preferred = workspace.URLForApplicationToOpenURL(file);
  var preferredPath = preferred.isNil() ? null : ObjC.unwrap(preferred.path);
  var urls = workspace.URLsForApplicationsToOpenURL(file);
  var apps = [];
  for (var i = 0; i < urls.count; i++) {
    var url = urls.objectAtIndex(i);
    var path = ObjC.unwrap(url.path);
    var image = null;
    if (argv[1] === 'icons') {
      var icon = workspace.iconForFile(path);
      var thumbnail = $.NSImage.alloc.initWithSize($.NSMakeSize(32, 32));
      thumbnail.lockFocus;
      icon.drawInRectFromRectOperationFraction($.NSMakeRect(0, 0, 32, 32), $.NSZeroRect, $.NSCompositingOperationSourceOver, 1);
      thumbnail.unlockFocus;
      var bitmap = $.NSBitmapImageRep.imageRepWithData(thumbnail.TIFFRepresentation);
      var png = bitmap.representationUsingTypeProperties($.NSBitmapImageFileTypePNG, $({}));
      image = png.isNil() ? null : 'data:image/png;base64,' + ObjC.unwrap(png.base64EncodedStringWithOptions(0));
    }
    var bundle = $.NSBundle.bundleWithURL(url);
    var bundleId = bundle.isNil() || bundle.bundleIdentifier.isNil() ? null : ObjC.unwrap(bundle.bundleIdentifier);
    var version = bundle.isNil() ? null : bundle.objectForInfoDictionaryKey('CFBundleShortVersionString');
    apps.push({
      id: path,
      name: ObjC.unwrap($.NSFileManager.defaultManager.displayNameAtPath(path)),
      default: path === preferredPath,
      icon: image,
      bundle: bundleId,
      version: version === null || version.isNil() ? null : String(ObjC.unwrap(version))
    });
  }
  return JSON.stringify(apps);
}"#;

/// Documents a browser renders, which open in the default browser.
const BROWSER_DOCUMENTS: &[&str] = &["html", "htm", "xhtml", "svg"];

/// The desktop this host runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS: `open` and `osascript`.
    Mac,
    /// Linux: `xdg-open`.
    Linux,
    /// Anything else; nothing can be opened.
    Other,
}

impl Platform {
    /// The platform this process runs on. WSL is `Other`: its desktop is
    /// Windows, which the openers here do not reach.
    pub fn detect() -> Self {
        let set = |name: &str| std::env::var(name).is_ok_and(|value| !value.is_empty());
        if cfg!(target_os = "macos") {
            Platform::Mac
        } else if cfg!(target_os = "linux") && !(set("WSL_DISTRO_NAME") || set("WSL_INTEROP")) {
            Platform::Linux
        } else {
            Platform::Other
        }
    }
}

/// Runs a native command and returns its standard output.
///
/// The error is the cause as a message: it reaches the host log, never the
/// client, which only learns that the open failed.
pub trait CommandRunner: Send + Sync {
    /// Run a query: `program` with `args` (no shell) to completion, returning
    /// what it printed.
    fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, String>>;

    /// Launch an opener and wait for it to exit, discarding its output. The
    /// output is not captured because an opener such as `xdg-open` can leave
    /// the application it started holding the pipes, which would keep a
    /// capturing reader waiting for as long as that application runs.
    fn launch(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<(), String>>;
}

/// Runs real processes, killing one that outlives `timeout` or whose request
/// is dropped.
pub struct SystemRunner {
    timeout: Duration,
}

impl SystemRunner {
    /// A runner whose commands may take at most `timeout`.
    pub fn new(timeout: Duration) -> Self {
        SystemRunner { timeout }
    }
}

impl CommandRunner for SystemRunner {
    fn launch(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<(), String>> {
        let program = program.to_string();
        let timeout = self.timeout;
        Box::pin(async move {
            let mut child = tokio::process::Command::new(&program)
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| format!("{program} could not start: {e}"))?;
            let status = tokio::time::timeout(timeout, child.wait())
                .await
                .map_err(|_| format!("{program} did not finish within {timeout:?}"))?
                .map_err(|e| format!("{program} could not be awaited: {e}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("{program} exited with {status}"))
            }
        })
    }

    fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, String>> {
        let program = program.to_string();
        let timeout = self.timeout;
        Box::pin(async move {
            let mut command = tokio::process::Command::new(&program);
            command
                .args(&args)
                .stdin(std::process::Stdio::null())
                .kill_on_drop(true);
            let output = tokio::time::timeout(timeout, command.output())
                .await
                .map_err(|_| format!("{program} did not finish within {timeout:?}"))?
                .map_err(|e| format!("{program} could not start: {e}"))?;
            if !output.status.success() {
                return Err(format!(
                    "{program} exited with {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
            String::from_utf8(output.stdout)
                .map_err(|e| format!("{program} printed non-UTF-8: {e}"))
        })
    }
}

/// The desktop operations behind the open endpoints.
pub struct Desktop {
    platform: Platform,
    /// Whether the session reaches a display (Linux only); macOS always does.
    graphical: bool,
    /// `$BROWSER` as one executable name, the way the TypeScript host reads it:
    /// a value with arguments, `%s`, or colon-separated alternatives names no
    /// executable, so opening a web document fails.
    browser: Option<String>,
    /// An operator's answer overriding detection.
    native_open: Option<bool>,
    runner: Arc<dyn CommandRunner>,
}

impl Desktop {
    /// A desktop with explicit facts, for tests.
    pub fn new(
        platform: Platform,
        graphical: bool,
        browser: Option<String>,
        native_open: Option<bool>,
        runner: Arc<dyn CommandRunner>,
    ) -> Self {
        Desktop {
            platform,
            graphical,
            browser,
            native_open,
            runner,
        }
    }

    /// The desktop this process runs on. `native_open` overrides whether
    /// opening is offered; `timeout` bounds each native command.
    pub fn system(native_open: Option<bool>, timeout: Duration) -> Self {
        let set = |name: &str| std::env::var(name).is_ok_and(|value| !value.is_empty());
        Desktop::new(
            Platform::detect(),
            set("DISPLAY") || set("WAYLAND_DISPLAY"),
            std::env::var("BROWSER")
                .ok()
                .filter(|value| !value.is_empty()),
            native_open,
            Arc::new(SystemRunner::new(timeout)),
        )
    }

    /// Whether handing a path to the desktop can work at all.
    pub fn can_open(&self) -> bool {
        match self.platform {
            Platform::Mac => self.native_open.unwrap_or(true),
            Platform::Linux => self.native_open.unwrap_or(self.graphical),
            Platform::Other => false,
        }
    }

    /// The applications registered for `path`, in preference order. A
    /// platform without a query reports none.
    async fn applications(&self, path: &Path, icons: bool) -> Result<Vec<MacApplication>, String> {
        if self.platform != Platform::Mac {
            return Ok(Vec::new());
        }
        let stdout = self
            .runner
            .run(
                "osascript",
                vec![
                    "-l".to_string(),
                    "JavaScript".to_string(),
                    "-e".to_string(),
                    MAC_APPLICATIONS.to_string(),
                    path.display().to_string(),
                    if icons { "icons" } else { "handlers" }.to_string(),
                ],
            )
            .await?;
        let listed: Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("the application query printed invalid JSON: {e}"))?;
        parse_mac_applications(&listed)
    }

    /// Open `path` with the default application, or the default browser for a
    /// document a browser renders.
    async fn open_default(&self, path: &Path) -> Result<(), String> {
        let shown = path.display().to_string();
        if self.browser_renders(path) && self.open_in_browser(&shown).await? {
            return Ok(());
        }
        match self.platform {
            Platform::Mac => self.runner.launch("open", vec![shown]).await,
            Platform::Linux => self.runner.launch("xdg-open", vec![shown]).await,
            Platform::Other => {
                Err("native path opening is unsupported on this platform".to_string())
            }
        }
    }

    fn browser_renders(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                BROWSER_DOCUMENTS
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(extension))
            })
    }

    /// Hand `path` to the default browser. `Ok(false)` means the platform
    /// names no browser, so the caller uses the default application.
    async fn open_in_browser(&self, path: &str) -> Result<bool, String> {
        match self.platform {
            Platform::Mac => {
                // A fresh account never changed a default and has no record:
                // the system's own handler is then the answer.
                let Ok(records) = self
                    .runner
                    .run(
                        "defaults",
                        vec![
                            "read".to_string(),
                            "com.apple.LaunchServices/com.apple.launchservices.secure".to_string(),
                        ],
                    )
                    .await
                else {
                    return Ok(false);
                };
                let Some(bundle) = mac_bundle_for_https(&records) else {
                    return Ok(false);
                };
                self.runner
                    .launch("open", vec!["-b".to_string(), bundle, path.to_string()])
                    .await?;
                Ok(true)
            }
            Platform::Linux => match &self.browser {
                Some(browser) => {
                    self.runner.launch(browser, vec![path.to_string()]).await?;
                    Ok(true)
                }
                None => Ok(false),
            },
            Platform::Other => Ok(false),
        }
    }

    /// Open `path` in one application, which must be registered for it right now.
    async fn open_in(&self, path: &Path, application: &str) -> Result<(), String> {
        let registered = self.applications(path, false).await?;
        if !registered.iter().any(|app| app.id == application) {
            return Err("Application is not registered for this file".to_string());
        }
        self.runner
            .launch(
                "open",
                vec![
                    "-a".to_string(),
                    application.to_string(),
                    path.display().to_string(),
                ],
            )
            .await
    }

    /// Show `path` in the file manager.
    async fn reveal(&self, path: &Path) -> Result<(), String> {
        match self.platform {
            Platform::Mac => {
                self.runner
                    .launch("open", vec!["-R".to_string(), path.display().to_string()])
                    .await
            }
            Platform::Linux => {
                let parent = path.parent().unwrap_or(path);
                self.runner
                    .launch("xdg-open", vec![parent.display().to_string()])
                    .await
            }
            Platform::Other => {
                Err("native file manager is unsupported on this platform".to_string())
            }
        }
    }
}

/// One application the OS offers for a file, with the grouping metadata the
/// display list collapses on.
#[derive(Debug, Clone, PartialEq)]
struct MacApplication {
    id: String,
    name: String,
    default: bool,
    icon: Option<String>,
    bundle: Option<String>,
    version: Option<String>,
}

fn parse_mac_applications(value: &Value) -> Result<Vec<MacApplication>, String> {
    let entries = value.as_array().ok_or("Invalid native application list")?;
    entries.iter().map(parse_mac_application).collect()
}

fn parse_mac_application(entry: &Value) -> Result<MacApplication, String> {
    const INVALID: &str = "Invalid native application entry";
    let object = entry.as_object().ok_or(INVALID)?;
    let text = |key: &str| object.get(key).and_then(Value::as_str);
    let optional = |key: &str| match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(INVALID),
    };
    let icon = match object.get("icon") {
        Some(Value::Null) => None,
        Some(Value::String(url)) if is_image_data_url(url) => Some(url.clone()),
        _ => return Err(INVALID.to_string()),
    };
    match (
        text("id"),
        text("name"),
        object.get("default").and_then(Value::as_bool),
    ) {
        (Some(id), Some(name), Some(default)) if !id.is_empty() => Ok(MacApplication {
            id: id.to_string(),
            name: name.to_string(),
            default,
            icon,
            bundle: optional("bundle")?,
            version: optional("version")?,
        }),
        _ => Err(INVALID.to_string()),
    }
}

/// A PNG data URL, the only icon the application query produces.
fn is_image_data_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("data:image/png;base64,") else {
        return false;
    };
    rest.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
        && !rest.is_empty()
}

/// Collapse duplicate registrations of one application for display.
///
/// Self-updating apps leave extra copies on disk and LaunchServices registers
/// every one. Copies sharing a bundle identifier and display name collapse to
/// the system default, else the highest version, at the group's first position.
/// Launch validation uses the full list, so every registered copy stays openable.
fn dedupe(applications: Vec<MacApplication>) -> Vec<MacApplication> {
    let mut order: Vec<MacApplication> = Vec::new();
    let mut groups: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    for app in applications {
        let Some(bundle) = app.bundle.clone() else {
            order.push(app);
            continue;
        };
        let key = (bundle, app.name.clone());
        match groups.get(&key) {
            None => {
                groups.insert(key, order.len());
                order.push(app);
            }
            Some(&index) => {
                let held = &order[index];
                if !held.default
                    && (app.default
                        || compare_versions(app.version.as_deref(), held.version.as_deref())
                            == std::cmp::Ordering::Greater)
                {
                    order[index] = app;
                }
            }
        }
    }
    order
}

/// Order two dotted versions numerically; a non-numeric segment counts as 0
/// and a missing version sorts lowest.
fn compare_versions(left: Option<&str>, right: Option<&str>) -> std::cmp::Ordering {
    let (Some(left), Some(right)) = (left, right) else {
        return left.is_some().cmp(&right.is_some());
    };
    let segments = |version: &str| -> Vec<f64> {
        version
            .split('.')
            .map(|part| {
                part.trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|n| n.is_finite())
                    .unwrap_or(0.0)
            })
            .collect()
    };
    let (a, b) = (segments(left), segments(right));
    for index in 0..a.len().max(b.len()) {
        let ordering = a
            .get(index)
            .copied()
            .unwrap_or(0.0)
            .total_cmp(&b.get(index).copied().unwrap_or(0.0));
        if ordering != std::cmp::Ordering::Equal {
            return ordering;
        }
    }
    std::cmp::Ordering::Equal
}

/// The bundle registered for `https`, as LaunchServices records it. The nested
/// version dictionary is removed first because it has its own `LSHandlerRoleAll`.
fn mac_bundle_for_https(records: &str) -> Option<String> {
    let versions = regex::Regex::new(r"LSHandlerPreferredVersions\s*=\s*\{[^}]*\};").ok()?;
    let stripped = versions.replace_all(records, "");
    let block = regex::Regex::new(r#"\{[^{}]*LSHandlerURLScheme\s*=\s*"?https"?;[^{}]*\}"#)
        .ok()?
        .find(&stripped)?
        .as_str()
        .to_string();
    let role = regex::Regex::new(r#"LSHandlerRoleAll\s*=\s*"?([A-Za-z0-9][\w.-]*)"?;"#).ok()?;
    Some(role.captures(&block)?.get(1)?.as_str().to_string())
}

/// Serve one desktop endpoint, or `None` when the method is not one of them.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    match method {
        "session/canOpenWorkspacePath" => Some(match check_descriptor(method, &[], payload) {
            Ok(()) => RpcResult::ok(json!(router.desktop.can_open())),
            Err(failure) => RpcResult::failed(failure),
        }),
        "session/workspacePathApplications" => Some(applications(router, ctx, payload).await),
        "session/openWorkspacePath" => Some(open(router, ctx, payload).await),
        _ => None,
    }
}

/// What `request` asks for.
struct OpenRequest {
    path: String,
    reveal: bool,
    application: Option<String>,
}

impl OpenRequest {
    /// How the path is opened, for the host log.
    fn how(&self) -> String {
        match (&self.application, self.reveal) {
            (_, true) => "revealed".to_string(),
            (Some(application), false) => format!("in {application}"),
            (None, false) => "with the default application".to_string(),
        }
    }
}

/// Check the descriptor, then the request object, as the typed gateway does.
fn read_request(endpoint: &str, payload: &Value, opening: bool) -> Result<OpenRequest, Failure> {
    check_descriptor(endpoint, &["request"], payload)?;
    let request = &payload["request"];
    let path = request.get("path").and_then(Value::as_str);
    let action = request.get("action");
    let application = request.get("application");
    let valid = request.is_object()
        && path.is_some()
        && (!opening
            || (action.is_none_or(|value| value == "reveal")
                && application.is_none_or(Value::is_string)));
    match path {
        Some(path) if valid => Ok(OpenRequest {
            path: path.to_string(),
            reveal: action.is_some(),
            application: application.and_then(Value::as_str).map(str::to_string),
        }),
        _ => Err(boundary(endpoint, "request")),
    }
}

async fn applications(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "session/workspacePathApplications";
    let request = match read_request(ENDPOINT, payload, false) {
        Ok(request) => request,
        Err(failure) => return RpcResult::failed(failure),
    };
    let desktop = &router.desktop;
    if !desktop.can_open() {
        return RpcResult::ok(json!([]));
    }
    let outcome = async {
        let path = match verified_host_path(&request.path).await {
            Ok(path) => path,
            Err(PathError::Refused(failure)) => return Err(failure),
            Err(PathError::Unreadable(cause)) => {
                return Err(internal(
                    ctx,
                    ENDPOINT,
                    "file application query failed",
                    &cause,
                ))
            }
        };
        match desktop.applications(&path, true).await {
            Ok(listed) => Ok(dedupe(listed)),
            Err(cause) => Err(internal(
                ctx,
                ENDPOINT,
                "file application query failed",
                &cause,
            )),
        }
    }
    .await;
    match outcome {
        Ok(listed) => RpcResult::ok(Value::Array(
            listed
                .into_iter()
                .map(|app| json!({ "id": app.id, "name": app.name, "default": app.default, "icon": app.icon }))
                .collect(),
        )),
        Err(failure) => RpcResult::failed(failure),
    }
}

async fn open(router: &Router, ctx: &Context, payload: &Value) -> RpcResult {
    const ENDPOINT: &str = "session/openWorkspacePath";
    let request = match read_request(ENDPOINT, payload, true) {
        Ok(request) => request,
        Err(failure) => return RpcResult::failed(failure),
    };
    let desktop = &router.desktop;
    let path = match verified_host_path(&request.path).await {
        Ok(path) => path,
        Err(PathError::Refused(failure)) => return RpcResult::failed(failure),
        Err(PathError::Unreadable(cause)) => {
            return RpcResult::failed(internal(ctx, ENDPOINT, "path open failed", &cause));
        }
    };
    let done = if !desktop.can_open() {
        Err("this host has no desktop to open the path on".to_string())
    } else if request.reveal {
        desktop.reveal(&path).await
    } else if let Some(application) = &request.application {
        desktop.open_in(&path, application).await
    } else {
        desktop.open_default(&path).await
    };
    match done {
        Ok(()) => {
            ctx.logger().log_event(
                cordis::logger::LogLevel::Info,
                "gui".to_string(),
                None,
                format!("{ENDPOINT}: opened {} ({})", path.display(), request.how()),
            );
            RpcResult::ok(json!({ "opened": true }))
        }
        Err(cause) => RpcResult::failed(internal(
            ctx,
            ENDPOINT,
            "path open failed",
            &format!("{} ({}): {cause}", path.display(), request.how()),
        )),
    }
}

/// The failure a client sees, which carries no cause; the cause is logged.
fn internal(ctx: &Context, endpoint: &str, message: &str, cause: &str) -> Failure {
    ctx.logger().log_event(
        cordis::logger::LogLevel::Warn,
        "gui".to_string(),
        None,
        format!("{endpoint}: {message}: {cause}"),
    );
    Failure {
        code: codes::INTERNAL.to_string(),
        message: message.to_string(),
        details: json!({}),
    }
}

enum PathError {
    /// The request named a path this host refuses.
    Refused(Failure),
    /// The host could not examine the path.
    Unreadable(String),
}

fn refused(message: &str) -> PathError {
    PathError::Refused(Failure {
        code: codes::BAD_REQUEST.to_string(),
        message: message.to_string(),
        details: json!({}),
    })
}

/// Resolve `path` the way Node's `path.resolve` does: relative to the working
/// directory, `.` and `..` applied lexically, no trailing separator.
///
/// The result is always absolute, so it can never begin with `-` and be read
/// as an option by `open` or `xdg-open`.
fn lexical_resolve(path: &str) -> Result<PathBuf, String> {
    let given = Path::new(path);
    let mut resolved = if given.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().map_err(|e| format!("no working directory: {e}"))?
    };
    for component in given.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => resolved.push(component),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(name) => resolved.push(name),
        }
    }
    Ok(resolved)
}

/// The canonical form of `path`: its longest existing ancestor with symlinks
/// resolved, then the remaining names unchanged.
async fn canonical_form(path: &Path) -> Result<PathBuf, String> {
    let mut ancestor = path;
    let mut rest: Vec<&std::ffi::OsStr> = Vec::new();
    loop {
        match tokio::fs::canonicalize(ancestor).await {
            Ok(mut canonical) => {
                canonical.extend(rest.iter().rev());
                return Ok(canonical);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let (Some(parent), Some(name)) = (ancestor.parent(), ancestor.file_name()) else {
                    return Err(format!("no existing ancestor of {}", path.display()));
                };
                rest.push(name);
                ancestor = parent;
            }
            Err(error) => return Err(format!("cannot resolve {}: {error}", ancestor.display())),
        }
    }
}

/// The path as the host spells it, refused unless it is already canonical.
async fn verified_host_path(path: &str) -> Result<PathBuf, PathError> {
    if path.is_empty() {
        return Err(refused("A non-empty file path is required"));
    }
    let resolved = lexical_resolve(path).map_err(PathError::Unreadable)?;
    let canonical = canonical_form(&resolved)
        .await
        .map_err(PathError::Unreadable)?;
    if canonical != resolved {
        return Err(refused("Path has no verified Host path"));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Records every command and answers from a script.
    struct Script {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        answers: Mutex<Vec<Result<String, String>>>,
    }

    impl Script {
        fn new(answers: Vec<Result<String, String>>) -> Arc<Self> {
            Arc::new(Script {
                calls: Mutex::new(Vec::new()),
                answers: Mutex::new(answers),
            })
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl CommandRunner for Script {
        fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, String>> {
            self.calls.lock().unwrap().push((program.to_string(), args));
            let mut answers = self.answers.lock().unwrap();
            let answer = if answers.is_empty() {
                Err("no answer scripted".to_string())
            } else {
                answers.remove(0)
            };
            Box::pin(async move { answer })
        }

        fn launch(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<(), String>> {
            let answered = self.run(program, args);
            Box::pin(async move { answered.await.map(drop) })
        }
    }

    fn mac(script: &Arc<Script>) -> Desktop {
        Desktop::new(Platform::Mac, true, None, None, script.clone())
    }

    fn app(
        id: &str,
        name: &str,
        bundle: Option<&str>,
        version: Option<&str>,
        default: bool,
    ) -> MacApplication {
        MacApplication {
            id: id.to_string(),
            name: name.to_string(),
            default,
            icon: None,
            bundle: bundle.map(str::to_string),
            version: version.map(str::to_string),
        }
    }

    #[test]
    fn opening_is_offered_where_a_desktop_is_reachable() {
        let script = Script::new(vec![]);
        let offered = |platform, graphical, override_| {
            Desktop::new(platform, graphical, None, override_, script.clone()).can_open()
        };
        assert!(offered(Platform::Mac, false, None));
        assert!(offered(Platform::Linux, true, None));
        assert!(!offered(Platform::Linux, false, None));
        assert!(!offered(Platform::Other, true, None));
        assert!(
            offered(Platform::Linux, false, Some(true)),
            "the operator's answer wins on a supported platform"
        );
        assert!(
            !offered(Platform::Mac, true, Some(false)),
            "the operator's answer wins on a supported platform"
        );
        assert!(
            !offered(Platform::Other, true, Some(true)),
            "an unsupported platform cannot open anything"
        );
    }

    #[test]
    fn copies_of_one_application_collapse_to_the_default_else_the_newest() {
        let listed = vec![
            app(
                "/A/Editor 1.app",
                "Editor",
                Some("x.editor"),
                Some("1.2"),
                false,
            ),
            app("/Other.app", "Other", None, None, false),
            app(
                "/A/Editor 3.app",
                "Editor",
                Some("x.editor"),
                Some("1.10"),
                false,
            ),
            app(
                "/A/Editor 2.app",
                "Editor",
                Some("x.editor"),
                Some("1.9"),
                false,
            ),
            app("/B/Viewer.app", "Viewer", Some("x.viewer"), Some("2"), true),
            app(
                "/B/Viewer new.app",
                "Viewer",
                Some("x.viewer"),
                Some("9"),
                false,
            ),
            app(
                "/C/Side.app",
                "Side by side",
                Some("x.editor"),
                Some("1"),
                false,
            ),
        ];
        let ids: Vec<_> = dedupe(listed).into_iter().map(|a| a.id).collect();
        assert_eq!(
            ids,
            ["/A/Editor 3.app", "/Other.app", "/B/Viewer.app", "/C/Side.app"],
            "versions compare numerically, a default is never displaced, and a distinct display name stays"
        );
    }

    #[test]
    fn a_missing_version_sorts_below_any_version() {
        use std::cmp::Ordering::{Equal, Greater, Less};
        assert_eq!(compare_versions(None, Some("0")), Less);
        assert_eq!(compare_versions(Some("0"), None), Greater);
        assert_eq!(compare_versions(None, None), Equal);
        assert_eq!(compare_versions(Some("1.x"), Some("1.0")), Equal);
    }

    #[test]
    fn an_application_entry_must_be_well_formed() {
        let ok = json!({ "id": "/A.app", "name": "A", "default": true, "icon": null, "bundle": "", "version": null });
        assert!(parse_mac_applications(&json!([ok])).is_ok());
        let bad = |patch: Value| {
            let mut entry = ok.clone();
            entry
                .as_object_mut()
                .unwrap()
                .extend(patch.as_object().unwrap().clone());
            parse_mac_applications(&json!([entry])).unwrap_err()
        };
        for patch in [
            json!({ "id": "" }),
            json!({ "name": 3 }),
            json!({ "default": "yes" }),
            json!({ "icon": "https://example.com/a.png" }),
            json!({ "icon": "data:image/png;base64," }),
            json!({ "icon": "data:image/svg+xml;base64,AAAA" }),
            json!({ "icon": "data:image/png;base64,not base64!" }),
            json!({ "bundle": 7 }),
        ] {
            assert_eq!(bad(patch), "Invalid native application entry");
        }
        assert_eq!(
            parse_mac_applications(&json!({})).unwrap_err(),
            "Invalid native application list"
        );
    }

    #[test]
    fn the_default_browser_is_read_from_the_https_record() {
        let records = r#"{ LSHandlers = (
            { LSHandlerContentType = "public.html"; LSHandlerRoleAll = "com.other.app"; },
            { LSHandlerPreferredVersions = { LSHandlerRoleAll = "-"; }; LSHandlerRoleAll = "org.mozilla.firefox"; LSHandlerURLScheme = https; }
        ); }"#;
        assert_eq!(
            mac_bundle_for_https(records).as_deref(),
            Some("org.mozilla.firefox")
        );
        assert_eq!(mac_bundle_for_https("{ LSHandlers = (); }"), None);
    }

    #[test]
    fn a_resolved_path_is_absolute_so_it_cannot_be_read_as_an_option() {
        for given in ["-n", "--reveal", "-a/x", "./-n", "../-n", "a/../-b"] {
            let resolved = lexical_resolve(given).unwrap();
            assert!(
                resolved.is_absolute(),
                "{given} resolved to {}",
                resolved.display()
            );
            assert!(!resolved.to_string_lossy().starts_with('-'));
        }
    }

    #[tokio::test]
    async fn a_path_is_verified_only_when_it_is_already_canonical() {
        let dir = tempfile::tempdir().unwrap();
        let real = std::fs::canonicalize(dir.path()).unwrap();
        std::fs::write(real.join("a.txt"), "x").unwrap();
        std::os::unix::fs::symlink(real.join("a.txt"), real.join("link.txt")).unwrap();
        let verify = |text: String| async move { verified_host_path(&text).await };

        let spelled = |tail: &str| format!("{}{tail}", real.display());
        assert_eq!(
            verify(spelled("/a.txt")).await.ok(),
            Some(real.join("a.txt"))
        );
        assert_eq!(
            verify(spelled("/missing/deeper.txt")).await.ok(),
            Some(real.join("missing/deeper.txt")),
            "a path that does not exist yet still verifies under a canonical ancestor"
        );
        assert_eq!(
            verify(spelled("/./sub/../a.txt")).await.ok(),
            Some(real.join("a.txt")),
            "dots resolve lexically"
        );
        assert_eq!(
            verify(spelled("/a.txt/")).await.ok(),
            Some(real.join("a.txt")),
            "a trailing separator is dropped"
        );
        match verify(spelled("/link.txt")).await {
            Err(PathError::Refused(failure)) => {
                assert_eq!(failure.message, "Path has no verified Host path")
            }
            other => panic!(
                "a symlinked spelling must be refused, got ok={}",
                other.is_ok()
            ),
        }
        for beneath_a_file in [spelled("/a.txt/under"), spelled("/link.txt/under")] {
            assert!(
                matches!(verify(beneath_a_file).await, Err(PathError::Unreadable(_))),
                "a path beneath a file cannot be examined: a failed query, not a refusal"
            );
        }
        match verify(String::new()).await {
            Err(PathError::Refused(failure)) => {
                assert_eq!(failure.message, "A non-empty file path is required")
            }
            other => panic!("an empty path must be refused, got ok={}", other.is_ok()),
        }
    }

    #[tokio::test]
    async fn the_default_open_uses_the_system_opener_and_a_browser_for_web_documents() {
        let script = Script::new(vec![Ok(String::new())]);
        mac(&script)
            .open_default(Path::new("/w/notes.txt"))
            .await
            .unwrap();
        assert_eq!(
            script.calls(),
            [("open".to_string(), vec!["/w/notes.txt".to_string()])]
        );

        let records = r#"{ LSHandlers = ( { LSHandlerRoleAll = "com.brave.Browser"; LSHandlerURLScheme = https; } ); }"#;
        let script = Script::new(vec![Ok(records.to_string()), Ok(String::new())]);
        mac(&script)
            .open_default(Path::new("/w/page.HTML"))
            .await
            .unwrap();
        assert_eq!(
            script.calls()[1],
            (
                "open".to_string(),
                vec![
                    "-b".to_string(),
                    "com.brave.Browser".to_string(),
                    "/w/page.HTML".to_string()
                ]
            )
        );

        let script = Script::new(vec![Err("no record".to_string()), Ok(String::new())]);
        mac(&script)
            .open_default(Path::new("/w/page.svg"))
            .await
            .unwrap();
        assert_eq!(
            script.calls()[1],
            ("open".to_string(), vec!["/w/page.svg".to_string()]),
            "no browser record falls back"
        );

        let script = Script::new(vec![Ok(String::new())]);
        let linux = Desktop::new(
            Platform::Linux,
            true,
            Some("firefox".to_string()),
            None,
            script.clone(),
        );
        linux.open_default(Path::new("/w/page.htm")).await.unwrap();
        assert_eq!(
            script.calls(),
            [("firefox".to_string(), vec!["/w/page.htm".to_string()])]
        );
    }

    #[tokio::test]
    async fn a_browser_value_that_names_no_executable_fails_the_open() {
        let script = Script::new(vec![Err("no such program".to_string())]);
        let linux = Desktop::new(
            Platform::Linux,
            true,
            Some("firefox %s".to_string()),
            None,
            script.clone(),
        );
        assert!(linux.open_default(Path::new("/w/page.html")).await.is_err());
        assert_eq!(
            script.calls().len(),
            1,
            "the failure is reported, not retried through another opener"
        );
    }

    #[tokio::test]
    async fn an_application_must_be_registered_for_the_file_at_the_moment_of_opening() {
        let listed =
            json!([{ "id": "/Apps/Editor.app", "name": "Editor", "default": false, "icon": null }])
                .to_string();
        let script = Script::new(vec![Ok(listed.clone()), Ok(String::new())]);
        mac(&script)
            .open_in(Path::new("/w/a.txt"), "/Apps/Editor.app")
            .await
            .unwrap();
        let calls = script.calls();
        assert_eq!(
            calls[0].1.last().map(String::as_str),
            Some("handlers"),
            "launch validation skips icons"
        );
        assert_eq!(
            calls[1],
            (
                "open".to_string(),
                vec![
                    "-a".to_string(),
                    "/Apps/Editor.app".to_string(),
                    "/w/a.txt".to_string()
                ]
            )
        );

        let script = Script::new(vec![Ok(listed)]);
        let refused = mac(&script)
            .open_in(Path::new("/w/a.txt"), "/Apps/Other.app")
            .await;
        assert_eq!(
            refused.unwrap_err(),
            "Application is not registered for this file"
        );
        assert_eq!(
            script.calls().len(),
            1,
            "nothing is launched for an unregistered application"
        );
    }

    #[tokio::test]
    async fn revealing_shows_the_file_in_the_file_manager() {
        let script = Script::new(vec![Ok(String::new())]);
        mac(&script).reveal(Path::new("/w/a.txt")).await.unwrap();
        assert_eq!(
            script.calls(),
            [(
                "open".to_string(),
                vec!["-R".to_string(), "/w/a.txt".to_string()]
            )]
        );

        let script = Script::new(vec![Ok(String::new())]);
        Desktop::new(Platform::Linux, true, None, None, script.clone())
            .reveal(Path::new("/w/a.txt"))
            .await
            .unwrap();
        assert_eq!(
            script.calls(),
            [("xdg-open".to_string(), vec!["/w".to_string()])],
            "Linux opens the parent directory"
        );
    }

    #[tokio::test]
    async fn a_system_command_that_hangs_is_killed_at_its_deadline() {
        let runner = SystemRunner::new(Duration::from_millis(100));
        let started = std::time::Instant::now();
        let outcome = runner.run("sleep", vec!["30".to_string()]).await;
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the command must not outlive its deadline"
        );
        assert!(outcome.unwrap_err().contains("did not finish"));
    }

    #[tokio::test]
    async fn a_launched_opener_returns_when_it_exits_even_if_its_application_keeps_running() {
        let runner = SystemRunner::new(Duration::from_secs(5));
        let started = std::time::Instant::now();
        // The shell exits at once; the background sleep outlives it holding the
        // pipes a capturing reader would wait on.
        runner
            .launch("sh", vec!["-c".to_string(), "sleep 20 &".to_string()])
            .await
            .unwrap();
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(runner
            .launch("false", vec![])
            .await
            .unwrap_err()
            .contains("exited with"));
        assert!(runner
            .launch("/no/such/program", vec![])
            .await
            .unwrap_err()
            .contains("could not start"));
        let hung = SystemRunner::new(Duration::from_millis(100));
        assert!(hung
            .launch("sleep", vec!["30".to_string()])
            .await
            .unwrap_err()
            .contains("did not finish"));
    }

    #[tokio::test]
    async fn a_system_command_reports_its_failure_and_its_output() {
        let runner = SystemRunner::new(Duration::from_secs(5));
        assert_eq!(
            runner.run("echo", vec!["hi".to_string()]).await.unwrap(),
            "hi\n"
        );
        assert!(runner
            .run("false", vec![])
            .await
            .unwrap_err()
            .contains("exited with"));
        assert!(runner
            .run("/no/such/program", vec![])
            .await
            .unwrap_err()
            .contains("could not start"));
    }
}
