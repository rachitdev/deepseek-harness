//! Compaction end to end: a real agent, a recording model, the real plugin.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, BoxFuture, LlmAdapterApi, LlmService};
use dsh_rs::bundle::install_profile;
use dsh_rs::types::{
    stream_from_chunks, AgentOptions, BoxStream, ContentBlock, GenerateOptions, LlmError, Message,
    SessionEventData, StreamChunk,
};
use serde_json::{json, Value};

mod common;
use common::text_response;

/// Answers from a queue of replies (`Err` = provider failure) and records every request.
struct Recorder {
    replies: Mutex<VecDeque<Result<String, String>>>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

impl LlmAdapterApi for Recorder {
    fn name(&self) -> &'static str {
        "recorder"
    }
    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<BoxStream<StreamChunk>, LlmError>> {
        self.seen.lock().unwrap().push(options);
        let reply = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok("ok".into()));
        Box::pin(async move {
            match reply {
                Ok(text) => Ok(stream_from_chunks(text_response(&text))),
                Err(message) => Err(LlmError::code("PROVIDER_DOWN", message)),
            }
        })
    }
}

struct Rig {
    ctx: Context,
    agent: Arc<dyn dsh_rs::api::services::AgentView>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

async fn rig(compaction: Value, replies: Vec<Result<String, String>>) -> Rig {
    let ctx = Context::new();
    let profile = json!({ "plugins": { "compaction": compaction } });
    install_profile(&ctx, &profile)
        .await
        .expect("profile installs");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(
        &["mock"],
        Arc::new(Recorder {
            replies: Mutex::new(replies.into()),
            seen: seen.clone(),
        }),
    )
    .unwrap();
    let agents = ctx.require::<AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
    Rig { ctx, agent, seen }
}

async fn say(rig: &Rig, id: &str, text: &str) {
    rig.agent
        .followup(Message::user(id, vec![ContentBlock::text(text)]));
    rig.agent.when_idle().await;
}

fn compactions(rig: &Rig) -> Vec<(String, u64)> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|e| match e.data {
            SessionEventData::Compaction {
                summary,
                before_seq,
            } => Some((summary, before_seq)),
            _ => None,
        })
        .collect()
}

fn texts(options: &GenerateOptions) -> Vec<String> {
    options.messages.iter().map(|m| m.text()).collect()
}

const BIG: &str = "0123456789 this is a long message padding the history out. ";

fn tight() -> Value {
    json!({ "triggerTokens": 50, "keepRecentTurns": 1, "minReclaimTokens": 1 })
}

#[tokio::test]
async fn a_long_conversation_is_summarised_and_the_model_then_sees_the_summary() {
    let big = BIG.repeat(8);
    let rig = rig(
        tight(),
        vec![
            Ok("answer one".into()),
            Ok("answer two".into()),
            Ok("SUMMARY: user asked two things".into()), // the summary call, before turn 3
            Ok("answer three".into()),
        ],
    )
    .await;
    say(&rig, "u1", &format!("first {big}")).await;
    say(&rig, "u2", &format!("second {big}")).await;
    assert!(
        compactions(&rig).is_empty(),
        "history was still small enough at turn 2"
    );
    say(&rig, "u3", "third").await;

    let log = compactions(&rig);
    assert_eq!(log.len(), 1, "{log:?}");
    assert!(log[0].0.contains("SUMMARY"));

    // Compaction runs before the incoming message is logged, so the kept "recent
    // turn" is the last completed one (turn 2) and the new message comes after it.
    // Only turn 1 is replaced.
    let seen = rig.seen.lock().unwrap();
    let turn3 = seen.last().unwrap();
    let sent = texts(turn3);
    assert!(
        sent[0].contains("SUMMARY: user asked two things"),
        "{sent:?}"
    );
    assert!(sent.iter().any(|t| t == "third"));
    assert!(
        sent.iter().any(|t| t.starts_with("second")),
        "turn 2 is kept: {sent:?}"
    );
    assert!(
        !sent.iter().any(|t| t.starts_with("first")),
        "turn 1 is replaced: {sent:?}"
    );
    // The summary request itself was a separate, tool-free call.
    let summary_call = &seen[seen.len() - 2];
    assert!(summary_call.tools.is_none());
    assert!(
        texts(summary_call)[0].contains("first 0123456789"),
        "turn 1 was summarised"
    );
    assert!(
        !texts(summary_call)[0].contains("second 0123"),
        "kept turns are not sent for summary"
    );
    drop(seen);

    // Replaying the log gives the same visible history the model saw.
    let replayed = dsh_rs::session::Session::new(
        "r".into(),
        dsh_rs::types::SessionHeader {
            agent_preset: None,
            cwd: None,
            created_at: 0,
            parent_session: None,
            seed_length: 0,
        },
        rig.agent.session().events(),
    );
    assert_eq!(
        replayed.derive_messages(),
        rig.agent.session().derive_messages()
    );
    let _ = &rig.ctx;
}

