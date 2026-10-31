//! The browser boot host: serves the page the shipped Web client boots from.
//!
//! The TypeScript host (`@deepseek-ai/dsh-client-modules`) scans every plugin
//! that declares `dsh.client` in its `package.json`, composes them into a module
//! graph, and serves that graph three ways. This module reproduces the same wire:
//!
//! - `index.html` gains the injection table: the module-loader queue script,
//!   `<link rel=preload>` rows for the application batches, one parser-blocking
//!   `<script>` for the bootstrap batch, then `window.__DSH_BOOT__ = <graph>` and
//!   the page globals the client packages read (theme, shortcuts, remote access…);
//! - `/plugins/??<id>/client.js,…&rev=<rev>` returns one combo script that
//!   concatenates several bundles, and the matching `….js.map` indexed source map;
//! - `/plugins/<id>/client.<chunk>.js?rev=<rev>` returns a lazily imported chunk.
//!
//! Revisions are opaque cache keys: they hash file size and mtime like the TS
//! host, but the digests differ, which no client observes.
//!
//! Limits, by design:
//! - there is no HMR: `client-hmr` and the desktop-only analytics row are excluded
//!   by default, and a bundle changed on disk is picked up on restart;
//! - source maps carry the authored map of each bundle without relocating its
//!   `sources`, so a debugger shows paths relative to the bundle, not the repo;
//! - the roster is read once at startup. A declared package whose bundle or
//!   `inject` dependency is missing fails startup instead of booting half a UI.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::{json, Value};

/// Longest browser-facing combo reference the TS host allows before splitting.
const MAX_COMBO_URL_BYTES: usize = 3 * 1024;
/// Hex digits kept from a revision hash.
const REV_LENGTH: usize = 12;
/// Package whose bundle implements the module system itself.
const CLIENT_MODULES_ID: &str = "@deepseek-ai/dsh-client-modules";
const PLUGIN_ROUTE: &str = "/plugins";
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";

/// The module-loader queue the first inline script installs. Copied from the TS
/// host's `bootInjections`: the client bundle for `client-modules` replaces the
/// queue with the real module system when `create()` runs.
fn queue_script() -> String {
    let bootstrap = serde_json::to_string(CLIENT_MODULES_ID).unwrap_or_default();
    format!(
        r#"(()=>{{
const pendingQueue=[]
window.__ModuleLoader__={{
  mode:"queue",
  pendingQueue,
  load(registration){{pendingQueue.push(registration)}},
  create(options){{
    if(this.mode!=="queue")throw new Error("client-modules: window.__ModuleLoader__.create called after module-system boot")
    const index=pendingQueue.findIndex(registration=>registration.id==={bootstrap})
    const registration=pendingQueue[index]
    if(registration===undefined)throw new Error("client-modules: HTML did not preload {CLIENT_MODULES_ID}/client.js")
    pendingQueue.splice(index,1)
    const exports=registration.factory(specifier=>{{
      throw new Error('client-modules: {CLIENT_MODULES_ID}/client.js requested external "'+specifier+'" before the module system existed')
    }})
    if(typeof exports!=="object"||exports===null||typeof exports.createClientModuleSystem!=="function"||typeof exports.apply!=="function"){{
      throw new Error("client-modules: {CLIENT_MODULES_ID}/client.js did not export the bootstrap module face")
    }}
    return exports.createClientModuleSystem(this,{{id:registration.id,exports}},options)
  }}
}}
}})()"#
    )
}

/// Which client packages the page loads.
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Root of the TypeScript checkout whose built `packages/*/*` supply the bundles.
    pub client_root: PathBuf,
    /// Allowlist of package names; `None` selects every declared web package that
    /// `exclude` and `include_experimental` leave in.
    pub packages: Option<Vec<String>>,
    /// Package names left out of the default roster.
    pub exclude: Vec<String>,
    /// Whether `experimental-*` packages join the default roster.
    pub include_experimental: bool,
}

