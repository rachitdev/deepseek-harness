//! `llm/discoverModels`: ask a provider endpoint which models it serves, for the
//! configuration page's "fetch available models" action.
//!
//! The request describes a draft the user is still editing. Nothing here is
//! stored: the reply is candidate metadata the page may offer for adoption.
//!
//! OpenAI-compatible protocols are read at `GET {baseURL}/models` with bearer
//! auth; Anthropic Messages at `GET {root}/v1/models` with `x-api-key`. The
//! reply is the standard `data` array or the enriched `models` map some gateways
//! expose. Every other protocol fails with a pointer to hand entry rather than
//! guessing its response fields.
//!
//! Differences from the TypeScript host, which this build cannot share:
//!
//! - TypeScript answers a route its bundled pi-ai catalog describes from that
//!   catalog, without a network call. This build has no such catalog, so a
//!   provider named without a `baseURL` is refused with a message saying so, and
//!   a request naming both is interrogated at its `baseURL`.
//! - There are no stored routes here, so the only credential and the only
//!   headers are the ones in the request.
//! - A redirect is not followed: it is reported as the status it answered, so
//!   a key typed for one host is never sent to another.
//! - Like the TypeScript host, the base URL is whatever the user typed: a
//!   loopback or private address is the usual case (a local gateway), and a key
//!   is sent over plain `http://` when the draft asks for it. The gateway's
//!   launch token is the only access control on who may ask.
//! - A request is bounded by `gui.llmDiscoveryTimeoutMs`; the TypeScript host
//!   bounds it only by the caller's cancellation.

use std::collections::HashSet;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Map, Value};

use super::router::{boundary, check_descriptor, Router};
use super::wire::{Failure, RpcResult};

const ENDPOINT: &str = "llm/discoverModels";
const REJECTED: &str = "llm/model-discovery-rejected";

/// The settings namespace whose discovery the TypeScript host registers, and
/// the only one this build serves.
pub const SETTINGS_NS: &str = "llm-pi-ai";

/// Protocols with a model listing this build can read.
const LISTABLE_PROTOCOLS: [&str; 3] = [
    "anthropic-messages",
    "openai-completions",
    "openai-responses",
];

/// A stable API version Anthropic's model-listing endpoint requires.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// The largest page Anthropic's public endpoint accepts. One page is read and
/// `has_more` is not followed.
const ANTHROPIC_MODEL_LIMIT: u32 = 1000;

/// Replies larger than this are refused, on the bytes actually read rather than
/// the length the server claims. A truncated listing cannot be parsed, so an
/// overflow rejects instead of truncating.
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

/// The `User-Agent` of probes, as the TypeScript host's attribution header.
const USER_AGENT: &str = "deepseek-harness/rs (+https://github.com/deepseek-ai/deepseek-harness)";

/// The draft endpoint a request describes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Draft {
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub api: Option<String>,
    pub api_key: Option<String>,
}

/// One model an endpoint advertises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub context_window: Option<u64>,
    pub max_tokens: Option<u64>,
}

impl Model {
    fn to_json(&self) -> Value {
        let mut out = Map::new();
        out.insert("id".into(), json!(self.id));
        out.insert("name".into(), json!(self.name));
        if let Some(window) = self.context_window {
            out.insert("contextWindow".into(), json!(window));
        }
        if let Some(max) = self.max_tokens {
            out.insert("maxTokens".into(), json!(max));
        }
        Value::Object(out)
    }
}

/// A probe that cannot list models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// What the page shows.
    pub message: String,
    /// What went wrong underneath, for the log; the page never sees it.
    pub cause: Option<String>,
}

impl Rejection {
    fn new(message: impl Into<String>) -> Self {
        Rejection {
            message: message.into(),
            cause: None,
        }
    }

    fn caused(message: impl Into<String>, cause: impl ToString) -> Self {
        Rejection {
            message: message.into(),
            cause: Some(cause.to_string()),
        }
    }
}

/// The HTTP client that interrogates endpoints.
pub struct Discovery {
    client: reqwest::Client,
}

impl Discovery {
    /// A discovery whose requests give up after `timeout`.
    pub fn new(timeout: Duration) -> Result<Self, String> {
        reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(USER_AGENT)
            // A redirect would carry `x-api-key` (reqwest strips only `Authorization`)
            // to whatever host the endpoint names, so a redirect is a refusal.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map(|client| Discovery { client })
            .map_err(|e| format!("cannot build the model discovery client: {e}"))
    }

