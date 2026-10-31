//! Settings-backed market indexes and bounded HTTP reads.

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::{json, Value};

use super::wire::{Failure, RpcResult};

/// The service key owned by the GUI plugin.
pub const PLUGIN_MARKET_SERVICE: &str = "pluginMarket";
/// Market methods, in inventory order.
pub const ENDPOINTS: &[&str] = &[
    "pluginMarket/listIndexes",
    "pluginMarket/readEntryDetail",
    "pluginMarket/search",
    "pluginMarket/setIndexes",
];

/// Settings-backed index configuration. Commits must be durable before returning.
pub trait IndexSettings: Send + Sync {
    /// Read current sources, including externally edited, unusable URLs.
    fn indexes(&self) -> Vec<String>;
    /// Commit the complete list through Settings, not a service-local shadow.
    fn set_indexes(&self, urls: Vec<String>) -> Result<(), Failure>;
}

/// The market's index list, kept in the settings store.
///
/// The TypeScript host keeps the list in a volatile plugin entry; this build
/// keeps it in the settings store's `plugin-market` namespace, so a commit is
/// durable before the call answers.
pub struct SettingsIndexes {
    store: std::sync::Arc<super::settings::SettingsStore>,
}

impl SettingsIndexes {
    /// Read and write the list in `store`.
    pub fn new(store: std::sync::Arc<super::settings::SettingsStore>) -> Self {
        Self { store }
    }
}

impl IndexSettings for SettingsIndexes {
    fn indexes(&self) -> Vec<String> {
        self.store
            .user(super::settings::MARKET_NAMESPACE)
            .and_then(|user| user.get("indexes").and_then(Value::as_array).cloned())
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn set_indexes(&self, urls: Vec<String>) -> Result<(), Failure> {
        let patch = serde_json::Map::from_iter([("indexes".to_string(), json!(urls))]);
        self.store
            .update(super::settings::MARKET_NAMESPACE, &patch, None)
            .map(|_| ())
    }
}

/// Market service backed by configured HTTP indexes.

pub struct PluginMarket {
    settings: Option<Arc<dyn IndexSettings>>,
    timeout_ms: u64,
    max_bytes: usize,
}

impl PluginMarket {
    /// Use the deployment's Settings adapter for index configuration.
    pub fn new(settings: Arc<dyn IndexSettings>) -> Self {
        Self {
            settings: Some(settings),
            ..Self::default()
        }
    }

    /// Override the HTTP deadline and body limit with the GUI configuration.
    pub fn with_fetch_limits(mut self, timeout_ms: u64, max_bytes: usize) -> Self {
        self.timeout_ms = timeout_ms.max(1);
        self.max_bytes = max_bytes.max(1);
        self
    }

    fn indexes(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.settings
            .as_ref()
            .map(|s| s.indexes())
            .unwrap_or_default()
            .into_iter()
            .filter(|url| seen.insert(url.clone()))
            .collect()
    }