/// One client package as the graph sees it.
#[derive(Debug, Clone)]
struct Row {
    id: String,
    bundle: PathBuf,
    inject: Vec<String>,
    immediately: bool,
    external: Vec<String>,
    rev: String,
}

#[derive(Debug, Clone)]
struct Batch {
    phase: &'static str,
    entries: Vec<String>,
    rev: String,
}

/// A composed boot graph plus the files that answer its URLs.
#[derive(Debug)]
pub struct BootHost {
    rows: BTreeMap<String, Row>,
    batches: Vec<Batch>,
    graph: Value,
    injections: Vec<Injection>,
}

/// A fetched resource: status, content type, body.
pub struct Served {
    /// HTTP status.
    pub status: u16,
    /// `Content-Type`.
    pub content_type: &'static str,
    /// Body bytes.
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
enum Injection {
    Global(&'static str, Value),
    Script(String),
    ScriptBody(String),
    ScriptSrc(String),
    Preload(String),
    Style(String),
}

impl BootHost {
    /// Scan the checkout, compose the graph, and fail loudly on anything that
    /// would boot half a UI.
    pub fn load(config: &BootConfig) -> Result<Self, String> {
        let declared = scan_packages(&config.client_root)?;
        let selected = select(&declared, config)?;
        check_dependencies(&selected, &declared)?;
        let ordered = order_by_module_graph(&selected)?;

        let mut rows = BTreeMap::new();
        for row in &ordered {
            rows.insert(row.id.clone(), row.clone());
        }
        let batches = compose_batches(&ordered)?;
        let version = package_version(CLIENT_MODULES_ID, &declared)?;
        let entries: Vec<Value> = ordered.iter().map(graph_row).collect();
        let batch_json: Vec<Value> = batches
            .iter()
            .map(|b| {
                json!({
                    "phase": b.phase,
                    "url": combo_reference(&b.entries, &b.rev, false),
                    "rev": b.rev,
                    "entries": b.entries,
                })
            })
            .collect();
        let graph = json!({
            "rev": short_hash(&serde_json::to_string(&json!({"entries": entries, "batches": batch_json})).unwrap_or_default()),
            "release": { "version": version },
            "entries": entries,
            "batches": batch_json,
        });
        let injections = injections(&graph, &batches, &rows);
        Ok(BootHost {
            rows,
            batches,
            graph,
            injections,
        })
    }

    /// The graph published as `window.__DSH_BOOT__`.
    pub fn graph(&self) -> &Value {
        &self.graph
    }

    /// Number of client packages in the roster.
    pub fn package_count(&self) -> usize {
        self.rows.len()
    }

    /// Render `index.html` with the injection table, as the TS web server does.
    pub fn render_index(&self, html: &str) -> String {
        let mut head = String::new();
        let mut body = String::new();
        for row in &self.injections {
            match row {
                Injection::Global(name, value) => {
                    let name = serde_json::to_string(name).unwrap_or_default().replace('<', "\\u003c");
                    let value = serde_json::to_string(value).unwrap_or_default().replace('<', "\\u003c");
                    head.push_str(&format!("<script>globalThis[{name}] = {value}</script>"));
                }
                Injection::Script(text) => head.push_str(&format!("<script>{text}</script>")),
                Injection::ScriptBody(text) => body.push_str(&format!("<script>{text}</script>")),
                Injection::ScriptSrc(src) => {
                    head.push_str(&format!("<script src=\"{}\"></script>", escape_attr(src)))
                }
                Injection::Preload(src) => head.push_str(&format!(
                    "<link rel=\"preload\" as=\"script\" href=\"{}\">",
                    escape_attr(src)
                )),
                Injection::Style(text) => head.push_str(&format!("<style>{text}</style>")),
            }
        }
        body.push_str(
            "<script>(globalThis.__DSH_BOOT_READY__ ??= Promise.withResolvers()).resolve()</script>",
        );
        let mut out = html.to_string();
        out = splice_after_open_tag(&out, "head", &head);
        splice_after_open_tag(&out, "body", &body)
    }