#[tokio::test]
async fn a_failed_summary_leaves_the_history_alone_and_the_turn_still_runs() {
    let big = BIG.repeat(8);
    let rig = rig(
        tight(),
        vec![
            Ok("one".into()),
            Ok("two".into()),
            Err("provider down".into()), // the summary call fails
            Ok("three".into()),
        ],
    )
    .await;
    say(&rig, "u1", &format!("first {big}")).await;
    say(&rig, "u2", &format!("second {big}")).await;
    say(&rig, "u3", "third").await;

    assert!(
        compactions(&rig).is_empty(),
        "nothing is logged when the summary fails"
    );
    let seen = rig.seen.lock().unwrap();
    let sent = texts(seen.last().unwrap());
    assert!(
        sent.iter().any(|t| t.starts_with("first")),
        "full history still sent: {sent:?}"
    );
    assert_eq!(
        rig.agent.session().derive_messages().last().unwrap().text(),
        "three"
    );
}

#[tokio::test]
async fn a_summary_that_is_not_shorter_is_rejected() {
    let rig = rig(
        tight(),
        vec![
            Ok("a".into()),
            Ok("b".into()),
            Ok("x".repeat(5000)),
            Ok("c".into()),
        ],
    )
    .await;
    say(&rig, "u1", &format!("first {}", BIG.repeat(8))).await;
    say(&rig, "u2", &format!("second {}", BIG.repeat(8))).await;
    say(&rig, "u3", "third").await;
    assert!(compactions(&rig).is_empty());
}

#[tokio::test]
async fn below_the_trigger_nothing_happens() {
    let rig = rig(json!({ "triggerTokens": 100000 }), vec![]).await;
    for i in 0..4 {
        say(
            &rig,
            &format!("u{i}"),
            &format!("turn {i} {}", BIG.repeat(4)),
        )
        .await;
    }
    assert!(compactions(&rig).is_empty());
    assert_eq!(
        rig.seen.lock().unwrap().len(),
        4,
        "one model call per turn, no summary calls"
    );
}

#[tokio::test]
async fn compacting_twice_keeps_one_summary_in_view() {
    let big = BIG.repeat(8);
    let rig = rig(
        tight(),
        vec![
            Ok("r1".into()),
            Ok("r2".into()),
            Ok("S1".into()),
            Ok("r3".into()),
            Ok("S2".into()),
            Ok("r4".into()),
        ],
    )
    .await;
    for (i, label) in ["first", "second", "third", "fourth"].iter().enumerate() {
        say(&rig, &format!("u{i}"), &format!("{label} {big}")).await;
    }
    assert_eq!(compactions(&rig).len(), 2);
    let visible: Vec<String> = rig
        .agent
        .session()
        .derive_messages()
        .iter()
        .map(|m| m.text())
        .collect();
    let summaries = visible
        .iter()
        .filter(|t| t.contains("[Summary of the earlier"))
        .count();
    assert_eq!(summaries, 1, "{visible:?}");
    assert!(visible[0].contains("S2") && !visible[0].contains("S1"));
}

#[tokio::test]
async fn an_invalid_setting_fails_the_profile_load() {
    let ctx = Context::new();
    let profile = json!({ "plugins": { "compaction": { "keepRecentTurns": 0 } } });
    let err = install_profile(&ctx, &profile)
        .await
        .map(|_| ())
        .unwrap_err();
    assert!(err.contains("keepRecentTurns"), "{err}");
}