    /// Dispatch market arguments, validating sources before fetching.
    pub fn dispatch(&self, method: &str, args: &Value) -> Option<RpcResult> {
        let url = args["url"].as_str().unwrap_or_default();
        Some(match method {
            "pluginMarket/listIndexes" => RpcResult::ok(json!(self
                .indexes()
                .iter()
                .map(|url| {
                    let mut value = json!({"url": url});
                    if let Err(error) = configured_url(url) {
                        value["error"] = json!(error.message);
                    }
                    value
                })
                .collect::<Vec<_>>())),
            "pluginMarket/setIndexes" => {
                let configured = self.indexes();
                let mut normalized = Vec::new();
                let mut seen = HashSet::new();
                for raw in args["urls"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    let url = if configured.iter().any(|url| url == raw) {
                        raw.to_string()
                    } else {
                        match normalize_url(raw) {
                            Ok(url) => url,
                            Err(error) => return Some(RpcResult::failed(error)),
                        }
                    };
                    let key = canonical_key(&url);
                    if !seen.insert(key) {
                        return Some(failed(
                            "invalid-index-url",
                            format!("Index source listed twice: {url}"),
                            json!({"url": url, "reason": "duplicate"}),
                        ));
                    }
                    normalized.push(url);
                }
                match &self.settings {
                    None => failed(
                        "settings-unavailable",
                        "Index sources require the settings service and a profile entry"
                            .to_string(),
                        json!({}),
                    ),
                    Some(settings) => match settings.set_indexes(normalized) {
                        Ok(()) => RpcResult::Empty,
                        Err(error) => RpcResult::failed(error),
                    },
                }
            }
            "pluginMarket/search" | "pluginMarket/readEntryDetail" => {
                if method == "pluginMarket/readEntryDetail" {
                    let language = args["language"].as_str().unwrap_or_default();
                    if !valid_language(language) {
                        return Some(failed(
                            "invalid-query",
                            format!("Not a language id: {language}"),
                            json!({"field": "language", "reason": "format"}),
                        ));
                    }
                }
                if method == "pluginMarket/search" {
                    let query = &args["query"];
                    let offset = query["offset"].as_f64().unwrap_or(-1.0);
                    let limit = query["limit"].as_f64().unwrap_or(-1.0);
                    let invalid = if !offset.is_finite()
                        || offset < 0.0
                        || offset.fract() != 0.0
                        || offset > 9_007_199_254_740_991.0
                    {
                        Some(("offset", "offset must be a non-negative integer"))
                    } else if !limit.is_finite()
                        || !(1.0..=100.0).contains(&limit)
                        || limit.fract() != 0.0
                    {
                        Some(("limit", "limit must be an integer from 1 to 100"))
                    } else {
                        None
                    };
                    if let Some((field, message)) = invalid {
                        return Some(failed(
                            "invalid-query",
                            message.to_string(),
                            json!({"field": field, "reason": "range"}),
                        ));
                    }
                }
                if !self.indexes().iter().any(|source| source == url) {
                    failed(
                        "unknown-index",
                        format!("Not a configured index source: {url}"),
                        json!({"url": url}),
                    )
                } else if let Err(error) = configured_url(url) {
                    failed(
                        "invalid-index-url",
                        error.message,
                        json!({"url": url, "reason": "configured"}),
                    )
                } else {
                    self.read_remote(method, args)
                }
            }
            _ => return None,
        })
    }
}

impl Default for PluginMarket {
    fn default() -> Self {
        Self {
            settings: None,
            timeout_ms: 15_000,
            max_bytes: 128 * 1024 * 1024,
        }
    }
}

impl PluginMarket {
    fn read_remote(&self, method: &str, args: &Value) -> RpcResult {
        let args = args.clone();
        let method = method.to_string();
        let timeout = self.timeout_ms;
        let max_bytes = self.max_bytes;
        // The gateway's market dispatch is synchronous. Do not nest a runtime on
        // its executor thread (including the current-thread test executor).
        std::thread::spawn(move || {
            #[cfg(feature = "openai")]
            {
                match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(remote(&method, &args, timeout, max_bytes)),
                    Err(error) => {
                        fetch_failure(args["url"].as_str().unwrap_or_default(), error.to_string())
                    }
                }
            }
            #[cfg(not(feature = "openai"))]
            {
                let _ = (method, timeout, max_bytes);
                fetch_failure(
                    args["url"].as_str().unwrap_or_default(),
                    "HTTP support is disabled".into(),
                )
            }
        })
        .join()
        .unwrap_or_else(|_| fetch_failure("", "market reader thread failed".into()))
    }
}

fn fetch_failure(url: &str, reason: String) -> RpcResult {
    failed(
        "fetch-failed",
        format!("Could not fetch {url}: {reason}"),
        json!({"url": url, "reason": reason}),
    )
}

