//! `commands/list` and `commands/execute`: the slash commands a session offers.
//!
//! The TypeScript host registers a command per plugin and logs each run as a
//! `command/run` / `command/done` pair around the handler. This build offers
//! the commands it can carry out, and lists exactly those: `/compact` (when the
//! compaction plugin is loaded), `/export` (when a session store is composed),
//! `/permission` (when the `permission` plugin is loaded), `/plan` (when the
//! `plan` plugin is loaded), `/feedback` and `/goal`. A line naming a command
//! that is not offered is answered as an unknown command.
//!
//! An unknown or malformed command line is answered with no value and nothing
//! is logged, as in TypeScript: the client reads that as "not a command".

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use cordis::Context;
use serde_json::{json, Value};

use super::goals::{self, Command as GoalCommand};
use super::router::{boundary, check_descriptor, require_session, unavailable, Router};
use super::wire::{Failure, RpcResult};
use crate::api::services::{AgentRegistryService, AgentView};
use crate::llm::{ContentBlock, MessageSource};
use crate::plugins::plan::{PlanMode, SetOutcome};
use crate::plugins::compaction::{
    compact_if_needed, CompactionConfig, CompactionRequest, Outcome, COMPACTION_SERVICE,
};
use crate::types::{AgentStatus, GoalPhase, Message, Role, SessionEventData};

const USAGE_GOAL: &str = "Usage: /goal [<objective>|clear|edit <objective>|pause|resume]";
const USAGE_COMPACT: &str = "Usage: /compact (no arguments)";
const USAGE_FEEDBACK: &str = "Usage: /feedback <text>";

/// One command a session offers.
struct Descriptor {
    definition_id: &'static str,
    name: &'static str,
    description: &'static str,
    /// Placeholder for the command's free-form input; none when it takes none.
    hint: Option<&'static str>,
    /// Whether composer attachments may accompany an invocation.
    attachments: bool,
    /// Whether the log keeps the text after the command name.
    record_input: bool,
}

/// Every command this build can run, by name.
const DESCRIPTORS: [Descriptor; 6] = [
    Descriptor {
        definition_id: "@deepseek-ai/dsh-command-compact",
        name: "compact",
        description: "Compact older conversation history",
        hint: None,
        attachments: false,
        record_input: true,
    },
    Descriptor {
        definition_id: "@deepseek-ai/dsh-session-log-export",
        name: "export",
        description: "Download this Session log as a ZIP archive",
        hint: None,
        attachments: false,
        record_input: true,
    },
    Descriptor {
        definition_id: "@deepseek-ai/dsh-command-feedback",
        name: "feedback",
        description: "Record feedback about this session",
        hint: Some("<text>"),
        attachments: false,
        record_input: false,
    },
    Descriptor {
        definition_id: "@deepseek-ai/dsh-command-goal",
        name: "goal",
        description: "Set or view the goal for a long-running task",
        hint: Some("[<objective>|clear|edit <objective>|pause|resume]"),
        attachments: true,
        record_input: true,
    },
    Descriptor {
        definition_id: "@deepseek-ai/dsh-permission-presets",
        name: "permission",
        description: "Switch the permission preset (sandbox mode + approval policy)",
        hint: Some("<preset>"),
        attachments: false,
        record_input: true,
    },
    Descriptor {
        definition_id: "@deepseek-ai/dsh-plan-mode",
        name: "plan",
        description: "Enter or leave plan mode",
        hint: Some("[off|message]"),
        attachments: true,
        record_input: true,
    },
];

impl Descriptor {
    fn wire(&self) -> Value {
        let mut item = json!({
            "definitionId": self.definition_id,
            "name": self.name,
            "description": self.description,
        });
        if let Some(hint) = self.hint {
            let mut input = json!({ "hint": hint });
            if self.attachments {
                input["attachments"] = json!(true);
            }
            item["input"] = input;
        }
        item
    }
}

/// The commands this deployment can run.
fn available(ctx: &Context) -> Vec<&'static Descriptor> {
    let compaction = ctx
        .get::<Arc<CompactionConfig>>(COMPACTION_SERVICE)
        .is_some();
    let sessions_stored = super::router::persistence(ctx).is_some();
    DESCRIPTORS
        .iter()
        .filter(|descriptor| match descriptor.name {
            "compact" => compaction,
            "export" => sessions_stored,
            "permission" => crate::permission::provided(ctx).is_some(),
            "plan" => ctx
                .get::<PlanMode>(crate::plugins::plan::PLAN_SERVICE)
                .is_some(),
            _ => true,
        })
        .collect()
}

/// A command line split into its name and the text after it.
#[derive(Debug, PartialEq, Eq)]
struct Parsed<'a> {
    name: &'a str,
    /// Everything after the name, the separating whitespace included.
    raw_input: &'a str,
}