    /// Answer a `/plugins/…` request, or `None` when the URL is not advertised.
    pub fn serve(&self, path: &str, query: Option<&str>) -> Option<Served> {
        if let Some(rest) = path.strip_prefix(&format!("{PLUGIN_ROUTE}/")) {
            if rest.is_empty() {
                return self.serve_combo(query?);
            }
            return self.serve_chunk(rest, query);
        }
        None
    }

    fn serve_combo(&self, query: &str) -> Option<Served> {
        let query = percent_decode(query);
        let query = query.strip_prefix('?')?;
        let (resources, rev) = query.split_once("&rev=")?;
        let source_map = resources.ends_with(".map");
        let suffix = if source_map { "/client.js.map" } else { "/client.js" };
        let ids: Vec<String> = resources
            .split(',')
            .map(|r| r.strip_suffix(suffix).map(str::to_string))
            .collect::<Option<_>>()?;
        // Only an advertised combination is a resource; anything else is unknown.
        let batch = self
            .batches
            .iter()
            .find(|b| b.entries == ids && b.rev == rev)?;
        let rows: Vec<&Row> = batch.entries.iter().filter_map(|id| self.rows.get(id)).collect();
        if source_map {
            return Some(Served {
                status: 200,
                content_type: "application/json; charset=utf-8",
                body: combo_source_map(&rows, "client.js").into_bytes(),
            });
        }
        let map_ref = combo_search(&batch.entries, &batch.rev, true);
        let mut script = String::new();
        for row in &rows {
            script.push_str(&prepared_source(&row.bundle).ok()?);
            script.push_str(";\n");
        }
        script.push_str(&format!("//# sourceMappingURL={map_ref}\n"));
        Some(Served {
            status: 200,
            content_type: "text/javascript; charset=utf-8",
            body: script.into_bytes(),
        })
    }

    fn serve_chunk(&self, rest: &str, query: Option<&str>) -> Option<Served> {
        let chunk = Regex::new(r"^client\.[A-Za-z0-9][A-Za-z0-9._-]*\.js$").ok()?;
        for row in self.rows.values() {
            let Some(requested) = rest.strip_prefix(&format!("{}/", row.id)) else {
                continue;
            };
            let source_map = requested.ends_with(".map");
            let file = requested.strip_suffix(".map").unwrap_or(requested);
            // The Node entry shares the directory and must never reach a browser.
            if !chunk.is_match(file) || file.starts_with("client.node.") {
                return None;
            }
            if query != Some(&format!("rev={}", row.rev)) {
                return None;
            }
            let path = row.bundle.parent()?.join(file);
            if !path.is_file() {
                return None;
            }
            if source_map {
                return Some(Served {
                    status: 200,
                    content_type: "application/json; charset=utf-8",
                    body: fs::read(path.with_extension("js.map"))
                        .unwrap_or_else(|_| b"{\"version\":3,\"sources\":[],\"names\":[],\"mappings\":\"\"}".to_vec()),
                });
            }
            let mut script = prepared_source(&path).ok()?;
            script.push_str(";\n");
            script.push_str(&format!("//# sourceMappingURL={file}.map?rev={}\n", row.rev));
            return Some(Served {
                status: 200,
                content_type: "text/javascript; charset=utf-8",
                body: script.into_bytes(),
            });
        }
        None
    }

