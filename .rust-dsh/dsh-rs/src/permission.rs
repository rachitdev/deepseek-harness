//! Permission presets: the `sandbox/mode` and `approval/policy` knobs a session
//! carries, the named presets that set both, and the checks that enforce them.
//!
//! The knobs live in the session log (`permission/preset`, `sandbox/mode`,
//! `approval/policy`, the TypeScript host's row shapes); everything here is a
//! pure function of that log plus the deployment's [`Permissions`] table.
//!
//! What dsh-rs can enforce differs from the TypeScript host, which confines
//! processes with an OS sandbox. This build has none, so:
//!
//! - `danger-full-access` restricts nothing.
//! - `read-only` refuses every file-changing tool and every shell tool; which
//!   tools those are is configuration (`fileTools`, `shellTools`). A tool that
//!   is in neither list, such as an MCP tool, is not covered.
//! - `workspace-write` refuses `write_file` and `edit_file` outside the
//!   session's working directory and the temporary directories (the TypeScript
//!   host would ask to escalate; there is no escalation here), and refuses
//!   shell tools when the approval policy is `never` or the approvals plugin is
//!   not composed, because a shell cannot be confined here and nobody could
//!   approve it. Under `ask`, shell tools go through the approvals plugin like
//!   any other tool.
//!
//! The checks fail closed: a file tool call without a readable `path`, a path
//! that cannot be resolved (or loops through links), a workspace that is the
//! file system root, and a knob value this build does not know all refuse the
//! call. The workspace is the directory the file tools resolve relative paths
//! against: the session's working directory, else the process's.
//! Paths are resolved component by component, following links, so neither a
//! dangling link nor `link/..` leads out of the workspace. The check runs
//! before the tool opens the file, so a path swapped in between is not caught.
//! - Approval `never` means no prompt is ever shown: a tool the approvals
//!   plugin would ask about runs when the sandbox allows it. (The TypeScript
//!   host rejects the escalation requests instead, which exist only under a
//!   confining sandbox.) The approvals plugin's `deny` list still applies.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::services::{AgentRegistryService, SessionView};
use crate::types::{SessionEvent, SessionEventData};

/// The `permissions` service key.
pub const PERMISSIONS_SERVICE: &str = "permissions";

/// The sandbox modes, as the `sandbox/mode` event spells them.
pub const SANDBOX_MODES: [&str; 3] = ["read-only", "workspace-write", "danger-full-access"];

/// The approval policies, as the `approval/policy` event spells them.
pub const APPROVAL_POLICIES: [&str; 2] = ["ask", "never"];

/// What the projection reports when the knobs match no preset.
pub const CUSTOM_PRESET: &str = "custom";

/// A name the TypeScript host reserves for its per-call review preset.
pub const RESERVED_AUTO_PRESET: &str = "auto";

/// One preset as configured.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetConfig {
    /// The preset's key, the value `/permission` takes.
    pub id: String,
    /// The `sandbox/mode` value it writes.
    pub sandbox: String,
    /// The `approval/policy` value it writes.
    pub approval: String,
    /// The label a client shows; the key when absent.
    #[serde(default)]
    pub name: Option<String>,
    /// One sentence on what the preset means.
    #[serde(default)]
    pub description: Option<String>,
}

/// Configuration for the `permission` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionConfig {
    /// The sandbox mode a session has until it records one.
    pub sandbox: String,
    /// The approval policy a session has until it records one.
    pub approval: String,
    /// The presets on offer, in display order. The first one matching the
    /// knobs names them.
    pub presets: Vec<PresetConfig>,
    /// Tools that change files; each names its target in the `path` argument.
    pub file_tools: Vec<String>,
    /// Tools that run a shell.
    pub shell_tools: Vec<String>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        let preset = |id: &str, sandbox: &str, approval: &str, name: &str, description: &str| {
            PresetConfig {
                id: id.to_string(),
                sandbox: sandbox.to_string(),
                approval: approval.to_string(),
                name: Some(name.to_string()),
                description: Some(description.to_string()),
            }
        };
        PermissionConfig {
            sandbox: "danger-full-access".to_string(),
            approval: "ask".to_string(),
            presets: vec![
                preset(
                    "profile",
                    "danger-full-access",
                    "ask",
                    "Profile rules",
                    "Tool approvals follow the approvals plugin configured in the profile; file access is not confined.",
                ),
                preset(
                    "workspace-write",
                    "workspace-write",
                    "ask",
                    "workspace-write",
                    "Write inside the workspace and permitted temporary directories; wider retries require approval.",
                ),
                preset(
                    "danger-full-access",
                    "danger-full-access",
                    "never",
                    "danger-full-access",
                    "Full file access without approval prompts.",
                ),
            ],
            file_tools: vec!["write_file".to_string(), "edit_file".to_string()],
            shell_tools: vec!["bash".to_string()],
        }
    }
}