/// Split `/name rest`. The name is lower case, starts with a letter, and ends
/// at the end of the line or at a tab, newline, carriage return or space.
fn parse_command(line: &str) -> Option<Parsed<'_>> {
    let body = line.strip_prefix('/')?;
    if !body.starts_with(|c: char| c.is_ascii_lowercase()) {
        return None;
    }
    let end = body
        .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'))
        .unwrap_or(body.len());
    let rest = &body[end..];
    if !rest.is_empty() && !rest.starts_with(['\t', '\n', '\r', ' ']) {
        return None;
    }
    Some(Parsed {
        name: &body[..end],
        raw_input: rest,
    })
}

/// JavaScript's whitespace: Unicode white space and the byte-order mark, but
/// not U+0085, which `String.prototype.trim` keeps.
fn is_js_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

fn js_trim(text: &str) -> &str {
    text.trim_matches(is_js_space)
}

/// What a handler reports to the human.
struct Reply {
    success: bool,
    text: Option<String>,
    /// The earlier event that holds a richer presentation of the result.
    source_event_seq: Option<u64>,
}

impl Reply {
    fn success(text: impl Into<String>) -> Self {
        Reply {
            success: true,
            text: Some(text.into()),
            source_event_seq: None,
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Reply {
            success: false,
            text: Some(text.into()),
            source_event_seq: None,
        }
    }
}

/// An attachment as the wire carries it.
enum Submitted {
    Image(super::images::WireImage),
    File(String),
}

/// An admitted attachment, in submission order.
type Admitted = Vec<ContentBlock>;

/// This process's command state.
pub struct Commands {
    /// Identifies this process in the ids it mints.
    token: String,
    counter: AtomicU64,
    anonymous_user: AnonymousUser,
    /// Sessions with a `/compact` in flight.
    compacting: Mutex<HashSet<String>>,
}

impl Commands {
    pub fn new(anonymous_user_file: Option<PathBuf>) -> Self {
        Commands {
            token: super::plugin::launch_token()[..8].to_string(),
            counter: AtomicU64::new(1),
            anonymous_user: AnonymousUser::new(anonymous_user_file),
            compacting: Mutex::new(HashSet::new()),
        }
    }

    fn mint_id(&self) -> String {
        format!(
            "cmd-{}-{}",
            self.token,
            self.counter.fetch_add(1, Ordering::Relaxed)
        )
    }
}

/// File name of the anonymous user id inside the harness home, as in the
/// TypeScript host (`ANONYMOUS_USER_ID_FILE_NAME`).
const ANONYMOUS_USER_ID_FILE: &str = ".anonymous-user-id";

/// The harness home: `$DSH_HOME` when that is set and not blank, else `~/.dsh`;
/// a leading `~/` expands to the home directory. `None` when neither
/// `DSH_HOME` nor a home directory is known, or the result is not absolute.
pub fn harness_home(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let home = env("HOME").filter(|home| !home.is_empty());
    let selected = match env("DSH_HOME").filter(|value| !value.trim().is_empty()) {
        Some(configured) => match configured.strip_prefix("~/") {
            Some(rest) => PathBuf::from(home?).join(rest),
            None if configured == "~" => PathBuf::from(home?),
            None => PathBuf::from(configured),
        },
        None => PathBuf::from(home?).join(".dsh"),
    };
    selected.is_absolute().then_some(selected)
}

/// Where the TypeScript host keeps the anonymous user id: `.anonymous-user-id`
/// in the harness home. `None` when no home is known, so the id lasts for the
/// process.
pub(super) fn default_anonymous_user_file(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    harness_home(env).map(|home| home.join(ANONYMOUS_USER_ID_FILE))
}

/// The id `/feedback` reports. It is read from, or written to, one file (the
/// TypeScript host's own id file unless the deployment names another), and lasts
/// for the process when no file is known. An id already in the file is never
/// replaced.
struct AnonymousUser {
    file: Option<PathBuf>,
    id: OnceLock<String>,
}

impl AnonymousUser {
    fn new(file: Option<PathBuf>) -> Self {
        AnonymousUser {
            file,
            id: OnceLock::new(),
        }
    }

    /// A persistence problem is reported to the log and the run keeps a usable
    /// id, so feedback is never blocked by an unwritable file.
    fn get(&self, ctx: &Context) -> String {
        self.id
            .get_or_init(|| {
                let Some(file) = &self.file else {
                    return super::plugin::uuid_v4();
                };
                match read_uuid(file) {
                    Some(id) => id,
                    None => {
                        let id = super::plugin::uuid_v4();
                        match persist_new(file, &id) {
                            Ok(()) => id,
                            Err(error) => {
                                // A concurrent first launch may have written one first.
                                read_uuid(file).unwrap_or_else(|| {
                                    ctx.logger().log_event(
                                        cordis::logger::LogLevel::Warn,
                                        "gui".to_string(),
                                        None,
                                        format!(
                                            "cannot persist the anonymous user id to {}: {error}",
                                            file.display()
                                        ),
                                    );
                                    id
                                })
                            }
                        }
                    }
                }
            })
            .clone()
    }
}

fn is_uuid(text: &str) -> bool {
    let groups: Vec<&str> = text.split('-').collect();
    groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(group, len)| group.len() == len && group.chars().all(|c| c.is_ascii_hexdigit()))
}