    /// Cache header for every `/plugins` resource.
    pub fn cache_control() -> &'static str {
        IMMUTABLE_CACHE
    }
}

// ---------------------------------------------------------------- scanning

#[derive(Debug, Clone)]
struct Declared {
    name: String,
    version: String,
    inject: Vec<String>,
    immediately: bool,
    external: Vec<String>,
    bundle: PathBuf,
}

fn scan_packages(root: &Path) -> Result<Vec<Declared>, String> {
    let packages = root.join("packages");
    if !packages.is_dir() {
        return Err(format!(
            "gui: client_root {} has no packages/ directory",
            root.display()
        ));
    }
    let mut out = Vec::new();
    for group in read_dirs(&packages) {
        for dir in read_dirs(&group) {
            let manifest = dir.join("package.json");
            let Ok(text) = fs::read_to_string(&manifest) else {
                continue;
            };
            let value: Value = serde_json::from_str(&text)
                .map_err(|e| format!("gui: {} is not valid JSON: {e}", manifest.display()))?;
            let Some(client) = value.pointer("/dsh/client") else {
                continue;
            };
            if client.get("platform").and_then(Value::as_str) != Some("web") {
                continue;
            }
            let name = value
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("gui: {} has no name", manifest.display()))?
                .to_string();
            let strings = |key: &str| -> Vec<String> {
                client
                    .get(key)
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default()
            };
            let relative = client_export(&name, value.get("exports"))?;
            out.push(Declared {
                name,
                bundle: dir.join(relative),
                version: value
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or("0.0.0")
                    .to_string(),
                inject: strings("inject"),
                immediately: client.get("immediately").and_then(Value::as_bool) == Some(true),
                external: strings("external"),
            });
        }
    }
    if out.is_empty() {
        return Err(format!(
            "gui: no package under {} declares dsh.client for the web platform",
            packages.display()
        ));
    }
    Ok(out)
}

/// The `./client` export, matching the TS rule: a string, or an object's `default`.
fn client_export(name: &str, exports: Option<&Value>) -> Result<String, String> {
    let Some(client) = exports.and_then(|e| e.get("./client")) else {
        return Err(format!("gui: {name} declares dsh.client but exports no \"./client\" bundle"));
    };
    if let Some(s) = client.as_str() {
        return Ok(s.to_string());
    }
    match client.get("default").and_then(Value::as_str) {
        Some(s) => Ok(s.to_string()),
        None => Err(format!(
            "gui: {name} exports[\"./client\"] must be a string or an object with a string default"
        )),
    }
}

fn read_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn select(declared: &[Declared], config: &BootConfig) -> Result<Vec<Row>, String> {
    let by_name: HashMap<&str, &Declared> = declared.iter().map(|d| (d.name.as_str(), d)).collect();
    let wanted: Vec<&Declared> = match &config.packages {
        Some(names) => names
            .iter()
            .map(|n| {
                by_name
                    .get(n.as_str())
                    .copied()
                    .ok_or_else(|| format!("gui: client package {n} is not declared under client_root"))
            })
            .collect::<Result<_, _>>()?,
        None => declared
            .iter()
            .filter(|d| !config.exclude.contains(&d.name))
            .filter(|d| config.include_experimental || !is_experimental(&d.name))
            .collect(),
    };
    let mut rows = Vec::new();
    for d in wanted {
        let bundle = d.bundle.clone();
        let meta = fs::metadata(&bundle).map_err(|e| {
            format!(
                "gui: client bundle not found for {} at {} ({e}); build the TS checkout first",
                d.name,
                bundle.display()
            )
        })?;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        rows.push(Row {
            id: d.name.clone(),
            rev: short_hash(&format!("{}:{}:{}", d.name, meta.len(), mtime)),
            bundle,
            inject: d.inject.clone(),
            immediately: d.immediately,
            external: d.external.clone(),
        });
    }
    Ok(rows)
}

fn package_version(name: &str, declared: &[Declared]) -> Result<String, String> {
    declared
        .iter()
        .find(|d| d.name == name)
        .map(|d| d.version.clone())
        .ok_or_else(|| format!("gui: the roster has no {name}; the page cannot build its module system"))
}

/// An `inject` edge may name a platform module the shell bundles or a host
/// service, which the page resolves itself. It must not name a client package
/// that is declared but left out of the roster: that boots a UI missing a part.
fn check_dependencies(rows: &[Row], declared: &[Declared]) -> Result<(), String> {
    let ids: HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    if !ids.contains(CLIENT_MODULES_ID) {
        return Err(format!(
            "gui: the client roster must include {CLIENT_MODULES_ID}, which implements the module system"
        ));
    }
    let declared: HashSet<&str> = declared.iter().map(|d| d.name.as_str()).collect();
    let mut left_out: Vec<String> = Vec::new();
    for row in rows {
        for dep in row.inject.iter().chain(row.external.iter().map(|e| e)) {
            let dep = strip_client_suffix(dep);
            if declared.contains(dep) && !ids.contains(dep) {
                left_out.push(format!("{} needs {dep}", row.id));
            }
        }
    }
    if left_out.is_empty() {
        return Ok(());
    }
    Err(format!(
        "gui: client packages need others that are declared but not in the roster:\n  {}",
        left_out.join("\n  ")
    ))
}