/// The last value of each knob event in a session log; `None` before an
/// override, when the deployment defaults apply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Knobs {
    /// Last `permission/preset` payload.
    pub preset: Option<String>,
    /// Last `sandbox/mode` payload.
    pub sandbox: Option<String>,
    /// Last `approval/policy` payload.
    pub approval: Option<String>,
}

/// The validated preset table and defaults.
#[derive(Debug, Clone)]
pub struct Permissions {
    sandbox: String,
    approval: String,
    presets: Vec<PresetConfig>,
    file_tools: Vec<String>,
    shell_tools: Vec<String>,
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions::from_config(&PermissionConfig::default())
            .expect("the built-in permission table is valid")
    }
}

/// The permission table the `permission` plugin provides, or `None` when the
/// deployment does not enforce presets.
pub fn provided(ctx: &cordis::Context) -> Option<Arc<Permissions>> {
    ctx.get::<Permissions>(PERMISSIONS_SERVICE)
}

impl Permissions {
    /// Check `config` and build the table.
    pub fn from_config(config: &PermissionConfig) -> Result<Permissions, String> {
        let check = |what: &str, value: &str, allowed: &[&str]| {
            if allowed.contains(&value) {
                Ok(())
            } else {
                Err(format!(
                    "permission: {what} \"{value}\" is not one of {}",
                    allowed.join(", ")
                ))
            }
        };
        check("sandbox", &config.sandbox, &SANDBOX_MODES)?;
        check("approval", &config.approval, &APPROVAL_POLICIES)?;
        if config.presets.is_empty() {
            return Err("permission: `presets` must name at least one preset".to_string());
        }
        let mut seen: Vec<&str> = Vec::new();
        for preset in &config.presets {
            if preset.id.trim().is_empty() {
                return Err("permission: a preset needs a non-empty `id`".to_string());
            }
            if preset.id == CUSTOM_PRESET || preset.id == RESERVED_AUTO_PRESET {
                return Err(format!(
                    "permission: \"{}\" is a reserved preset name",
                    preset.id
                ));
            }
            if seen.contains(&preset.id.as_str()) {
                return Err(format!("permission: preset \"{}\" is listed twice", preset.id));
            }
            seen.push(&preset.id);
            check(
                &format!("preset \"{}\" sandbox", preset.id),
                &preset.sandbox,
                &SANDBOX_MODES,
            )?;
            check(
                &format!("preset \"{}\" approval", preset.id),
                &preset.approval,
                &APPROVAL_POLICIES,
            )?;
        }
        Ok(Permissions {
            sandbox: config.sandbox.clone(),
            approval: config.approval.clone(),
            presets: config.presets.clone(),
            file_tools: config.file_tools.clone(),
            shell_tools: config.shell_tools.clone(),
        })
    }

    /// The preset keys, in display order.
    pub fn names(&self) -> Vec<&str> {
        self.presets.iter().map(|p| p.id.as_str()).collect()
    }

    /// The knobs a session log last set.
    pub fn knobs(events: &[SessionEvent]) -> Knobs {
        let mut knobs = Knobs::default();
        for event in events.iter().rev() {
            match &event.data {
                SessionEventData::PermissionPreset { preset } if knobs.preset.is_none() => {
                    knobs.preset = Some(preset.clone());
                }
                SessionEventData::SandboxMode { mode } if knobs.sandbox.is_none() => {
                    knobs.sandbox = Some(mode.clone());
                }
                SessionEventData::ApprovalPolicy { policy, .. } if knobs.approval.is_none() => {
                    knobs.approval = Some(policy.clone());
                }
                _ => {}
            }
            if knobs.preset.is_some() && knobs.sandbox.is_some() && knobs.approval.is_some() {
                break;
            }
        }
        knobs
    }