    /// Interrogate `draft`'s endpoint for the models it advertises, in endpoint
    /// order and without duplicates.
    pub async fn probe(&self, draft: &Draft) -> Result<Vec<Model>, Rejection> {
        let base_url = match draft.base_url.as_deref() {
            Some(base) if !base.is_empty() => base,
            _ => {
                return Err(Rejection::new(format!(
                    "this build ships no model catalog for provider {:?}, so its models can only \
                     come from its endpoint; set a baseURL, or enter this provider's models by hand",
                    draft.provider.as_deref().unwrap_or("")
                )))
            }
        };
        // A draft with no protocol yet is asked as OpenAI Chat Completions, the
        // dialect a gateway most likely speaks; hand entry remains the way out.
        let api = draft.api.as_deref().unwrap_or("openai-completions");
        if !LISTABLE_PROTOCOLS.contains(&api) {
            return Err(Rejection::new(format!(
                "pi-ai protocol {api:?} has no model listing this build can read; enter this \
                 provider's models by hand"
            )));
        }
        let url = listing_url(base_url, api);
        let key = draft.api_key.as_deref().map(usable_key).transpose()?;
        let mut request = self
            .client
            .get(&url)
            .header(reqwest::header::ACCEPT, "application/json");
        request = match (api, key) {
            ("anthropic-messages", key) => {
                let request = request.header("anthropic-version", ANTHROPIC_VERSION);
                match key {
                    Some(key) => request.header("x-api-key", key),
                    None => request,
                }
            }
            (_, Some(key)) => request.bearer_auth(key),
            (_, None) => request,
        };
        let response = request
            .send()
            .await
            .map_err(|e| Rejection::caused(format!("could not reach {url}"), e))?;
        let status = response.status();
        if !status.is_success() {
            let hint = match status.as_u16() {
                401 | 403 => "; check the API key",
                300..=399 => "; redirects are not followed, so use the endpoint's final URL",
                _ => "",
            };
            return Err(Rejection::new(format!(
                "{url} answered {}{hint}",
                status.as_u16()
            )));
        }
        let text = read_bounded(response, &url).await?;
        let mut seen = HashSet::new();
        Ok(read_listing(&text, &url)?
            .into_iter()
            .filter(|model| seen.insert(model.id.clone()))
            .collect())
    }
}

/// A key as an HTTP header can carry it: trimmed, printable ASCII without
/// space. A fault in the key is reported as the key's, not as a network fault.
fn usable_key(raw: &str) -> Result<String, Rejection> {
    let key = raw.trim();
    if key.is_empty() {
        return Err(Rejection::new(
            "this provider's API key is blank; enter it on the Models page, or clear it to probe unauthenticated",
        ));
    }
    if !key.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
        return Err(Rejection::new(
            "this provider's API key contains characters no HTTP header can carry; paste the raw key only",
        ));
    }
    Ok(key.to_string())
}

/// Join the base with the protocol's listing path. The base is a prefix, so a
/// deployment path such as `https://gateway.example/openai/v1` keeps its
/// segments. Anthropic lists at `{root}/v1/models`, where the root is the base
/// without a trailing `/v1`: gateway documentation publishes both spellings.
fn listing_url(base_url: &str, api: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if api != "anthropic-messages" {
        return format!("{base}/models");
    }
    let root = base.strip_suffix("/v1").unwrap_or(base);
    format!("{root}/v1/models?limit={ANTHROPIC_MODEL_LIMIT}")
}

/// The reply body, refused once it outgrows the ceiling.
async fn read_bounded(response: reqwest::Response, url: &str) -> Result<Vec<u8>, Rejection> {
    let oversized = || {
        Rejection::new(format!(
            "{url} answered with more than {MAX_RESPONSE_BYTES} bytes"
        ))
    };
    if response
        .content_length()
        .is_some_and(|declared| declared > MAX_RESPONSE_BYTES as u64)
    {
        return Err(oversized());
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| Rejection::caused(format!("could not reach {url}"), e))?;
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err(oversized());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// A positive whole number from the first usable candidate.
fn capacity<'a>(candidates: impl IntoIterator<Item = Option<&'a Value>>) -> Option<u64> {
    candidates.into_iter().flatten().find_map(|value| {
        let number = value.as_f64()?;
        (number.fract() == 0.0 && number > 0.0 && number <= 9_007_199_254_740_991.0)
            .then_some(number as u64)
    })
}

/// The first non-empty string among the candidates.
fn label<'a>(candidates: impl IntoIterator<Item = Option<&'a Value>>) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|text| !text.is_empty())
        .map(str::to_string)
}

/// A JSON object read in document order. `serde_json::Value` sorts keys, and the
/// listing's order is the order the page offers the models in.
struct Ordered(Vec<(String, Value)>);

impl<'de> serde::Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visit;
        impl<'de> serde::de::Visitor<'de> for Visit {
            type Value = Ordered;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Ordered, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Value>()? {
                    entries.push(entry);
                }
                Ok(Ordered(entries))
            }
        }
        deserializer.deserialize_map(Visit)
    }
}