fn read_uuid(file: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    let id = js_trim(&text);
    is_uuid(id).then(|| id.to_string())
}

/// Publish `id` as the file's whole content, or fail if the file exists. The
/// id is written and synced in a private file first and then linked into
/// place, so a concurrent reader never sees an empty or partial id.
fn persist_new(file: &std::path::Path, id: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let staging = file.with_extension(format!("{}.tmp", super::plugin::uuid_v4()));
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)?;
    let written = writeln!(out, "{id}").and_then(|()| out.sync_all());
    let linked = written.and_then(|()| std::fs::hard_link(&staging, file));
    std::fs::remove_file(&staging)?;
    linked
}

/// Serve `commands/list` and `commands/execute`.
pub(super) async fn serve(
    router: &Router,
    ctx: &Context,
    endpoint: &str,
    payload: &Value,
) -> RpcResult {
    let result = match endpoint {
        "commands/list" => list(ctx, payload),
        _ => execute(router, ctx, payload).await,
    };
    result.unwrap_or_else(RpcResult::failed)
}

fn agent_id<'a>(endpoint: &str, payload: &'a Value) -> Result<&'a str, Failure> {
    payload["agentId"]
        .as_str()
        .ok_or_else(|| boundary(endpoint, "agentId"))
}

fn list(ctx: &Context, payload: &Value) -> Result<RpcResult, Failure> {
    const ENDPOINT: &str = "commands/list";
    check_descriptor(ENDPOINT, &["agentId"], payload)?;
    require_session(ctx, ENDPOINT, agent_id(ENDPOINT, payload)?)?;
    Ok(RpcResult::ok(Value::Array(
        available(ctx).iter().map(|d| d.wire()).collect(),
    )))
}

/// The attachments of a request, in wire order.
fn submitted(payload: &Value) -> Result<Vec<Submitted>, Failure> {
    const ENDPOINT: &str = "commands/execute";
    let bad = || boundary(ENDPOINT, "submittedAttachments");
    let items = payload["submittedAttachments"].as_array().ok_or_else(bad)?;
    items
        .iter()
        .map(|item| {
            let text = |key: &str| item.get(key).and_then(Value::as_str);
            match text("type") {
                Some("image") => {
                    let media_type = text("mediaType")
                        .filter(|kind| crate::images::MEDIA_TYPES.contains(kind))
                        .ok_or_else(bad)?;
                    let data = text("data").ok_or_else(bad)?;
                    let name = match item.get("name") {
                        None => None,
                        Some(Value::String(name)) => Some(name.clone()),
                        Some(_) => return Err(bad()),
                    };
                    Ok(Submitted::Image(super::images::WireImage {
                        media_type: media_type.to_string(),
                        data: data.to_string(),
                        name,
                    }))
                }
                Some("file") => Ok(Submitted::File(
                    text("receiptId").ok_or_else(bad)?.to_string(),
                )),
                _ => Err(bad()),
            }
        })
        .collect()
}