fn strip_client_suffix(name: &str) -> &str {
    name.strip_suffix("/client").unwrap_or(name)
}

/// A requested package row precedes its consumers.
fn order_by_module_graph(rows: &[Row]) -> Result<Vec<Row>, String> {
    let by_id: HashMap<&str, &Row> = rows.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut placed: HashSet<String> = HashSet::new();
    let mut open: Vec<String> = Vec::new();
    let mut out = Vec::new();
    fn visit<'a>(
        row: &'a Row,
        by_id: &HashMap<&str, &'a Row>,
        placed: &mut HashSet<String>,
        open: &mut Vec<String>,
        out: &mut Vec<Row>,
    ) -> Result<(), String> {
        if placed.contains(&row.id) {
            return Ok(());
        }
        if let Some(start) = open.iter().position(|id| id == &row.id) {
            let mut cycle = open[start..].to_vec();
            cycle.push(row.id.clone());
            return Err(format!("gui: client module graph cycle {}", cycle.join(" -> ")));
        }
        open.push(row.id.clone());
        for name in &row.external {
            let dep = by_id
                .get(name.as_str())
                .or_else(|| by_id.get(strip_client_suffix(name)))
                .copied();
            if let Some(dep) = dep {
                if dep.id == row.id {
                    return Err(format!("gui: {} requests its own package as external", row.id));
                }
                visit(dep, by_id, placed, open, out)?;
            }
        }
        open.pop();
        placed.insert(row.id.clone());
        out.push(row.clone());
        Ok(())
    }
    for row in rows {
        visit(row, &by_id, &mut placed, &mut open, &mut out)?;
    }
    Ok(out)
}

// ---------------------------------------------------------------- composition

fn compose_batches(ordered: &[Row]) -> Result<Vec<Batch>, String> {
    let bootstrap: Vec<&Row> = ordered.iter().filter(|r| r.id == CLIENT_MODULES_ID).collect();
    let application: Vec<&Row> = ordered.iter().filter(|r| r.id != CLIENT_MODULES_ID).collect();
    let mut batches = Vec::new();
    for (phase, rows) in [("bootstrap", bootstrap), ("application", application)] {
        for chunk in partition(&rows)? {
            let entries: Vec<String> = chunk.iter().map(|r| r.id.clone()).collect();
            let rev = short_hash(
                &chunk
                    .iter()
                    .map(|r| format!("{}:{}", r.id, r.rev))
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            batches.push(Batch { phase, entries, rev });
        }
    }
    Ok(batches)
}

fn partition<'a>(rows: &[&'a Row]) -> Result<Vec<Vec<&'a Row>>, String> {
    let placeholder = "0".repeat(REV_LENGTH);
    let bytes = |chunk: &[&Row]| {
        let ids: Vec<String> = chunk.iter().map(|r| r.id.clone()).collect();
        combo_reference(&ids, &placeholder, true).len()
    };
    let mut chunks: Vec<Vec<&Row>> = Vec::new();
    let mut current: Vec<&Row> = Vec::new();
    for row in rows {
        let mut candidate = current.clone();
        candidate.push(row);
        if bytes(&candidate) <= MAX_COMBO_URL_BYTES {
            current = candidate;
            continue;
        }
        if current.is_empty() {
            return Err(format!("gui: {} exceeds the {MAX_COMBO_URL_BYTES}-byte combo URL limit", row.id));
        }
        chunks.push(std::mem::take(&mut current));
        current.push(row);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    Ok(chunks)
}