#[cfg(feature = "openai")]
async fn body(client: &reqwest::Client, url: &str, max: usize) -> Result<Option<Vec<u8>>, String> {
    let mut response = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    let too_large = || format!("body exceeds {max} bytes");
    if response
        .content_length()
        .is_some_and(|size| size > max as u64)
    {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if chunk.len() > max.saturating_sub(bytes.len()) {
            return Err(too_large());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}

#[cfg(feature = "openai")]
async fn remote(method: &str, args: &Value, timeout: u64, max: usize) -> RpcResult {
    let url = args["url"].as_str().unwrap_or_default();
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(error) => return fetch_failure(url, error.to_string()),
    };
    let bytes = match body(&client, url, max).await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return fetch_failure(url, "HTTP 404".into()),
        Err(reason) => return fetch_failure(url, reason),
    };
    let index: Value = match serde_json::from_slice(&bytes) {
        Ok(index) => index,
        Err(error) => return fetch_failure(url, error.to_string()),
    };
    let entries = match index.as_array().or_else(|| index["plugins"].as_array()) {
        Some(entries)
            if entries.iter().all(|e| {
                e["name"].is_string()
                    && (e["version"].is_string()
                        || e["versions"].as_array().is_some_and(|v| {
                            !v.is_empty() && v.iter().all(|v| v["version"].is_string())
                        }))
            }) =>
        {
            entries
        }
        _ => return fetch_failure(url, "expected an index with plugin entries".into()),
    };
    if method == "pluginMarket/readEntryDetail" {
        let name = args["name"].as_str().unwrap_or_default();
        let version = args["version"].as_str().unwrap_or_default();
        let found = entries.iter().find_map(|entry| {
            if entry["name"] != name {
                return None;
            }
            if entry["version"] == version {
                return Some((entry, entry));
            }
            entry["versions"]
                .as_array()?
                .iter()
                .find(|v| v["version"] == version)
                .map(|v| (entry, v))
        });
        return match found {
            None => failed(
                "unknown-plugin",
                format!("{url} lists no {name}@{version}"),
                json!({"name": name, "version": version}),
            ),
            Some((entry, listed)) => {
                detail(&client, url, entry, listed, args, max.min(1024 * 1024)).await
            }
        };
    }
    let query = &args["query"];
    let text = query["text"].as_str().unwrap_or_default().to_lowercase();
    let matching: Vec<_> = entries
        .iter()
        .filter(|entry| {
            ["name", "title", "description", "tags", "author", "metadata"]
                .iter()
                .map(|key| entry[*key].to_string().to_lowercase())
                .collect::<Vec<_>>()
                .join(" ")
                .contains(&text)
        })
        .collect();
    let offset = query["offset"].as_u64().unwrap_or_default() as usize;
    let limit = query["limit"].as_u64().unwrap_or(10) as usize;
    let page: Vec<_> = matching
        .iter()
        .skip(offset)
        .take(limit)
        .map(|e| listing(e))
        .collect();
    let next = offset.saturating_add(page.len());
    let mut result = json!({"url": url, "title": index.get("title").cloned().unwrap_or(json!("")),
        "metadata": index.get("metadata").cloned().unwrap_or(json!({"categories": {}, "labels": {}, "sortKeys": {}})),
        "fetchedAt": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
        "entries": page, "total": matching.len(), "nextOffset": if next < matching.len() { Some(next) } else { None }});
    if let Some(description) = index.get("description") {
        result["description"] = description.clone();
    }
    RpcResult::ok(result)
}

#[cfg(feature = "openai")]
fn listing(entry: &Value) -> Value {
    let mut entry = entry.clone();
    if entry.get("metadata").is_none() {
        entry["metadata"] = json!({"keywords": [], "labels": [], "sortKeys": {}});
    }
    if !entry["versions"].is_array() {
        entry["versions"] = json!([{"version": entry["version"], "publishTimestamp": 0, "source": {"type": "npm"}, "engines": {}}]);
    }
    let name = entry["name"].as_str().unwrap_or_default().to_string();
    let versions = entry["versions"].as_array_mut().unwrap();
    for version in versions.iter_mut() {
        let source = &version["source"];
        let spec = if source["type"] == "git" {
            format!(
                "git+{}.git#{}{}",
                source["url"]
                    .as_str()
                    .unwrap_or_default()
                    .trim_end_matches(".git"),
                source["commit"].as_str().unwrap_or_default(),
                source["path"]
                    .as_str()
                    .map(|p| format!("&path:/{p}"))
                    .unwrap_or_default()
            )
        } else {
            format!("{name}@{}", version["version"].as_str().unwrap_or_default())
        };
        let mut install = json!({"spec": spec});
        if let Some(registry) = source.get("registry") {
            install["registry"] = registry.clone();
        }
        version["install"] = install;
        version["compatibility"] = json!({"compatible": true});
    }
    let default = versions
        .iter()
        .find(|v| !v["version"].as_str().unwrap_or_default().contains('-'))
        .or_else(|| versions.first())
        .map(|v| v["version"].clone());
    if let Some(version) = default {
        entry["defaultVersion"] = version;
    }
    entry["installation"] = json!({"status": "not-installed"});
    entry
}

