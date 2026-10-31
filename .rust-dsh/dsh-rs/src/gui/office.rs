//! `officeToPdf/*`: the document preview's conversion.
//!
//! The TypeScript host converts an Office document to PDF with
//! `@deepseek-ai/libreoffice-kit`, which drives a bundled LibreOffice. This
//! build runs the same kit's command-line interface — the same converter, the
//! same failure categories — and answers the same shapes: the conversion's
//! bytes, the provider's generation, and the failure reasons the TypeScript
//! host reports.

use std::path::{Path, PathBuf};
use std::time::Duration;

use cordis::Context;
use serde_json::{json, Value};

use super::wire::{codes, Attachment, Failure, RpcResult};
use crate::api::services::AgentRegistryService;

/// The extensions the TypeScript host converts.
const CONVERTIBLE: [&str; 6] = ["doc", "docx", "xls", "xlsx", "ppt", "pptx"];

/// The failure categories the converter publishes.
const CATEGORIES: [&str; 9] = [
    "input-too-large",
    "output-too-large",
    "invalid-document",
    "unsupported-format",
    "invalid-output",
    "timeout",
    "unavailable",
    "failed",
    "busy",
];

/// The command that converts a document, and the limits it runs under.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OfficeConfig {
    /// The converter's command: the arguments before `convert --input … --output …`.
    /// Unset means this build converts nothing.
    pub command: Option<Vec<String>>,
    /// How long a conversion may take.
    pub timeout_ms: u64,
    /// The largest document that is converted.
    pub max_input_bytes: usize,
    /// The largest PDF that is returned.
    pub max_output_bytes: usize,
}

impl Default for OfficeConfig {
    fn default() -> Self {
        OfficeConfig {
            command: None,
            timeout_ms: 60_000,
            // The TypeScript host's limits.
            max_input_bytes: 50 * 1024 * 1024,
            max_output_bytes: 100 * 1024 * 1024,
        }
    }
}

impl OfficeConfig {
    /// Validate the limits before the plugin starts.
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout_ms == 0 {
            return Err("gui: officeConverter.timeoutMs must be positive".to_string());
        }
        if self.max_input_bytes == 0 || self.max_output_bytes == 0 {
            return Err("gui: officeConverter byte limits must be positive".to_string());
        }
        if let Some(command) = &self.command {
            if command.is_empty() {
                return Err("gui: officeConverter.command must name a program".to_string());
            }
        }
        Ok(())
    }
}

/// The office document service.
pub struct OfficeService {
    /// The provider's generation: it changes when the provider's configuration
    /// is replaced, so a client knows its cached previews are stale.
    generation: String,
    config: OfficeConfig,
}

impl OfficeService {
    /// A service with a fresh generation.
    pub fn new(config: OfficeConfig) -> Self {
        OfficeService { generation: uuid_v4(), config }
    }

    /// The provider's generation.
    pub fn generation(&self) -> &str {
        &self.generation
    }