/// Reviewer scenario B1: keep two turns. After the first compaction the summary has a
/// higher seq than the turns kept before it, so on the very next turn the cut (an older
/// kept turn) is lower than the summary's seq. That compaction must still happen.
#[tokio::test]
async fn compaction_on_the_turn_after_a_compaction_still_replaces_older_kept_turns() {
    let big = BIG.repeat(8);
    let rig = rig(
        json!({ "triggerTokens": 50, "keepRecentTurns": 2, "minReclaimTokens": 1 }),
        vec![
            Ok("r1".into()),
            Ok("r2".into()),
            Ok("r3".into()),
            Ok("S1".into()),
            Ok("r4".into()), // turn 4: compacts turn 1
            Ok("S2".into()),
            Ok("r5".into()), // turn 5: must compact turn 2
        ],
    )
    .await;
    for (i, label) in ["first", "second", "third", "fourth", "fifth"]
        .iter()
        .enumerate()
    {
        say(&rig, &format!("u{i}"), &format!("{label} {big}")).await;
    }
    let log = compactions(&rig);
    assert_eq!(log.len(), 2, "turn 5 compacted again: {log:?}");
    let visible: Vec<String> = rig
        .agent
        .session()
        .derive_messages()
        .iter()
        .map(|m| m.text())
        .collect();
    assert_eq!(
        visible
            .iter()
            .filter(|t| t.contains("[Summary of the earlier"))
            .count(),
        1,
        "{visible:?}"
    );
    assert!(visible[0].contains("S2"), "{visible:?}");
    assert!(
        !visible
            .iter()
            .any(|t| t.starts_with("first") || t.starts_with("second")),
        "{visible:?}"
    );
}

#[tokio::test]
async fn the_compact_tool_folds_history_on_demand_and_passes_the_focus_hint() {
    let big = BIG.repeat(8);
    let rig = rig(
        // Far above the history size, so only the tool can trigger a compaction.
        json!({ "triggerTokens": 1_000_000, "keepRecentTurns": 1, "minReclaimTokens": 1 }),
        vec![
            Ok("answer one".into()),
            Ok("answer two".into()),
            Ok("SUMMARY: kept the regression test".into()),
        ],
    )
    .await;
    say(&rig, "u1", &format!("first {big}")).await;
    say(&rig, "u2", &format!("second {big}")).await;
    assert!(
        compactions(&rig).is_empty(),
        "auto-compaction stayed out of the way"
    );

    let tools = rig
        .ctx
        .require::<dsh_rs::api::services::ToolsService>(dsh_rs::api::TOOLS_SERVICE)
        .unwrap();
    assert!(
        tools.list().contains(&"compact".to_string()),
        "compact is registered"
    );
    let result = tools
        .execute(
            "call-1".to_string(),
            "compact".to_string(),
            json!({ "focus": "keep the regression test" }),
            dsh_rs::types::ToolRunContext {
                ctx: rig.ctx.clone(),
                signal: Default::default(),
                agent_id: Some(rig.agent.id().to_string()),
                cwd: None,
            },
        )
        .await;
    assert!(!result.is_error(), "{:?}", result.content());
    assert_eq!(compactions(&rig).len(), 1, "the tool compacted on demand");

    let seen = rig.seen.lock().unwrap();
    let summary_request = seen.last().expect("the summary request was recorded");
    assert!(
        summary_request
            .system
            .as_deref()
            .unwrap_or("")
            .contains("keep the regression test"),
        "the focus hint reaches the summariser"
    );
}

#[tokio::test]
async fn compact_reports_an_error_when_it_has_no_agent() {
    let rig = rig(tight(), vec![]).await;
    let tools = rig
        .ctx
        .require::<dsh_rs::api::services::ToolsService>(dsh_rs::api::TOOLS_SERVICE)
        .unwrap();
    let result = tools
        .execute(
            "call-2".to_string(),
            "compact".to_string(),
            json!({}),
            dsh_rs::types::ToolRunContext {
                ctx: rig.ctx.clone(),
                signal: Default::default(),
                agent_id: None,
                cwd: None,
            },
        )
        .await;
    assert!(
        result.is_error(),
        "compact refuses to guess whose history to fold"
    );
}