    /// The sandbox mode and approval policy in force for `knobs`. A value this
    /// build does not know (a hand-edited or newer log) reads as the strictest
    /// one, `read-only` or `ask`, so it never loosens anything.
    pub fn effective(&self, knobs: &Knobs) -> (&'static str, &'static str) {
        let known = |value: &str, allowed: &[&'static str], strictest: &'static str| {
            allowed.iter().find(|a| **a == value).copied().unwrap_or(strictest)
        };
        (
            known(
                knobs.sandbox.as_deref().unwrap_or(&self.sandbox),
                &SANDBOX_MODES,
                "read-only",
            ),
            known(
                knobs.approval.as_deref().unwrap_or(&self.approval),
                &APPROVAL_POLICIES,
                "ask",
            ),
        )
    }

    /// The preset the knobs amount to: the last selection when it still
    /// matches, else the first configured match, else `custom`.
    pub fn derive(&self, knobs: &Knobs) -> String {
        let (sandbox, approval) = self.effective(knobs);
        let matches = |p: &PresetConfig| p.sandbox == sandbox && p.approval == approval;
        if let Some(selected) = &knobs.preset {
            if self.presets.iter().any(|p| &p.id == selected && matches(p)) {
                return selected.clone();
            }
        }
        self.presets
            .iter()
            .find(|p| matches(p))
            .map_or_else(|| CUSTOM_PRESET.to_string(), |p| p.id.clone())
    }

    /// The preset in force for a session log.
    pub fn current(&self, events: &[SessionEvent]) -> String {
        self.derive(&Self::knobs(events))
    }

    /// The preset a session starts on.
    pub fn default_preset(&self) -> String {
        self.derive(&Knobs::default())
    }

    /// The `permissions` projection value.
    pub fn view(&self, events: &[SessionEvent]) -> Value {
        json!({ "currentValue": self.current(events) })
    }

    /// The `permissionPresets/catalog` payload.
    pub fn catalog(&self) -> Value {
        let options: Vec<Value> = self
            .presets
            .iter()
            .map(|preset| {
                let mut option = json!({
                    "value": preset.id,
                    "name": preset.name.as_deref().unwrap_or(&preset.id),
                });
                if let Some(description) = &preset.description {
                    option["description"] = json!(description);
                }
                option
            })
            .collect();
        json!({
            "options": options.clone(),
            "defaultOptions": options,
            "defaultPreset": self.default_preset(),
        })
    }

    /// Switch a session to `name`: record the selection when it is not the
    /// preset in force, then each knob that changes. Selecting the preset in
    /// force again appends nothing. The knobs are written tightening-first, so
    /// a tool call racing the switch never sees a state looser than both the
    /// old and the new one.
    pub fn apply(&self, session: &dyn SessionView, name: &str) -> Result<(), UnknownPreset> {
        let preset = self
            .presets
            .iter()
            .find(|p| p.id == name)
            .ok_or_else(|| UnknownPreset(name.to_string()))?;
        let knobs = Self::knobs(&session.events());
        let (sandbox, approval) = self.effective(&knobs);
        if self.derive(&knobs) != name {
            session.append(SessionEventData::PermissionPreset { preset: name.to_string() });
        }
        let sandbox_rank = |mode: &str| SANDBOX_MODES.iter().position(|m| *m == mode);
        // `read-only` is first in SANDBOX_MODES, so a lower rank is stricter.
        let sandbox_tighter = sandbox_rank(&preset.sandbox) < sandbox_rank(sandbox);
        let approval_tighter = preset.approval == "ask" && approval != "ask";
        let set_sandbox = || {
            if preset.sandbox != sandbox {
                session.append(SessionEventData::SandboxMode { mode: preset.sandbox.clone() });
            }
        };
        let set_approval = || {
            if preset.approval != approval {
                session.append(SessionEventData::ApprovalPolicy {
                    policy: preset.approval.clone(),
                    source: None,
                });
            }
        };
        if approval_tighter && !sandbox_tighter {
            set_approval();
            set_sandbox();
        } else {
            set_sandbox();
            set_approval();
        }
        Ok(())
    }

    /// Record the session's initial permission setup.
    ///
    /// The TypeScript host logs the preset, the sandbox mode and the approval
    /// policy when a session is created, so a blank session's log says which
    /// setup it runs. This build logs the same three rows, with the values the
    /// deployment's configuration gives.
    pub fn pin(&self, session: &dyn SessionView) {
        let knobs = Self::knobs(&session.events());
        let (sandbox, approval) = self.effective(&knobs);
        session.append(SessionEventData::PermissionPreset {
            preset: self.derive(&knobs),
        });
        session.append(SessionEventData::SandboxMode { mode: sandbox.to_string() });
        session.append(SessionEventData::ApprovalPolicy {
            policy: approval.to_string(),
            source: None,
        });
    }

    /// Copy the knobs `parent` has set onto `child`, so a delegated agent
    /// starts under the permissions of the agent that started it. The rows are
    /// appended to the child's log, so the inheritance replays.
    pub fn inherit(parent: &dyn SessionView, child: &dyn SessionView) {
        let knobs = Self::knobs(&parent.events());
        if let Some(sandbox) = knobs.sandbox {
            child.append(SessionEventData::SandboxMode { mode: sandbox });
        }
        if let Some(approval) = knobs.approval {
            child.append(SessionEventData::ApprovalPolicy { policy: approval, source: None });
        }
        if let Some(preset) = knobs.preset {
            child.append(SessionEventData::PermissionPreset { preset });
        }
    }

    /// Why the sandbox refuses `tool`, or `None` when it allows it.
    ///
    /// `workspace` is the session's working directory, the root of what
    /// `workspace-write` allows; `base` is what a relative `path` is taken
    /// against. `approvals_present` says whether the approvals plugin is
    /// composed to gate shell tools under the `ask` policy.
    pub fn sandbox_denial(&self, call: &SandboxCall<'_>) -> Option<String> {
        let (sandbox, approval) = self.effective(call.knobs);
        let tool = call.tool;
        let is_file = self.file_tools.iter().any(|t| t == tool);
        let is_shell = self.shell_tools.iter().any(|t| t == tool);
        match sandbox {
            "read-only" if is_file || is_shell => Some(format!(
                "`{tool}` is refused: the session is in read-only sandbox mode"
            )),
            "workspace-write" if is_shell && approval == "never" => Some(format!(
                "`{tool}` is refused: this build cannot confine a shell to the workspace, and \
                 the approval policy is `never`, so nobody can approve it"
            )),
            "workspace-write" if is_shell && !call.approvals_present => Some(format!(
                "`{tool}` is refused: this build cannot confine a shell to the workspace, and \
                 no approvals plugin is composed to ask for approval"
            )),
            "workspace-write" if is_file => Self::workspace_denial(call),
            _ => None,
        }
    }

    fn workspace_denial(call: &SandboxCall<'_>) -> Option<String> {
        let tool = call.tool;
        let refuse = |why: &str| Some(format!("`{tool}` is refused (sandbox mode workspace-write): {why}"));
        let Some(workspace) = call
            .workspace
            .filter(|w| Path::new(w).is_absolute() && Path::new(w).parent().is_some())
        else {
            return refuse("there is no working directory narrower than `/` to confine writes to");
        };
        let Some(raw) = call.arguments.get("path").and_then(Value::as_str) else {
            return refuse("the call names no path");
        };
        let base = call.base.filter(|b| Path::new(b).is_absolute()).unwrap_or(workspace);
        let roots = [workspace, "/tmp", &std::env::temp_dir().to_string_lossy()]
            .map(|root| resolve(Path::new("/"), root));
        let Some(target) = resolve(Path::new(base), raw) else {
            return refuse(&format!("the path {raw} cannot be resolved"));
        };
        let inside = roots.iter().flatten().any(|root| target.starts_with(root));
        (!inside).then(|| {
            format!(
                "`{tool}` is refused: {} is outside the workspace and the temporary directories \
                 (sandbox mode workspace-write)",
                target.display()
            )
        })
    }
}

/// A tool call as the sandbox sees it.
pub struct SandboxCall<'a> {
    /// The calling session's knobs.
    pub knobs: &'a Knobs,
    /// The session's working directory.
    pub workspace: Option<&'a str>,
    /// What a relative `path` is taken against; the workspace when absent.
    pub base: Option<&'a str>,
    /// The tool's name.
    pub tool: &'a str,
    /// The tool's arguments.
    pub arguments: &'a Value,
    /// Whether the approvals plugin is composed.
    pub approvals_present: bool,
}

