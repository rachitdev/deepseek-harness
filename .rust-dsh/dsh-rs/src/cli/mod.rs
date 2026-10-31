//! Shared helpers for the dsh-rs headless runner.
//!
//! This crate is a pure **consumer** of the interface layer: it depends only
//! on `dsh-api` (service wrappers), `dsh-types` (shared vocabulary), and
//! `dsh-bundle` (base-bundle composition). It never imports implementation
//! crate types.

pub mod tui;

use crate::types::{
    GoalChange, ContentBlock, Message, MessageSource, Role, SessionEvent, SessionEventData, TurnEndReason,
    now_ms,
};
use serde_json::Value;

/// Build a user-role message for the model (consumer-side helper).
pub fn user_message_with_text(id: impl Into<String>, text: impl Into<String>) -> Message {
    Message {
        id: id.into(),
        role: Role::User,
        content: vec![ContentBlock::text(text)],
        source: MessageSource::User { rpc_id: None },
    }
}

/// Render the final assistant text of a session (the last assistant message).
pub fn last_assistant_text(session: &dyn crate::api::services::SessionView) -> String {
    let messages = session.derive_messages();
    messages
        .iter()
        .rev()
        .find(|m| m.role == Role::Assistant)
        .map(|m| m.text())
        .unwrap_or_default()
}

/// One-line textual rendering of a session event (transcript display).
pub fn render_event(event: &SessionEvent) -> String {
    match &event.data {
        SessionEventData::UserMessage { message } => format!("[user] {}", message.text()),
        SessionEventData::AssistantMessage { message, .. } => {
            format!("[assistant] {}", message.text())
        }
        SessionEventData::ToolCall { name, arguments, .. } => {
            let parsed: Value = serde_json::from_str(arguments).unwrap_or(Value::Null);
            format!("[tool-call] {name} {parsed}")
        }
        SessionEventData::ToolResult { message, .. } => {
            format!("[tool-result] {}", message.text())
        }
        SessionEventData::TurnStart { .. } => "[turn/start]".to_string(),
        SessionEventData::TurnEnd { reason, .. } => format!("[turn/end] {reason:?}"),
        SessionEventData::StepStart { .. } => "[step/start]".to_string(),
        SessionEventData::StepEnd { .. } => "[step/end]".to_string(),
        SessionEventData::AssistantChunk { chunk, .. } => format!("[assistant/chunk] {chunk:?}"),
        SessionEventData::TodoWrite { todos } => format!("[todo/write] {todos:?}"),
        SessionEventData::Compaction { before_seq, .. } => {
            format!("[compaction] events before #{before_seq} summarised")
        }
        SessionEventData::Goal { objective, status, .. } => {
            format!("[goal] {status:?}: {objective}")
        }
        SessionEventData::GoalChange { change } => match change {
            GoalChange::Snapshot { operation, goal, .. } => format!(
                "[goal/change] {} {}: {}",
                operation.as_str(),
                goal.phase.as_str(),
                goal.objective
            ),
            GoalChange::Clear { cleared, .. } => format!("[goal/change] clear {}", cleared.id),
        },
        SessionEventData::RequestHeader { .. } => "[request/header]".to_string(),
        SessionEventData::SessionTitle { title, .. } => format!("[session/title] {title}"),
        SessionEventData::ModelSelection { provider, model } => format!("[model/selection] {provider}/{model}"),
        SessionEventData::PermissionPreset { preset } => format!("[permission/preset] {preset}"),
        SessionEventData::SandboxMode { mode } => format!("[sandbox/mode] {mode}"),
        SessionEventData::ApprovalPolicy { policy, .. } => format!("[approval/policy] {policy}"),
        SessionEventData::PlanMode { active } => format!("[plan/mode] {active}"),
        SessionEventData::AgentPresetSelected { agent_preset } => {
            format!("[agent-preset/selected] {agent_preset}")
        }
        SessionEventData::InboxSpliced { target, start, removed_count, inserted, .. } => {
            format!(
                "[agent/inbox/spliced] {} @{start} +{} -{}",
                target.as_str_name(),
                inserted.len(),
                removed_count.unwrap_or_default()
            )
        }
        SessionEventData::SessionEndSeed => "[session/end-seed]".to_string(),
        SessionEventData::CommandRun { name, .. } => format!("[command/run] /{name}"),
        SessionEventData::CommandDone { success, .. } => {
            format!("[command/done] {}", if *success { "success" } else { "error" })
        }
        SessionEventData::FeedbackRecord { .. } => "[feedback/record]".to_string(),
        SessionEventData::FeedbackMessagePut { item, .. } => {
            format!("[feedback/message-put] {} {}", item.message_id, item.rating)
        }
        SessionEventData::FeedbackMessageDelete { message_id, .. } => {
            format!("[feedback/message-delete] {message_id}")
        }
    }
}

/// Close a crash-orphaned trailing turn with `turn/end { interrupted }`.
/// Returns whether a repair was made (idempotent). Mirrors the persistence
/// crate's reload repair, expressed over the shared vocabulary only.
pub fn repair_crash_turns(events: &mut Vec<SessionEvent>) -> bool {
    let mut open: Option<u64> = None;
    for event in events.iter() {
        match &event.data {
            SessionEventData::TurnStart { turn } => open = Some(*turn),
            SessionEventData::TurnEnd { .. } => open = None,
            _ => {}
        }
    }
    let Some(turn) = open else { return false };
    events.push(SessionEvent::new(
        events.len() as u64,
        now_ms(),
        SessionEventData::TurnEnd {
            turn,
            reason: TurnEndReason::Interrupted,
        },
    ));
    true
}