async fn execute(router: &Router, ctx: &Context, payload: &Value) -> Result<RpcResult, Failure> {
    const ENDPOINT: &str = "commands/execute";
    check_descriptor(
        ENDPOINT,
        &["agentId", "line", "submittedAttachments"],
        payload,
    )?;
    let session_id = agent_id(ENDPOINT, payload)?;
    let line = payload["line"]
        .as_str()
        .ok_or_else(|| boundary(ENDPOINT, "line"))?;
    let attachments = submitted(payload)?;
    require_session(ctx, ENDPOINT, session_id)?;
    let Some(agents) = ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE) else {
        return Ok(unavailable(ENDPOINT, "agents"));
    };
    let Some(agent) = agents.get(session_id) else {
        return Ok(super::router::session_not_found(session_id));
    };
    let Some(parsed) = parse_command(line) else {
        return Ok(RpcResult::Empty);
    };
    let Some(descriptor) = available(ctx).into_iter().find(|d| d.name == parsed.name) else {
        return Ok(RpcResult::Empty);
    };

    let commands = &router.commands;
    let command_id = commands.mint_id();
    let session = agent.session();
    session.append(SessionEventData::CommandRun {
        command_id: command_id.clone(),
        name: parsed.name.to_string(),
        args: descriptor
            .record_input
            .then(|| parsed.raw_input.to_string()),
    });
    let settle = |outcome: Reply| {
        session.append(SessionEventData::CommandDone {
            command_id: command_id.clone(),
            success: outcome.success,
            text: outcome.text.clone(),
            source_event_seq: outcome.source_event_seq.filter(|_| outcome.success),
        });
        outcome
    };

    let outcome = match run(router, ctx, &agent, descriptor, &parsed, attachments).await {
        Ok(outcome) => settle(outcome),
        // A handler that fails for a reason of its own is logged as an error
        // result and reported to the caller as the failure it is.
        Err(failure) => {
            settle(Reply::error(failure.message.clone()));
            // The handler's failure stays the reported one; a second failure
            // to save the log is reported beside it.
            if let Err(saving) =
                super::feedback::make_durable(ctx, ENDPOINT, &*agent, "the command log").await
            {
                ctx.logger().log_event(
                    cordis::logger::LogLevel::Warn,
                    "gui".to_string(),
                    None,
                    format!("{}: also failed: {}", failure.message, saving.message),
                );
            }
            return Err(failure);
        }
    };
    super::feedback::make_durable(ctx, ENDPOINT, &*agent, "the command log").await?;
    let mut result = json!({ "kind": if outcome.success { "success" } else { "error" } });
    if let Some(text) = &outcome.text {
        result["text"] = json!(text);
    }
    if let Some(seq) = outcome.source_event_seq.filter(|_| outcome.success) {
        result["sourceEventSeq"] = json!(seq);
    }
    Ok(RpcResult::ok(
        json!({ "commandId": command_id, "result": result }),
    ))
}

/// Admit the attachments, then run the command's handler.
async fn run(
    router: &Router,
    ctx: &Context,
    agent: &Arc<dyn AgentView>,
    descriptor: &Descriptor,
    parsed: &Parsed<'_>,
    attachments: Vec<Submitted>,
) -> Result<Reply, Failure> {
    let name = descriptor.name;
    let images = router.images();
    let _serialized;
    let mut admitted = Admitted::new();
    if !attachments.is_empty() {
        if !descriptor.attachments {
            return Ok(Reply::error(format!("/{name} does not accept attachments")));
        }
        if !images.has_store() {
            return Ok(Reply::error(format!(
                "/{name}: attachments are unavailable because no attachment store is composed"
            )));
        }
        // Image admissions for one session run one at a time, as for prompts.
        _serialized = images.serialize(agent.id()).await;
        match admit(router, ctx, agent.id(), attachments).await {
            Ok(blocks) => admitted = blocks,
            Err(message) => return Ok(Reply::error(message)),
        }
    }
    match name {
        "compact" => compact(router, ctx, agent, parsed.raw_input).await,
        "export" => Ok(export(parsed.raw_input)),
        "feedback" => feedback(router, ctx, agent, parsed.raw_input),
        "goal" => goal(router, ctx, agent, parsed.raw_input, admitted),
        "permission" => permission(ctx, agent, parsed.raw_input),
        "plan" => plan(ctx, agent, parsed.raw_input, admitted),
        other => unreachable!("{other} is listed but has no handler"),
    }
}

/// Resolve file receipts, then store the images, keeping submission order.
/// The error is the text shown to the human.
async fn admit(
    router: &Router,
    ctx: &Context,
    session_id: &str,
    attachments: Vec<Submitted>,
) -> Result<Admitted, String> {
    let uploads = router.uploads();
    let mut files = std::collections::HashMap::new();
    for item in &attachments {
        if let Submitted::File(receipt) = item {
            if files.contains_key(receipt) {
                continue;
            }
            let Some(file) = uploads.resolve(session_id, receipt) else {
                return Err("File upload receipt is unknown for this session.".to_string());
            };
            files.insert(receipt.clone(), file);
        }
    }
    let wire: Vec<super::images::WireImage> = attachments
        .iter()
        .filter_map(|item| match item {
            Submitted::Image(image) => Some(image.clone()),
            Submitted::File(_) => None,
        })
        .collect();
    let mut stored = if wire.is_empty() {
        Vec::new()
    } else {
        let policy = router.images();
        match tokio::task::spawn_blocking(move || policy.admit(&wire)).await {
            Ok(Ok(refs)) => refs,
            Ok(Err(error)) => {
                if let Some(cause) = &error.cause {
                    ctx.logger().log_event(
                        cordis::logger::LogLevel::Warn,
                        "gui".to_string(),
                        None,
                        format!("image admission: {}: {cause}", error.code),
                    );
                }
                return Err(error.message);
            }
            Err(error) => return Err(format!("image admission failed: {error}")),
        }
    }
    .into_iter();
    Ok(attachments
        .into_iter()
        .map(|item| match item {
            Submitted::Image(_) => ContentBlock::Image {
                attachment: stored.next().expect("one stored image per submitted image"),
            },
            Submitted::File(receipt) => ContentBlock::File {
                attachment: files[&receipt].clone(),
            },
        })
        .collect())
}