fn graph_row(row: &Row) -> Value {
    let mut value = json!({
        "id": row.id,
        "url": combo_reference(std::slice::from_ref(&row.id), &row.rev, false),
        "rev": row.rev,
    });
    let object = value.as_object_mut().expect("json! object");
    if !row.inject.is_empty() {
        object.insert("inject".into(), json!(row.inject));
    }
    if row.immediately {
        object.insert("immediately".into(), json!(true));
    }
    if !row.external.is_empty() {
        object.insert("external".into(), json!(row.external));
    }
    value
}

fn combo_search(ids: &[String], rev: &str, source_map: bool) -> String {
    let map = if source_map { ".map" } else { "" };
    let resources: Vec<String> = ids.iter().map(|id| format!("{id}/client.js{map}")).collect();
    format!("??{}&rev={rev}", resources.join(","))
}

/// Document-relative reference, with the route's leading slash stripped.
fn combo_reference(ids: &[String], rev: &str, source_map: bool) -> String {
    format!("{}/{}", PLUGIN_ROUTE.trim_start_matches('/'), combo_search(ids, rev, source_map))
}

// ---------------------------------------------------------------- injections

/// Theme font sizes and defaults mirrored from `@deepseek-ai/dsh-client-ui-theme`.
const THEME_BODY_SCRIPT: &str = r#"(() => {
  const preference = "system"
  const systemDark = preference === 'system'
    && typeof matchMedia !== 'undefined'
    && matchMedia('(prefers-color-scheme: dark)').matches
  const dark = preference === 'dark' || systemDark
  document.documentElement.dataset.dsThemeSource = preference
  document.body.toggleAttribute('data-ds-dark-theme', dark)
  document.body.style.setProperty("--dsh-content-font-size", "14px")
  document.body.style.setProperty("--dsh-code-font-size", "11px")
  document.body.style.setProperty("--dsh-terminal-font-size", "13px")
})()"#;

const THEME_STYLE: &str = ":root{color-scheme:light}body{background-color:#fff;--dsh-boot-bg:#fff}@media(prefers-color-scheme:dark){:root{color-scheme:dark}body{background-color:#151517;--dsh-boot-bg:#151517}}";