/// The `models` property: an object, or something else this reader skips.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ModelsProperty {
    Map(Ordered),
    Other(#[allow(dead_code)] Value),
}

/// The two properties a supported reply may carry its models in.
#[derive(serde::Deserialize)]
struct Listing {
    data: Option<Value>,
    models: Option<ModelsProperty>,
}

/// Read one supported model-listing reply. The standard `data` array wins when
/// both formats are present. In an enriched `models` map each property key is
/// the id the endpoint accepts and a nested `id` only fills an empty key. Rows
/// without a usable id are skipped, so one malformed row does not deny the user
/// the rest of a working catalog; a missing name falls back to the id.
fn read_listing(body: &[u8], url: &str) -> Result<Vec<Model>, Rejection> {
    let listing: Listing = serde_json::from_slice(body).map_err(|e| {
        // A reply that parses as JSON but is not an object has no listing; text
        // that is not JSON at all is a different fault.
        if serde_json::from_slice::<Value>(body).is_ok() {
            Rejection::new(NO_LISTING)
        } else {
            Rejection::caused(format!("{url} did not answer with JSON"), e)
        }
    })?;
    let rows: Vec<(Option<&str>, &Value)> = match (&listing.data, &listing.models) {
        (Some(Value::Array(data)), _) => data.iter().map(|row| (None, row)).collect(),
        (_, Some(ModelsProperty::Map(models))) => models
            .0
            .iter()
            .filter(|(_, row)| row.is_object())
            .map(|(key, row)| (Some(key.as_str()), row))
            .collect(),
        _ => return Err(Rejection::new(NO_LISTING)),
    };
    let mut models = Vec::new();
    for (key, row) in rows {
        let Some(entry) = row.as_object() else {
            continue;
        };
        let field = |name: &str| entry.get(name);
        let nested = |outer: &str, inner: &str| entry.get(outer).and_then(|v| v.get(inner));
        let key_value = key.map(|k| json!(k));
        let Some(id) = label([key_value.as_ref(), field("id")]) else {
            continue;
        };
        let name = label([field("name"), field("display_name"), field("displayName")])
            .unwrap_or_else(|| id.clone());
        models.push(Model {
            context_window: capacity([
                field("contextWindow"),
                field("context_window"),
                field("context_length"),
                field("max_input_tokens"),
                nested("limit", "context"),
            ]),
            max_tokens: capacity([
                field("maxOutputTokens"),
                field("max_output_tokens"),
                field("maxTokens"),
                field("max_tokens"),
                nested("limit", "output"),
                nested("top_provider", "max_completion_tokens"),
            ]),
            id,
            name,
        });
    }
    Ok(models)
}

const NO_LISTING: &str = "the endpoint's model listing has neither a \"data\" array nor a \
                          \"models\" object; enter this provider's models by hand";

/// Serve `llm/discoverModels`.
pub(super) async fn dispatch(
    router: &Router,
    ctx: &cordis::Context,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    if method != ENDPOINT {
        return None;
    }
    Some(match serve(router, ctx, payload).await {
        Ok(models) => RpcResult::ok(Value::Array(models.iter().map(Model::to_json).collect())),
        Err(failure) => RpcResult::failed(failure),
    })
}

async fn serve(
    router: &Router,
    ctx: &cordis::Context,
    payload: &Value,
) -> Result<Vec<Model>, Failure> {
    check_descriptor(ENDPOINT, &["settingsNs", "request"], payload)?;
    let settings_ns = payload["settingsNs"]
        .as_str()
        .ok_or_else(|| boundary(ENDPOINT, "settingsNs"))?;
    let draft = parse_draft(&payload["request"]).ok_or_else(|| boundary(ENDPOINT, "request"))?;
    let reject = |message: String| Failure {
        code: REJECTED.to_string(),
        message,
        details: match &draft.base_url {
            Some(base_url) => json!({ "settingsNs": settings_ns, "baseURL": base_url }),
            None => json!({ "settingsNs": settings_ns }),
        },
    };
    if settings_ns != SETTINGS_NS {
        return Err(reject(format!(
            "no model discovery is registered for {settings_ns:?}"
        )));
    }
    let named = |value: &Option<String>| value.as_deref().is_some_and(|text| !text.is_empty());
    if !named(&draft.provider) && !named(&draft.base_url) {
        return Err(reject(
            "model discovery needs a provider route or a baseURL".to_string(),
        ));
    }
    router.discovery.probe(&draft).await.map_err(|rejection| {
        if let Some(cause) = &rejection.cause {
            ctx.logger().log_event(
                cordis::logger::LogLevel::Warn,
                "gui".to_string(),
                None,
                format!("{ENDPOINT}: {}: {cause}", rejection.message),
            );
        }
        reject(rejection.message)
    })
}