/// Parse `--key value` style arguments. Returns (flags, positionals).
///
/// A flag whose next token is itself a flag (e.g. `--line --provider mock`)
/// is treated as a bare flag, and the next token is parsed normally — it is
/// never swallowed as a value.
pub fn parse_args(args: &[String]) -> (std::collections::HashMap<String, String>, Vec<String>) {
    let mut flags = std::collections::HashMap::new();
    let mut positionals = Vec::new();
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if let Some(key) = arg.strip_prefix("--") {
            let key = key.to_string();
            // --flag=value or --flag value
            if let Some((k, v)) = key.split_once('=') {
                flags.insert(k.to_string(), v.to_string());
            } else if iter.peek().is_some_and(|next| !next.starts_with("--")) {
                flags.insert(key, iter.next().unwrap().clone());
            } else {
                flags.insert(key, String::new());
            }
        } else {
            positionals.push(arg.clone());
        }
    }
    (flags, positionals)
}

/// Load a JSON profile from a file path or inline JSON.
///
/// `${NAME}` placeholders inside string values are replaced with the
/// environment variable `NAME` after parsing, so secrets and machine paths stay
/// out of the file. An unset variable is an error, never an empty string.
pub fn load_profile(source: &str) -> Result<Value, String> {
    let trimmed = source.trim();
    let mut profile: Value = if trimmed.starts_with('{') {
        serde_json::from_str(trimmed).map_err(|e| format!("bad profile JSON: {e}"))?
    } else {
        let text = std::fs::read_to_string(source).map_err(|e| format!("cannot read {source}: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("bad profile JSON in {source}: {e}"))?
    };
    // A disabled plugin is never started, so its placeholders must not be required.
    if let Some(plugins) = profile.get_mut("plugins").and_then(Value::as_object_mut) {
        plugins.retain(|_, config| config.get("enabled").and_then(Value::as_bool) != Some(false));
    }
    expand_env(&mut profile)?;
    Ok(profile)
}

/// Replace `${NAME}` in every string of `value` (object keys are left alone).
pub fn expand_env(value: &mut Value) -> Result<(), String> {
    match value {
        Value::String(text) => {
            let mut out = String::with_capacity(text.len());
            let mut rest = text.as_str();
            while let Some(start) = rest.find("${") {
                out.push_str(&rest[..start]);
                let after = &rest[start + 2..];
                let end = after.find('}').ok_or_else(|| format!("unterminated ${{ placeholder starting at byte {}", text.len() - after.len() - 2))?;
                let name = &after[..end];
                let resolved = std::env::var(name).map_err(|_| format!("profile needs environment variable {name}, which is not set"))?;
                out.push_str(&resolved);
                rest = &after[end + 1..];
            }
            out.push_str(rest);
            *text = out;
        }
        Value::Array(items) => items.iter_mut().try_for_each(expand_env)?,
        Value::Object(map) => map.values_mut().try_for_each(expand_env)?,
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{expand_env, parse_args};
    use serde_json::json;

    #[test]
    fn env_placeholders_expand_in_nested_strings_only() {
        std::env::set_var("DSH_RS_EXPAND_A", "va\"l");
        let mut v = json!({"k": ["x-${DSH_RS_EXPAND_A}-y", 3], "${DSH_RS_EXPAND_A}": "plain"});
        expand_env(&mut v).unwrap();
        assert_eq!(v["k"][0], "x-va\"l-y");
        assert_eq!(v["k"][1], 3);
        assert_eq!(v["${DSH_RS_EXPAND_A}"], "plain", "keys are not expanded");
    }

    #[test]
    fn disabled_plugins_do_not_require_their_variables() {
        let inline = r#"{"plugins":{"off":{"enabled":false,"k":"${DSH_RS_NEVER_SET_Z}"},"on":{"k":"v"}}}"#;
        let profile = super::load_profile(inline).expect("disabled entry is skipped before expansion");
        assert!(profile["plugins"].get("off").is_none() && profile["plugins"].get("on").is_some());
    }

    #[test]
    fn unset_or_unterminated_placeholders_are_errors() {
        assert!(expand_env(&mut json!("${DSH_RS_SURELY_UNSET_X}")).unwrap_err().contains("DSH_RS_SURELY_UNSET_X"));
        assert!(expand_env(&mut json!("a ${oops")).unwrap_err().contains("unterminated"));
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn flag_followed_by_another_flag_is_not_swallowed() {
        // Regression: `--line --provider mock` must keep both flags; the
        // second flag used to be eaten as `--line`'s value.
        let (flags, positionals) = parse_args(&args(&["chat", "--line", "--provider", "mock", "--model", "mock-1"]));
        assert_eq!(flags.get("line").map(|s| s.as_str()), Some(""));
        assert_eq!(flags.get("provider").map(|s| s.as_str()), Some("mock"));
        assert_eq!(flags.get("model").map(|s| s.as_str()), Some("mock-1"));
        assert_eq!(positionals, vec!["chat"]);
    }

    #[test]
    fn flag_value_and_equals_forms() {
        let (flags, positionals) = parse_args(&args(&["run", "--prompt", "hi", "--max-tokens=64", "--print-json"]));
        assert_eq!(flags.get("prompt").map(|s| s.as_str()), Some("hi"));
        assert_eq!(flags.get("max-tokens").map(|s| s.as_str()), Some("64"));
        assert!(flags.contains_key("print-json"));
        assert_eq!(positionals, vec!["run"]);
    }

    #[test]
    fn trailing_bare_flag_is_ok() {
        let (flags, positionals) = parse_args(&args(&["chat", "--line"]));
        assert!(flags.contains_key("line"));
        assert_eq!(positionals, vec!["chat"]);
    }
}