fn valid_language(language: &str) -> bool {
    let mut parts = language.split('-');
    let first = parts.next().unwrap_or_default();
    (2..=8).contains(&first.len())
        && first.bytes().all(|c| c.is_ascii_alphabetic())
        && parts.all(|part| {
            (1..=8).contains(&part.len()) && part.bytes().all(|c| c.is_ascii_alphanumeric())
        })
}

fn failed(code: &str, message: String, details: Value) -> RpcResult {
    RpcResult::failed(Failure {
        code: format!("plugin-market/{code}"),
        message,
        details,
    })
}

fn invalid_url(text: &str, reason: &str, message: String) -> Failure {
    Failure {
        code: "plugin-market/invalid-index-url".to_string(),
        message,
        details: json!({"url": text, "reason": reason}),
    }
}

#[cfg(feature = "openai")]
fn canonical_key(text: &str) -> String {
    reqwest::Url::parse(text)
        .map(|url| url.to_string())
        .unwrap_or_else(|_| text.to_string())
}

#[cfg(not(feature = "openai"))]
fn canonical_key(text: &str) -> String {
    text.to_string()
}

#[cfg(feature = "openai")]
fn normalize_url(text: &str) -> Result<String, Failure> {
    let url = reqwest::Url::parse(text.trim())
        .map_err(|_| invalid_url(text, "not-absolute", format!("Not an absolute URL: {text}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid_url(
            text,
            "scheme",
            format!("Index URLs use http or https: {text}"),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(invalid_url(
            text,
            "credentials-or-fragment",
            format!("Index URLs carry no credentials or fragment: {text}"),
        ));
    }
    Ok(url.to_string())
}

#[cfg(not(feature = "openai"))]
fn normalize_url(text: &str) -> Result<String, Failure> {
    Err(invalid_url(
        text,
        "not-absolute",
        "dsh-rs: market URL validation requires the openai build feature".to_string(),
    ))
}

fn configured_url(text: &str) -> Result<(), Failure> {
    let normalized = normalize_url(text)?;
    if normalized != text {
        return Err(invalid_url(
            text,
            "configured",
            format!("Index URL is not in canonical form; use {normalized}"),
        ));
    }
    Ok(())
}

#[cfg(feature = "openai")]
async fn detail(
    client: &reqwest::Client,
    index_url: &str,
    entry: &Value,
    listed: &Value,
    args: &Value,
    max: usize,
) -> RpcResult {
    use sha2::{Digest, Sha256};
    let source = &listed["source"];
    let name = args["name"].as_str().unwrap_or_default();
    let version = args["version"].as_str().unwrap_or_default();
    let mut roots = Vec::new();
    let mut key = format!("npm:{name}@{version}");
    if source["type"] == "git" {
        let repo = source["url"]
            .as_str()
            .unwrap_or_default()
            .trim_end_matches(".git");
        let commit = source["commit"].as_str().unwrap_or("HEAD");
        key = format!("git:{repo}.git#{commit}");
        if let Some(project) = repo.strip_prefix("https://github.com/") {
            roots.push(
                format!(
                    "https://raw.githubusercontent.com/{project}/{commit}/{}/",
                    source["path"].as_str().unwrap_or_default()
                )
                .replace(&format!("{commit}//"), &format!("{commit}/")),
            );
        }
    } else if source["registry"].as_str().is_none_or(|r| {
        reqwest::Url::parse(r).is_ok_and(|u| {
            matches!(
                u.host_str(),
                Some("registry.npmjs.org" | "registry.npmmirror.com")
            )
        })
    }) {
        roots.push(format!("https://cdn.jsdelivr.net/npm/{name}@{version}/"));
        roots.push(format!("https://unpkg.com/{name}@{version}/"));
        roots.push(format!(
            "https://registry.npmmirror.com/{name}/{version}/files/"
        ));
    }
    // Flat indexes may carry their package files directly, avoiding another fetch.
    let files = listed.get("files").or_else(|| entry.get("files"));
    let read = |path: String| {
        let roots = &roots;
        async move {
            if let Some(files) = files {
                return Ok(files.get(&path).and_then(Value::as_str).map(str::to_string));
            }
            let mut last = None;
            for root in roots {
                let url = reqwest::Url::parse(root)
                    .and_then(|u| u.join(&path))
                    .map_err(|e| e.to_string())?;
                match body(client, url.as_str(), max).await {
                    Ok(Some(bytes)) => {
                        return Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
                    }
                    Ok(None) => return Ok(None),
                    Err(error) => last = Some(error),
                }
            }
            match last {
                Some(error) => Err(error),
                None => Ok(None),
            }
        }
    };
    let unavailable = |reason: &str, message: String| {
        failed("detail-unavailable", message, json!({"reason": reason}))
    };
    let found = |origin: &str, path: &str, markdown: String| {
        let markdown = markdown.trim_start_matches('\u{feff}');
        let frontmatter =
            regex::Regex::new(r"\A---\r?\n[\s\S]*?\r?\n---[ \t]*(?:\r?\n|$)").unwrap();
        let document = format!("{:x}", Sha256::digest(format!("{key}\0{path}").as_bytes()));
        RpcResult::ok(
            json!({"status": "found", "origin": origin, "markdown": frontmatter.replace(markdown, ""), "document": &document[..32]}),
        )
    };
    let language = args["language"].as_str().unwrap_or("en").to_lowercase();
    let parts: Vec<_> = language.split('-').collect();
    let mut languages: Vec<_> = (1..=parts.len())
        .rev()
        .map(|n| parts[..n].join("-"))
        .collect();
    if !languages.iter().any(|l| l == "en") {
        languages.push("en".into());
    }
    for language in languages {
        let path = format!("locale/{language}.json");
        let locale = match read(path.clone()).await {
            Ok(Some(text)) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => value,
                Err(error) => return unavailable("invalid", format!("{path}: {error}")),
            },
            Ok(None) => continue,
            Err(error) => return unavailable("network", error),
        };
        if let Some(value) = locale["meta"].get("detail") {
            let path = match value.as_str() {
                Some(path)
                    if !path.contains(['\\', ':'])
                        && path.split('/').all(|p| !matches!(p, "" | "." | ".."))
                        && (path.to_lowercase().ends_with(".md")
                            || path.to_lowercase().ends_with(".markdown")) =>
                {
                    path
                }
                _ => {
                    return unavailable(
                        "invalid",
                        "meta.detail must name a relative Markdown file".into(),
                    )
                }
            };
            return match read(path.to_string()).await {
                Ok(Some(text)) => found("locale", path, text),
                Ok(None) => unavailable(
                    "invalid",
                    format!("{path}, named by meta.detail, does not exist"),
                ),
                Err(error) => unavailable("network", error),
            };
        }
    }
    if let Some(markdown) = listed
        .get("readme")
        .or_else(|| entry.get("readme"))
        .and_then(Value::as_str)
    {
        return found("readme", "README.md", markdown.to_string());
    }
    for path in [
        "README.md",
        "readme.md",
        "Readme.md",
        "README.markdown",
        "README",
    ] {
        match read(path.to_string()).await {
            Ok(Some(text)) => return found("readme", path, text),
            Ok(None) => {}
            Err(error) => return unavailable("network", error),
        }
    }
    if let Some(repository) = entry["repository"]["url"].as_str() {
        if let Some(project) = repository
            .trim_end_matches(".git")
            .strip_prefix("https://github.com/")
        {
            for path in [
                "README.md",
                "readme.md",
                "Readme.md",
                "README.markdown",
                "README",
            ] {
                let dir = entry["repository"]["directory"]
                    .as_str()
                    .unwrap_or_default();
                let url = format!(
                    "https://raw.githubusercontent.com/{project}/HEAD/{}{path}",
                    if dir.is_empty() {
                        String::new()
                    } else {
                        format!("{dir}/")
                    }
                );
                match body(client, &url, max).await {
                    Ok(Some(bytes)) => {
                        return found(
                            "repository",
                            path,
                            String::from_utf8_lossy(&bytes).into_owned(),
                        )
                    }
                    Ok(None) => {}
                    Err(error) => return unavailable("network", error),
                }
            }
        }
    }
    let _ = index_url;
    RpcResult::ok(json!({"status": "none"}))
}