fn feedback(
    router: &Router,
    ctx: &Context,
    agent: &Arc<dyn AgentView>,
    raw_input: &str,
) -> Result<Reply, Failure> {
    let text = js_trim(raw_input);
    if text.is_empty() {
        return Ok(Reply::error(format!(
            "Feedback text is required. {USAGE_FEEDBACK}"
        )));
    }
    agent.session().append(SessionEventData::FeedbackRecord {
        text: Some(text.to_string()),
        category: None,
    });
    Ok(Reply::success(format!(
        "Feedback recorded for session {}\nAnonymous user: {}.",
        agent.id(),
        router.commands.anonymous_user.get(ctx)
    )))
}

/// Marks a session as compacting until dropped, including when the request is
/// cancelled mid-flight.
struct CompactGuard<'a> {
    commands: &'a Commands,
    session_id: String,
}

impl<'a> CompactGuard<'a> {
    fn enter(commands: &'a Commands, session_id: &str) -> Option<Self> {
        let entered = commands
            .compacting
            .lock()
            .unwrap()
            .insert(session_id.to_string());
        entered.then(|| CompactGuard {
            commands,
            session_id: session_id.to_string(),
        })
    }
}

impl Drop for CompactGuard<'_> {
    fn drop(&mut self) {
        self.commands
            .compacting
            .lock()
            .unwrap()
            .remove(&self.session_id);
    }
}

/// `/export`: the browser starts the download once this succeeds; the host
/// only validates the line.
fn export(raw_input: &str) -> Reply {
    if js_trim(raw_input).is_empty() {
        Reply::success("Session log download requested.")
    } else {
        Reply::error("The Web /export command does not accept a path.")
    }
}

async fn compact(
    router: &Router,
    ctx: &Context,
    agent: &Arc<dyn AgentView>,
    raw_input: &str,
) -> Result<Reply, Failure> {
    const BUSY: &str = "Compaction is unavailable because this process has an active compaction, or the agent is not idle.";
    if !js_trim(raw_input).is_empty() {
        return Ok(Reply::error(USAGE_COMPACT));
    }
    let Some(config) = ctx.get::<Arc<CompactionConfig>>(COMPACTION_SERVICE) else {
        return Ok(Reply::error("Compaction is not available in this build."));
    };
    if agent.status() != AgentStatus::Idle || agent.driver_busy() {
        return Ok(Reply::error(BUSY));
    }
    let Some(_in_flight) = CompactGuard::enter(&router.commands, agent.id()) else {
        return Ok(Reply::error(BUSY));
    };
    let request = CompactionRequest {
        force: true,
        focus: None,
    };
    let result = compact_if_needed(ctx, agent.id(), &config, &request).await;
    Ok(match result {
        Ok(Outcome::Skipped) => Reply::success("No compactable history yet."),
        Ok(Outcome::Compacted {
            replaced_tokens,
            replaced_items,
            summary_seq,
            ..
        }) => Reply {
            success: true,
            text: Some(format!(
                "Compacted {replaced_items} history items (~{replaced_tokens} tokens)."
            )),
            source_event_seq: Some(summary_seq),
        },
        Err(reason) => {
            ctx.logger().log_event(
                cordis::logger::LogLevel::Warn,
                "compaction".to_string(),
                Some("SKIPPED".to_string()),
                format!("agent {}: /compact did not compact ({reason})", agent.id()),
            );
            Reply::error(format!("Compaction failed: {reason}"))
        }
    })
}

/// A parsed `/goal` input.
#[derive(Debug, PartialEq, Eq)]
enum Goal<'a> {
    Show,
    Create(&'a str),
    Edit(&'a str),
    InvalidEdit,
    Pause,
    Resume,
    Clear,
}

fn parse_goal(raw_input: &str) -> Goal<'_> {
    let input = js_trim(raw_input);
    if input.is_empty() {
        return Goal::Show;
    }
    for (word, goal) in [
        ("clear", Goal::Clear),
        ("pause", Goal::Pause),
        ("resume", Goal::Resume),
        ("edit", Goal::InvalidEdit),
    ] {
        if input.eq_ignore_ascii_case(word) {
            return goal;
        }
    }
    let mut chars = input.chars();
    let head: String = chars.by_ref().take(4).collect();
    if head.eq_ignore_ascii_case("edit") && chars.next().is_some_and(is_js_space) {
        return Goal::Edit(js_trim(&input[4..]));
    }
    Goal::Create(input)
}