/// `/permission` named a preset that is not configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownPreset(pub String);

/// One step of a path being resolved.
enum Step {
    Root(std::ffi::OsString),
    Parent,
    Name(std::ffi::OsString),
}

fn steps_of(path: &Path) -> Vec<Step> {
    path.components()
        .filter_map(|part| match part {
            Component::CurDir => None,
            Component::ParentDir => Some(Step::Parent),
            Component::Normal(name) => Some(Step::Name(name.to_os_string())),
            other => Some(Step::Root(other.as_os_str().to_os_string())),
        })
        .collect()
}

/// The most links one resolution follows before it gives up.
const MAX_LINK_HOPS: usize = 40;

/// `raw` against `base` as the file system would resolve it: component by
/// component, a link replaced by its target, and `..` applied to the real
/// directory it follows. A component that does not exist (yet) is kept as
/// written. `None` when a link loops or a component cannot be inspected.
fn resolve(base: &Path, raw: &str) -> Option<PathBuf> {
    let mut todo = steps_of(&base.join(raw));
    todo.reverse();
    let mut real = PathBuf::new();
    let mut hops = 0;
    while let Some(step) = todo.pop() {
        match step {
            Step::Root(root) => real = PathBuf::from(root),
            Step::Parent => {
                real.pop();
            }
            Step::Name(name) => {
                let next = real.join(&name);
                match std::fs::symlink_metadata(&next) {
                    Ok(meta) if meta.file_type().is_symlink() => {
                        hops += 1;
                        if hops > MAX_LINK_HOPS {
                            return None;
                        }
                        let target = std::fs::read_link(&next).ok()?;
                        let mut steps = steps_of(&target);
                        steps.reverse();
                        todo.extend(steps);
                    }
                    Ok(_) => real = next,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => real = next,
                    Err(_) => return None,
                }
            }
        }
    }
    Some(real)
}