/// Page globals the client packages read, with the defaults their TS schemas
/// give an unconfigured host. A row is emitted only when its package is loaded.
fn page_globals(rows: &BTreeMap<String, Row>) -> Vec<Injection> {
    let table: [(&str, &'static str, Value); 6] = [
        (
            "@deepseek-ai/dsh-api-remote-access",
            "__DSH_REMOTE_ACCESS__",
            json!({"type": "baseline", "owner": true}),
        ),
        (
            "@deepseek-ai/dsh-client-connection",
            "__DSH_CONNECTION_RECOVERY__",
            json!({
                "backoffBaseMs": 500, "backoffFactor": 2, "backoffMaxMs": 10000,
                "generationReadyWarnMs": 3000, "generationReadyTimeoutMs": 15000
            }),
        ),
        (
            "@deepseek-ai/dsh-client-shortcuts",
            "__DSH_SHORTCUTS_CONFIG__",
            json!({"stopSequenceMs": 500}),
        ),
        (
            "@deepseek-ai/dsh-client-ui-sidebar-documentpreview",
            "__DSH_DOCUMENT_PREVIEW_CONFIG__",
            json!({
                "office": {"maxCachedEntries": 8, "maxCachedBytes": 67108864, "maxPending": 8, "maxReaders": 32},
                "excel": {"maxBytes": 16777216, "maxCells": 250000, "timeoutMs": 15000}
            }),
        ),
        (
            "@deepseek-ai/dsh-client-ui-settings-models",
            "__DSH_MODELS_ONBOARDING__",
            json!({"credentialOnboarding": true}),
        ),
        (
            "@deepseek-ai/dsh-client-ui-settings-account",
            "__DSH_CONTACT_CONFIG__",
            json!({
                "contactFormUrl": "https://trtgsjkv6r.feishu.cn/share/base/form/shrcnlCoGElW7MQznGy9r3YYXcg?hide_uid=1&hide_device_info=1&hide_harness_version=1",
                "contactSource": "", "bonusAckRetryDelayMs": 1000, "bonusAckRetryMaxDelayMs": 60000
            }),
        ),
    ];
    table
        .into_iter()
        .filter(|(package, _, _)| rows.contains_key(*package))
        .map(|(_, name, value)| Injection::Global(name, value))
        .collect()
}

fn injections(graph: &Value, batches: &[Batch], rows: &BTreeMap<String, Row>) -> Vec<Injection> {
    let mut out = Vec::new();
    if rows.contains_key("@deepseek-ai/dsh-client-ui-theme") {
        out.push(Injection::Style(THEME_STYLE.to_string()));
        out.push(Injection::ScriptBody(THEME_BODY_SCRIPT.to_string()));
    }
    out.extend(page_globals(rows));
    out.push(Injection::Script(queue_script()));
    for batch in batches.iter().filter(|b| b.phase == "application") {
        out.push(Injection::Preload(combo_reference(&batch.entries, &batch.rev, false)));
    }
    for batch in batches.iter().filter(|b| b.phase == "bootstrap") {
        out.push(Injection::ScriptSrc(combo_reference(&batch.entries, &batch.rev, false)));
    }
    out.push(Injection::Global("__DSH_BOOT__", graph.clone()));
    out
}

// ---------------------------------------------------------------- helpers

fn prepared_source(path: &Path) -> Result<String, String> {
    let mut source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let url_trailer = Regex::new(r"(?:\r?\n)?//# sourceURL=[^\r\n]+(?:\r?\n)?$").map_err(|e| e.to_string())?;
    let map_trailer = Regex::new(r"(?:\r?\n)?//# sourceMappingURL=[^\r\n]*(?:\r?\n)?$").map_err(|e| e.to_string())?;
    source = url_trailer.replace(&source, "").into_owned();
    source = map_trailer.replace(&source, "").into_owned();
    if !source.ends_with('\n') {
        source.push('\n');
    }
    Ok(source)
}

fn combo_source_map(rows: &[&Row], file: &str) -> String {
    let mut sections = Vec::new();
    let mut line = 0usize;
    for row in rows {
        let source = prepared_source(&row.bundle).unwrap_or_default();
        let section = fs::read_to_string(format!("{}.map", row.bundle.display()))
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .filter(|m| m.get("version") == Some(&json!(3)) && m.get("mappings").is_some_and(Value::is_string))
            .unwrap_or_else(|| json!({"version": 3, "sources": [format!("/plugins/{}/client.js", row.id)], "names": [], "mappings": ""}));
        sections.push(json!({"offset": {"line": line, "column": 0}, "map": section}));
        line += format!("{source};\n").matches('\n').count();
    }
    format!("{}\n", json!({"version": 3, "file": file, "sections": sections}))
}

fn short_hash(input: &str) -> String {
    // FNV-1a over two lanes; an opaque cache key, not a security primitive.
    let mut a: u64 = 0xcbf29ce484222325;
    let mut b: u64 = 0x84222325cbf29ce4;
    for byte in input.bytes() {
        a = (a ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        b = (b.rotate_left(5) ^ u64::from(byte)).wrapping_mul(0x9e3779b97f4a7c15);
    }
    let mut hex = format!("{a:016x}{b:016x}");
    hex.truncate(REV_LENGTH);
    hex
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn splice_after_open_tag(html: &str, tag: &str, markup: &str) -> String {
    if markup.is_empty() {
        return html.to_string();
    }
    let open = Regex::new(&format!(r"(?i)<{tag}(?:\s[^>]*)?>")).ok().and_then(|re| re.find(html));
    match open {
        Some(found) => format!("{}{markup}{}", &html[..found.end()], &html[found.end()..]),
        None if tag == "head" => format!("{markup}{html}"),
        None => format!("{html}{markup}"),
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(v) = input.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Experimental packages are named `…dsh-experimental-*` or `…dsh-client-ui-experimental-*`.
fn is_experimental(name: &str) -> bool {
    name.trim_start_matches("@deepseek-ai/dsh-")
        .split('-')
        .any(|part| part == "experimental")
}