    /// Convert `source` (in `directory`) and answer the bytes.
    async fn convert(&self, directory: &Path, source: &Path, name: &str) -> Result<(Vec<u8>, Vec<String>), Failure> {
        let Some(command) = self.config.command.clone() else {
            return Err(render_failure("unavailable", "Office conversion is not configured in this build"));
        };
        let output = directory.join("converted.pdf");
        let mut argv = command;
        argv.push("convert".to_string());
        argv.push("--input".to_string());
        argv.push(source.display().to_string());
        argv.push("--output".to_string());
        argv.push(output.display().to_string());
        let mut process = tokio::process::Command::new(&argv[0]);
        process.args(&argv[1..]).stdin(std::process::Stdio::null()).kill_on_drop(true);
        let result = tokio::time::timeout(Duration::from_millis(self.config.timeout_ms), process.output())
            .await
            .map_err(|_| render_failure("timeout", "The conversion did not finish in time"))?
            .map_err(|error| render_failure("failed", format!("the converter could not start: {error}")))?;
        if !result.status.success() {
            // The converter publishes its category on its standard error.
            let stderr = String::from_utf8_lossy(&result.stderr);
            let category = serde_json::from_str::<Value>(stderr.trim())
                .ok()
                .and_then(|value| value["code"].as_str().map(str::to_string))
                .filter(|code| CATEGORIES.contains(&code.as_str()))
                .unwrap_or_else(|| "failed".to_string());
            return Err(render_failure(&category, category_message(&category, name)));
        }
        let bytes = std::fs::read(&output)
            .map_err(|error| render_failure("failed", format!("the converter wrote no document: {error}")))?;
        if bytes.len() > self.config.max_output_bytes {
            return Err(render_failure("output-too-large", "The converted document is too large to return"));
        }
        // The converter reports the fonts it could not load.
        let missing_fonts = serde_json::from_str::<Value>(String::from_utf8_lossy(&result.stdout).trim())
            .ok()
            .and_then(|value| value["missingFonts"].as_array().cloned())
            .map(|fonts| fonts.into_iter().filter_map(|font| font.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        Ok((bytes, missing_fonts))
    }
}

/// The message a failure category carries.
fn category_message(category: &str, name: &str) -> String {
    match category {
        "invalid-document" => format!("{name} is not a document this converter can read."),
        "unsupported-format" => format!("{name} is not a document this converter can read."),
        "input-too-large" => "The document is too large to convert.".to_string(),
        "output-too-large" => "The converted document is too large to return.".to_string(),
        "timeout" => "The conversion did not finish in time.".to_string(),
        "unavailable" => "The converter is not available.".to_string(),
        _ => format!("The conversion of {name} failed."),
    }
}


/// The failure the TypeScript host answers for a conversion that failed.
///
/// The reason is the category; the message is the same for every one of them,
/// as in the TypeScript host.
fn render_failure(reason: &str, _detail: impl Into<String>) -> Failure {
    Failure {
        code: "document-render/failed".to_string(),
        message: "Office conversion failed.".to_string(),
        details: json!({ "reason": reason }),
    }
}

/// The extension of `path`, lower-cased.
fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_lowercase()
}

/// Serve the `officeToPdf/*` endpoints.
pub(super) async fn dispatch(
    ctx: &Context,
    service: Option<&OfficeService>,
    method: &str,
    payload: &Value,
) -> Option<RpcResult> {
    let name = method.strip_prefix("officeToPdf/")?;
    let Some(service) = service else {
        return Some(RpcResult::failed(Failure {
            code: codes::SERVICE_UNAVAILABLE.to_string(),
            message: "the office document service is not wired into this build".to_string(),
            details: json!({}),
        }));
    };
    Some(match name {
        "generation" => RpcResult::ok(Value::String(service.generation().to_string())),
        "render" => match render(ctx, service, payload).await {
            Ok(result) => result,
            Err(failure) => RpcResult::failed(failure),
        },
        _ => return None,
    })
}

/// `officeToPdf/render`: the document's PDF.
async fn render(ctx: &Context, service: &OfficeService, payload: &Value) -> Result<RpcResult, Failure> {
    let scope = payload["workspaceFileScopeId"].as_str().unwrap_or_default();
    let path = payload["path"].as_str().unwrap_or_default();
    let extension = extension(path);
    if !CONVERTIBLE.contains(&extension.as_str()) {
        return Err(render_failure("unsupported-format", "The path must end in doc, docx, xls, xlsx, ppt, or pptx."));
    }
    let Some(root) = workspace_root(ctx, scope) else {
        return Err(Failure {
            code: codes::LOOKUP_NOT_FOUND.to_string(),
            message: "typert gateway: officeToPdf/render: lookup provider \"workspaceFileScope\" did not resolve the requested identity".to_string(),
            details: json!({ "endpoint": "officeToPdf/render", "field": "workspaceFileScopeId" }),
        });
    };
    let source = root.join(path);
    let Ok(metadata) = std::fs::metadata(&source) else {
        return Err(Failure {
            code: "workspace-file/not-found".to_string(),
            message: format!("no entry at {path:?}"),
            details: json!({ "path": path }),
        });
    };
    if !metadata.is_file() {
        return Err(Failure {
            code: "workspace-file/not-found".to_string(),
            message: format!("no entry at {path:?}"),
            details: json!({ "path": path }),
        });
    }
    if metadata.len() > service.config.max_input_bytes as u64 {
        return Err(render_failure("input-too-large", "The document exceeds the reserved byte capacity."));
    }
    let directory = std::env::temp_dir().join(format!("dsh-office-{}", uuid_v4()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| render_failure("failed", format!("the working directory could not be made: {error}")))?;
    let result = service.convert(&directory, &source, path).await;
    let _ = std::fs::remove_dir_all(&directory);
    let (pdf, missing_fonts) = result?;
    // The document's bytes travel as a binary part of the reply, as the
    // TypeScript gateway sends them; the value's own field is null.
    // The path the TypeScript host reports is the canonical one.
    let absolute = std::fs::canonicalize(&source).unwrap_or_else(|_| source.clone());
    Ok(RpcResult::with_attachments(
        json!({
            "absolutePath": absolute.display().to_string(),
            "version": super::files::version_of(&metadata),
            "offset": 0,
            "eof": true,
            "bytes": pdf.len(),
            "data": Value::Null,
            "missingFonts": missing_fonts,
            "generation": service.generation(),
        }),
        vec![Attachment {
            path: vec!["data".to_string()],
            codec: "bytes".to_string(),
            part: "bytes-0".to_string(),
            bytes: pdf,
        }],
    ))
}

/// The workspace root of a session, when the session is known.
///
/// A session with no working directory of its own works in the host's
/// directory, as the TypeScript host's working-directory service does.
fn workspace_root(ctx: &Context, session: &str) -> Option<PathBuf> {
    if session.is_empty() {
        return None;
    }
    let agents = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)?;
    let agent = agents.get(session)?;
    let cwd = agent.session().header_cwd().map(PathBuf::from).or_else(|| std::env::current_dir().ok())?;
    Some(cwd)
}

/// A version-4 UUID.
fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}