/// The request object, or `None` when it is not one the wire type admits: an
/// object whose `provider`, `baseURL`, `api` and `apiKey` are strings when
/// present. Fields beyond those are ignored.
fn parse_draft(request: &Value) -> Option<Draft> {
    let object = request.as_object()?;
    let text = |name: &str| match object.get(name) {
        None => Some(None),
        Some(Value::String(value)) => Some(Some(value.clone())),
        Some(_) => None,
    };
    Some(Draft {
        provider: text("provider")?,
        base_url: text("baseURL")?,
        api: text("api")?,
        api_key: text("apiKey")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "http://h/models";

    fn read(body: &str) -> Vec<Model> {
        read_listing(body.as_bytes(), URL).unwrap()
    }

    fn ids(body: &str) -> Vec<String> {
        read(body).into_iter().map(|m| m.id).collect()
    }

    #[test]
    fn the_data_array_wins_over_a_models_map_and_unusable_rows_are_skipped() {
        let body = r#"{
            "data": [{ "id": "a" }, { "id": "" }, { "name": "no id" }, null, 7, "x", { "id": "b", "name": "B" }],
            "models": { "ignored": {} }
        }"#;
        assert_eq!(ids(body), ["a", "b"]);
    }

    #[test]
    fn a_models_map_keeps_the_endpoints_order_and_ignores_primitive_properties() {
        let body = r#"{ "models": {
            "zeta": { "id": "canonical", "display_name": "Alias" },
            "": { "id": "fallback" },
            "version": "2024-01-01",
            "list": [1],
            "nothing": null,
            "alpha": {}
        }}"#;
        let models = read(body);
        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            ["zeta", "fallback", "alpha"],
            "document order, not sorted"
        );
        assert_eq!(models[0].name, "Alias");
        assert_eq!(
            models[1].name, "fallback",
            "a missing name falls back to the id"
        );
    }

    #[test]
    fn capacities_come_from_the_first_positive_whole_number() {
        let body = r#"{ "data": [{
            "id": "m",
            "context_window": -5,
            "context_length": 8192.0,
            "max_input_tokens": 1,
            "max_tokens": "big",
            "limit": { "output": 4096 },
            "top_provider": { "max_completion_tokens": 1 }
        }, { "id": "n", "contextWindow": 1.5, "limit": null, "top_provider": null }]}"#;
        let models = read(body);
        assert_eq!(
            (models[0].context_window, models[0].max_tokens),
            (Some(8192), Some(4096))
        );
        assert_eq!(
            (models[1].context_window, models[1].max_tokens),
            (None, None)
        );
    }

    #[test]
    fn a_reply_with_no_listing_says_to_enter_models_by_hand() {
        for body in [
            "{}",
            "[]",
            "null",
            r#"{ "models": [] }"#,
            r#"{ "data": "x" }"#,
            "7",
        ] {
            let error = read_listing(body.as_bytes(), URL).unwrap_err();
            assert!(error.message.contains("neither a \"data\" array"), "{body}");
        }
    }

    #[test]
    fn text_that_is_not_json_is_reported_as_such_and_logged() {
        let error = read_listing(b"<html>", URL).unwrap_err();
        assert_eq!(error.message, "http://h/models did not answer with JSON");
        assert!(error.cause.is_some());
    }

    #[test]
    fn listing_urls_keep_the_deployment_path_and_normalise_the_anthropic_root() {
        assert_eq!(
            listing_url("https://g.example/openai/v1/", "openai-responses"),
            "https://g.example/openai/v1/models"
        );
        assert_eq!(
            listing_url("https://api.example", "anthropic-messages"),
            "https://api.example/v1/models?limit=1000"
        );
        assert_eq!(
            listing_url("https://api.example/v1//", "anthropic-messages"),
            "https://api.example/v1/models?limit=1000"
        );
        assert_eq!(
            listing_url("https://g.example/proxy", "anthropic-messages"),
            "https://g.example/proxy/v1/models?limit=1000"
        );
    }

    #[test]
    fn a_key_is_trimmed_and_must_be_printable_ascii() {
        assert_eq!(usable_key("  sk-abc \n").unwrap(), "sk-abc");
        assert!(usable_key("  ").unwrap_err().message.contains("blank"));
        for bad in ["a b", "caf\u{e9}", "tab\tkey", "k\u{7f}"] {
            assert!(
                usable_key(bad)
                    .unwrap_err()
                    .message
                    .contains("no HTTP header"),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_draft_needs_string_fields_only() {
        assert!(parse_draft(&json!({ "provider": "p", "extra": 1 })).is_some());
        for bad in [
            json!(null),
            json!([]),
            json!("x"),
            json!({ "provider": 5 }),
            json!({ "baseURL": null }),
            json!({ "api": [] }),
            json!({ "apiKey": 1 }),
        ] {
            assert!(parse_draft(&bad).is_none(), "{bad}");
        }
    }
}