/// The registry's session for an agent id, when it is live.
pub fn session_of(ctx: &cordis::Context, agent: &str) -> Option<Arc<dyn SessionView>> {
    ctx.get::<AgentRegistryService>(crate::api::AGENTS_SERVICE)
        .and_then(|agents| agents.get(agent))
        .map(|agent| agent.session())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(seq: u64, data: SessionEventData) -> SessionEvent {
        SessionEvent { seq, time: 0, data }
    }

    #[test]
    fn a_fresh_log_reports_the_default_preset() {
        let table = Permissions::default();
        assert_eq!(table.current(&[]), "profile");
        assert_eq!(table.default_preset(), "profile");
    }

    #[test]
    fn knobs_that_match_no_preset_read_custom() {
        let table = Permissions::default();
        let log = [ev(0, SessionEventData::SandboxMode { mode: "read-only".into() })];
        assert_eq!(table.current(&log), "custom");
    }

    #[test]
    fn the_last_selection_names_a_shared_bundle() {
        let config = PermissionConfig {
            presets: vec![
                PresetConfig {
                    id: "one".into(),
                    sandbox: "read-only".into(),
                    approval: "ask".into(),
                    name: None,
                    description: None,
                },
                PresetConfig {
                    id: "two".into(),
                    sandbox: "read-only".into(),
                    approval: "ask".into(),
                    name: None,
                    description: None,
                },
            ],
            ..PermissionConfig::default()
        };
        let table = Permissions::from_config(&config).unwrap();
        let both = [
            ev(0, SessionEventData::PermissionPreset { preset: "two".into() }),
            ev(1, SessionEventData::SandboxMode { mode: "read-only".into() }),
        ];
        assert_eq!(table.current(&both), "two");
        assert_eq!(table.current(&both[1..]), "one");
    }

    #[test]
    fn a_bad_table_is_refused_with_the_reason() {
        let config = PermissionConfig { sandbox: "moat".into(), ..PermissionConfig::default() };
        assert!(Permissions::from_config(&config).unwrap_err().contains("moat"));
        let mut config = PermissionConfig::default();
        config.presets[0].id = "custom".into();
        assert!(Permissions::from_config(&config).unwrap_err().contains("reserved"));
        let mut config = PermissionConfig::default();
        config.presets.push(config.presets[0].clone());
        assert!(Permissions::from_config(&config).unwrap_err().contains("twice"));
    }

    #[test]
    fn paths_resolve_through_links_and_parents() {
        let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
        let base = dir.path().canonicalize().unwrap();
        let outside = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
        let outside = outside.path().canonicalize().unwrap();
        std::fs::create_dir(base.join("sub")).unwrap();
        // A plain lexical fold.
        assert_eq!(resolve(&base, "sub/../b/c.txt"), Some(base.join("b/c.txt")));
        // `link/..` is the parent of the link's target, not of the link.
        std::os::unix::fs::symlink(&outside, base.join("link")).unwrap();
        assert_eq!(
            resolve(&base, "link/../x"),
            Some(outside.parent().unwrap().join("x"))
        );
        // A dangling link resolves to where it points.
        std::os::unix::fs::symlink(outside.join("missing/file"), base.join("dangling")).unwrap();
        assert_eq!(resolve(&base, "dangling"), Some(outside.join("missing/file")));
        // A loop gives up.
        std::os::unix::fs::symlink(base.join("b"), base.join("a")).unwrap();
        std::os::unix::fs::symlink(base.join("a"), base.join("b")).unwrap();
        assert_eq!(resolve(&base, "a"), None);
    }

    fn call<'a>(knobs: &'a Knobs, tool: &'a str, arguments: &'a Value, approvals: bool) -> SandboxCall<'a> {
        SandboxCall {
            knobs,
            workspace: Some(env!("CARGO_MANIFEST_DIR")),
            base: None,
            tool,
            arguments,
            approvals_present: approvals,
        }
    }

    fn knobs(sandbox: &str, approval: &str) -> Knobs {
        Knobs { preset: None, sandbox: Some(sandbox.into()), approval: Some(approval.into()) }
    }

    #[test]
    fn the_sandbox_decides_by_mode_policy_and_tool() {
        let table = Permissions::default();
        let none = json!({});
        let inside = json!({ "path": concat!(env!("CARGO_MANIFEST_DIR"), "/x.txt") });
        let outside = json!({ "path": "/etc/passwd" });
        // (sandbox, approval, tool, arguments, approvals plugin, refused?)
        let cases: [(&str, &str, &str, &Value, bool, bool); 14] = [
            ("danger-full-access", "never", "bash", &none, false, false),
            ("danger-full-access", "never", "write_file", &outside, false, false),
            ("read-only", "ask", "write_file", &inside, true, true),
            ("read-only", "ask", "bash", &none, true, true),
            ("read-only", "ask", "read_file", &outside, true, false),
            ("workspace-write", "ask", "write_file", &inside, true, false),
            ("workspace-write", "ask", "write_file", &outside, true, true),
            ("workspace-write", "ask", "write_file", &none, true, true),
            ("workspace-write", "ask", "write_file", &json!({ "path": 7 }), true, true),
            ("workspace-write", "ask", "bash", &none, true, false),
            ("workspace-write", "ask", "bash", &none, false, true),
            ("workspace-write", "never", "bash", &none, true, true),
            // A value this build does not know reads as the strictest one.
            ("readonly", "ask", "write_file", &inside, true, true),
            ("danger-full-access", "sometimes", "bash", &none, true, false),
        ];
        for (sandbox, approval, tool, arguments, approvals, refused) in cases {
            let knobs = knobs(sandbox, approval);
            let verdict = table.sandbox_denial(&call(&knobs, tool, arguments, approvals));
            assert_eq!(verdict.is_some(), refused, "{sandbox}/{approval} {tool} {arguments}: {verdict:?}");
        }
    }

    #[test]
    fn a_session_without_a_working_directory_cannot_write_under_workspace_write() {
        let table = Permissions::default();
        let knobs = knobs("workspace-write", "ask");
        let args = json!({ "path": "/tmp/x" });
        let mut c = call(&knobs, "write_file", &args, true);
        c.workspace = None;
        assert!(table.sandbox_denial(&c).unwrap().contains("no working directory narrower"));
    }
}