fn goal(
    router: &Router,
    ctx: &Context,
    agent: &Arc<dyn AgentView>,
    raw_input: &str,
    attachments: Admitted,
) -> Result<Reply, Failure> {
    let command = parse_goal(raw_input);
    if !attachments.is_empty() && !matches!(command, Goal::Create(_) | Goal::Edit(_)) {
        return Ok(Reply::error(
            "Attachments only accompany a goal objective: /goal <objective> or /goal edit <objective>.",
        ));
    }
    let session = agent.session();
    // The read below and the write in `goals::run` are not one step: a
    // `goals/*` call can land between them. `goals::apply` re-checks under its
    // own lock (a create refuses while a goal is live, an edit or pause names
    // the revision it read), so the loser is refused rather than overwriting.
    let current = goals::current_view(router, agent.id(), &session.events());
    let missing = |action: &str| {
        Ok(Reply::error(format!(
            "No goal is currently set; /goal {action} requires one. {USAGE_GOAL}"
        )))
    };
    let run = |command: GoalCommand| goals::run(router, ctx, agent.id(), command);
    // The domain refusing an operation reads as one stock message; any other
    // failure is the log write, which the caller must hear about.
    let refused = |failure: Failure| {
        if goals::is_domain_refusal(&failure) {
            Ok(Reply::error(
                "The goal command is not valid for the current state. Run /goal to view available commands.",
            ))
        } else {
            Err(failure)
        }
    };
    // A `goals/*` call can change the goal between the write and this read.
    let rendered =
        |title: &str| match goals::current_view(router, agent.id(), &agent.session().events()) {
            Some(view) => Reply::success(render_goal(title, &view)),
            None => Reply::error("The goal changed while the command ran. Run /goal to see it."),
        };
    let attach = || {
        if !attachments.is_empty() {
            agent.followup(attachment_message(attachments.clone()));
        }
    };
    match command {
        Goal::Show => Ok(match current {
            None => Reply::success(format!("No goal is currently set.\n{USAGE_GOAL}")),
            Some(view) => Reply::success(render_goal("Goal", &view)),
        }),
        Goal::InvalidEdit => Ok(Reply::error(format!(
            "Goal editing requires a replacement objective.\n{USAGE_GOAL}"
        ))),
        Goal::Create(objective) => {
            if let Some(view) = current
                .as_ref()
                .filter(|v| v.goal.phase != GoalPhase::Complete)
            {
                return Ok(Reply::error(format!(
                    "A goal is already {}. Use /goal edit <objective> to change it or /goal clear before replacing it.",
                    view.goal.phase.as_str()
                )));
            }
            match run(GoalCommand::Create(objective.to_string())) {
                Ok(()) => {
                    attach();
                    Ok(rendered("Goal created"))
                }
                Err(failure) => refused(failure),
            }
        }
        Goal::Edit(objective) => {
            let Some(view) = current else {
                return missing("edit");
            };
            let (command, title) = if view.goal.phase == GoalPhase::Complete {
                (GoalCommand::Create(objective.to_string()), "Goal created")
            } else {
                (
                    GoalCommand::Edit(goal_ref(&view), objective.to_string()),
                    "Goal updated",
                )
            };
            match run(command) {
                Ok(()) => {
                    attach();
                    Ok(rendered(title))
                }
                Err(failure) => refused(failure),
            }
        }
        Goal::Pause => {
            let Some(view) = current else {
                return missing("pause");
            };
            match run(GoalCommand::Pause(goal_ref(&view))) {
                Ok(()) => Ok(rendered("Goal paused")),
                Err(failure) => refused(failure),
            }
        }
        Goal::Resume => {
            let Some(view) = current else {
                return missing("resume");
            };
            match run(GoalCommand::Resume(goal_ref(&view))) {
                Ok(()) => Ok(rendered("Goal resumed")),
                Err(failure) => refused(failure),
            }
        }
        Goal::Clear => {
            let Some(view) = current else {
                return Ok(Reply::success("No goal to clear."));
            };
            match run(GoalCommand::Clear(goal_ref(&view))) {
                Ok(()) => Ok(Reply::success("Goal cleared.")),
                Err(failure) => refused(failure),
            }
        }
    }
}

fn goal_ref(view: &goals::View) -> crate::types::GoalRef {
    crate::types::GoalRef {
        id: view.goal.id.clone(),
        revision: view.goal.revision,
    }
}

/// The follow-up that carries a goal's reference attachments to the model.
fn attachment_message(mut attachments: Admitted) -> Message {
    attachments.push(ContentBlock::text(
        "Reference attachments for the goal objective.",
    ));
    Message {
        id: format!("gui-{}", crate::plugins::unique_id()),
        role: Role::User,
        content: attachments,
        source: MessageSource::User { rpc_id: None },
    }
}

/// `/permission [<preset>]`: show the preset in force, or switch to one.
fn permission(ctx: &Context, agent: &Arc<dyn AgentView>, raw_input: &str) -> Result<Reply, Failure> {
    let Some(table) = crate::permission::provided(ctx) else {
        return Ok(Reply::error("permission presets are not composed"));
    };
    let name = raw_input.trim();
    let names = table.names().join(", ");
    let session = agent.session();
    if name.is_empty() {
        let current = table.current(&session.events());
        return Ok(Reply::success(format!("current preset {current} (available: {names})")));
    }
    if let Err(crate::permission::UnknownPreset(unknown)) = table.apply(&*session, name) {
        return Ok(Reply::error(format!(
            "unknown preset \"{unknown}\" (available: {names})"
        )));
    }
    Ok(Reply::success(format!("preset {name}")))
}

/// `/plan [off|message]`: leave plan mode, or enter it (optionally steering
/// the agent with a message).
fn plan(
    ctx: &Context,
    agent: &Arc<dyn AgentView>,
    raw_input: &str,
    attachments: Admitted,
) -> Result<Reply, Failure> {
    let Some(mode) = ctx.get::<PlanMode>(crate::plugins::plan::PLAN_SERVICE) else {
        return Ok(Reply::error("plan mode is not composed"));
    };
    let session = agent.session();
    let message = raw_input.trim();
    if message == "off" {
        if !attachments.is_empty() {
            return Ok(Reply::error("Attachments cannot accompany /plan off."));
        }
        return Ok(Reply::success(match mode.set(&*session, false) {
            SetOutcome::Committed => "Plan mode off.",
            SetOutcome::Queued => "Leaving plan mode (applies from the next step).",
            SetOutcome::Cancelled => "Plan mode entry cancelled.",
            // An exit still waiting for a step reads as queued; only a session
            // that is really out of plan mode reads as idempotent.
            SetOutcome::Noop if PlanMode::logged_active(&*session) => {
                "Leaving plan mode (applies from the next step)."
            }
            SetOutcome::Noop => "Plan mode is already inactive.",
        }));
    }
    let outcome = mode.set(&*session, true);
    if !message.is_empty() || !attachments.is_empty() {
        let mut content = attachments;
        if !message.is_empty() {
            content.push(ContentBlock::text(message));
        }
        agent.steer(Message {
            id: format!("gui-{}", crate::plugins::unique_id()),
            role: Role::User,
            content,
            source: MessageSource::User { rpc_id: None },
        });
    }
    Ok(Reply::success(if outcome == SetOutcome::Committed {
        "Plan mode on. Use /plan off to leave."
    } else {
        "Entering plan mode (applies from the next step). Use /plan off to leave."
    }))
    // The TS host answers every non-committed outcome the same way, including
    // an idle repeat of `/plan`; the log is unaffected.
}

fn render_goal(title: &str, view: &goals::View) -> String {
    let goal = &view.goal;
    let mut lines = vec![
        title.to_string(),
        format!("Status: {}", goal.phase.as_str()),
    ];
    if goal.phase == GoalPhase::Blocked {
        if let Some(reason) = &goal.blocked_reason {
            lines.push(format!("Blocker: {}: {}", reason.code, reason.message));
        }
    }
    lines.push(format!("Objective: {}", goal.objective));
    lines.push(format!(
        "Rounds: {}/{}",
        view.rounds_started, goal.max_goal_rounds
    ));
    lines.push(format!("Activation: {}", view.activation));
    lines.push(String::new());
    lines.push(format!("Commands: {}", command_hint(view)));
    lines.join("\n")
}

fn command_hint(view: &goals::View) -> &'static str {
    match view.goal.phase {
        GoalPhase::Active if view.activation == "armed" => {
            "/goal edit <objective>, /goal pause, /goal clear"
        }
        GoalPhase::Active | GoalPhase::Paused | GoalPhase::Blocked => {
            "/goal edit <objective>, /goal resume, /goal clear"
        }
        GoalPhase::Complete => "/goal <objective>, /goal clear",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(line: &str) -> Option<(&str, &str)> {
        parse_command(line).map(|p| (p.name, p.raw_input))
    }

    #[test]
    fn a_command_line_is_a_slash_a_lower_case_name_and_the_rest_verbatim() {
        assert_eq!(parsed("/goal do it"), Some(("goal", " do it")));
        assert_eq!(parsed("/compact"), Some(("compact", "")));
        assert_eq!(parsed("/feedback\tx y"), Some(("feedback", "	x y")));
        assert_eq!(parsed("/compact\n"), Some(("compact", "\n")));
        assert_eq!(parsed("/a-b_9 z"), Some(("a-b_9", " z")));
    }

    #[test]
    fn anything_else_is_not_a_command() {
        for line in ["", "/", "plain", " /compact", "/Feedback x", "/9a", "//x"] {
            assert_eq!(parsed(line), None, "{line:?}");
        }
        // Only a space, tab, newline or carriage return ends a name; a
        // non-breaking space or a stray character does not.
        assert_eq!(parsed("/goal\u{a0}x"), None);
        assert_eq!(parsed("/goal!x"), None);
    }

    #[test]
    fn goal_input_follows_the_typescript_grammar() {
        assert_eq!(parse_goal(""), Goal::Show);
        assert_eq!(parse_goal("   "), Goal::Show);
        assert_eq!(parse_goal(" CLEAR "), Goal::Clear);
        assert_eq!(parse_goal("Pause"), Goal::Pause);
        assert_eq!(parse_goal("resume"), Goal::Resume);
        assert_eq!(parse_goal(" edit "), Goal::InvalidEdit);
        assert_eq!(parse_goal(" EDIT  upper "), Goal::Edit("upper"));
        assert_eq!(parse_goal("edit\tx"), Goal::Edit("x"));
        assert_eq!(parse_goal("editor"), Goal::Create("editor"));
        assert_eq!(parse_goal("ship it"), Goal::Create("ship it"));
        assert_eq!(
            parse_goal("clear everything"),
            Goal::Create("clear everything")
        );
    }

    #[test]
    fn trimming_keeps_the_next_line_character_and_strips_the_byte_order_mark() {
        assert_eq!(js_trim("\u{feff} x \u{85}"), "x \u{85}");
        assert_eq!(js_trim("\u{a0}x\u{2028}"), "x");
    }

    #[test]
    fn only_well_formed_uuids_are_accepted_as_an_anonymous_user_id() {
        assert!(is_uuid("6f14a7bf-63e6-46f0-9160-c61fd77706a1"));
        assert!(!is_uuid("6f14a7bf-63e6-46f0-9160-c61fd77706a"));
        assert!(!is_uuid("zf14a7bf-63e6-46f0-9160-c61fd77706a1"));
        assert!(!is_uuid(""));
    }

    #[test]
    fn a_session_is_marked_compacting_until_the_guard_is_dropped() {
        let commands = Commands::new(None);
        let first = CompactGuard::enter(&commands, "s").expect("first entry");
        assert!(
            CompactGuard::enter(&commands, "s").is_none(),
            "one at a time"
        );
        assert!(
            CompactGuard::enter(&commands, "other").is_some(),
            "per session"
        );
        drop(first);
        assert!(
            CompactGuard::enter(&commands, "s").is_some(),
            "free again, as after a cancelled request"
        );
    }

    #[test]
    fn a_new_anonymous_id_appears_whole_or_not_at_all_and_never_replaces_one() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("anonymous-user-id");
        persist_new(&file, "6f14a7bf-63e6-46f0-9160-c61fd77706a1").unwrap();
        assert_eq!(
            read_uuid(&file).as_deref(),
            Some("6f14a7bf-63e6-46f0-9160-c61fd77706a1")
        );
        let error = persist_new(&file, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            read_uuid(&file).as_deref(),
            Some("6f14a7bf-63e6-46f0-9160-c61fd77706a1")
        );
        let left: Vec<_> = std::fs::read_dir(file.parent().unwrap())
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(left.len(), 1, "no staging file is left behind");
    }

    #[test]
    fn the_default_id_file_is_the_typescript_hosts_own() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |key: &str| {
                pairs
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value.to_string())
            }
        };
        let expect = |pairs, want: Option<&str>| {
            assert_eq!(
                default_anonymous_user_file(env(pairs)),
                want.map(PathBuf::from)
            )
        };
        expect(
            &[("HOME", "/Users/a")],
            Some("/Users/a/.dsh/.anonymous-user-id"),
        );
        expect(
            &[("HOME", "/Users/a"), ("DSH_HOME", "/srv/dsh")],
            Some("/srv/dsh/.anonymous-user-id"),
        );
        expect(
            &[("HOME", "/Users/a"), ("DSH_HOME", "~/alt")],
            Some("/Users/a/alt/.anonymous-user-id"),
        );
        expect(
            &[("HOME", "/Users/a"), ("DSH_HOME", "  ")],
            Some("/Users/a/.dsh/.anonymous-user-id"),
        );
        expect(
            &[("DSH_HOME", "/srv/dsh")],
            Some("/srv/dsh/.anonymous-user-id"),
        );
        expect(&[], None);
        expect(&[("DSH_HOME", "relative/dir")], None);
    }

    #[test]
    fn an_id_already_in_the_file_is_reported_and_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".anonymous-user-id");
        std::fs::write(&file, "0A1B2C3D-0000-4000-8000-0123456789AB\n").unwrap();
        assert_eq!(
            read_uuid(&file).as_deref(),
            Some("0A1B2C3D-0000-4000-8000-0123456789AB")
        );
    }
}
