//! Goal tools end to end, including how a goal survives compaction.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cordis::Context;
use dsh_rs::api::services::{
    AgentRegistryService, AgentView, BoxFuture, LlmAdapterApi, LlmService,
};
use dsh_rs::bundle::install_profile;
use dsh_rs::types::{
    stream_from_chunks, AgentOptions, BoxStream, ContentBlock, GenerateOptions, GoalStatus,
    LlmError, Message, SessionEventData, StreamChunk,
};
use serde_json::{json, Value};

mod common;
use common::{text_response, tool_call_response};

#[derive(Clone)]
enum Reply {
    Text(&'static str),
    Tool(&'static str, Value),
}

struct Script {
    replies: Mutex<VecDeque<Reply>>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

impl LlmAdapterApi for Script {
    fn name(&self) -> &'static str {
        "script"
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
            .unwrap_or(Reply::Text("ok"));
        Box::pin(async move {
            Ok(stream_from_chunks(match reply {
                Reply::Text(t) => text_response(t),
                Reply::Tool(name, args) => tool_call_response("call-1", name, args),
            }))
        })
    }
}

struct Rig {
    agent: Arc<dyn AgentView>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

async fn rig(plugins: Value, replies: Vec<Reply>) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": plugins }))
        .await
        .expect("profile installs");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(
        &["mock"],
        Arc::new(Script {
            replies: Mutex::new(replies.into()),
            seen: seen.clone(),
        }),
    )
    .unwrap();
    let agents = ctx.require::<AgentRegistryService>("agents").unwrap();
    let agent = agents
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
    std::mem::forget(ctx); // keep plugins alive for the test's duration
    Rig { agent, seen }
}

async fn say(rig: &Rig, id: &str, text: &str) {
    rig.agent
        .followup(Message::user(id, vec![ContentBlock::text(text)]));
    rig.agent.when_idle().await;
}

fn goal_events(rig: &Rig) -> Vec<(String, GoalStatus, Option<String>)> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|e| match e.data {
            SessionEventData::Goal {
                objective,
                status,
                note,
            } => Some((objective, status, note)),
            _ => None,
        })
        .collect()
}

fn last_tool_result(rig: &Rig) -> (String, bool) {
    rig.agent
        .session()
        .events()
        .into_iter()
        .rev()
        .find_map(|e| match e.data {
            SessionEventData::ToolResult { message, .. } => {
                message.content.into_iter().find_map(|b| match b {
                    ContentBlock::ToolResult {
                        content, is_error, ..
                    } => Some((
                        content
                            .iter()
                            .filter_map(|c| c.as_text())
                            .collect::<String>(),
                        is_error == Some(true),
                    )),
                    _ => None,
                })
            }
            _ => None,
        })
        .expect("a tool result")
}

fn reminders(options: &GenerateOptions) -> usize {
    options
        .messages
        .iter()
        .filter(|m| m.text().starts_with("[Session goal:"))
        .count()
}

#[tokio::test]
async fn set_get_update_walk_the_goal_through_its_life_and_every_step_is_logged() {
    let rig = rig(
        json!({ "goal": {} }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
            Reply::Tool("goal_get", json!({})),
            Reply::Text("got"),
            Reply::Tool(
                "goal_update",
                json!({ "status": "achieved", "note": "all green" }),
            ),
            Reply::Text("closed"),
        ],
    )
    .await;
    say(&rig, "u1", "set a goal").await;
    assert_eq!(
        goal_events(&rig),
        [("make the tests pass".into(), GoalStatus::Active, None)]
    );

    say(&rig, "u2", "what is it").await;
    assert!(last_tool_result(&rig)
        .0
        .contains("Goal (active): make the tests pass"));

    say(&rig, "u3", "finish").await;
    let events = goal_events(&rig);
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[1],
        (
            "make the tests pass".into(),
            GoalStatus::Achieved,
            Some("all green".into())
        )
    );
}

#[tokio::test]
async fn a_second_goal_needs_replace_and_a_closed_goal_cannot_be_updated() {
    let rig = rig(
        json!({ "goal": {} }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "first goal" })),
            Reply::Text("a"),
            Reply::Tool("goal_set", json!({ "objective": "second goal" })),
            Reply::Text("b"),
            Reply::Tool(
                "goal_set",
                json!({ "objective": "second goal", "replace": true }),
            ),
            Reply::Text("c"),
            Reply::Tool("goal_update", json!({ "status": "abandoned" })),
            Reply::Text("d"),
            Reply::Tool("goal_update", json!({ "status": "achieved" })),
            Reply::Text("e"),
            Reply::Tool("goal_set", json!({ "objective": "third goal" })),
            Reply::Text("f"),
        ],
    )
    .await;
    say(&rig, "u1", "1").await;
    say(&rig, "u2", "2").await;
    let (text, is_error) = last_tool_result(&rig);
    assert!(
        is_error && text.contains("already active") && text.contains("first goal"),
        "{text}"
    );
    assert_eq!(
        goal_events(&rig).len(),
        1,
        "the refused call logged nothing"
    );

    say(&rig, "u3", "3").await;
    say(&rig, "u4", "4").await;
    say(&rig, "u5", "5").await;
    let (text, is_error) = last_tool_result(&rig);
    assert!(is_error && text.contains("already abandoned"), "{text}");

    say(&rig, "u6", "6").await;
    let events = goal_events(&rig);
    assert_eq!(
        events.last().unwrap().0,
        "third goal",
        "a closed goal can be followed by a new one"
    );
    assert_eq!(events.last().unwrap().1, GoalStatus::Active);
}

#[tokio::test]
async fn update_without_a_goal_and_bad_input_are_errors() {
    let rig = rig(
        json!({ "goal": { "maxObjectiveChars": 20 } }),
        vec![
            Reply::Tool("goal_update", json!({ "status": "achieved" })),
            Reply::Text("a"),
            Reply::Tool("goal_set", json!({ "objective": "x".repeat(21) })),
            Reply::Text("b"),
            Reply::Tool("goal_set", json!({ "objective": "   " })),
            Reply::Text("c"),
            Reply::Tool("goal_get", json!({})),
            Reply::Text("d"),
        ],
    )
    .await;
    say(&rig, "u1", "1").await;
    assert!(last_tool_result(&rig).0.contains("no goal to update"));
    say(&rig, "u2", "2").await;
    assert!(last_tool_result(&rig).0.contains("longer than 20"));
    say(&rig, "u3", "3").await;
    assert!(last_tool_result(&rig).0.contains("objective is required"));
    say(&rig, "u4", "4").await;
    assert!(last_tool_result(&rig).0.contains("no goal is set"));
    assert!(goal_events(&rig).is_empty());
}

#[tokio::test]
async fn the_reminder_is_added_once_then_not_repeated_and_not_for_a_closed_goal() {
    let rig = rig(
        json!({ "goal": {} }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "ship the release" })),
            Reply::Text("a"),
            Reply::Text("b"),
            Reply::Text("c"),
            Reply::Tool("goal_update", json!({ "status": "achieved" })),
            Reply::Text("d"),
            Reply::Text("e"),
        ],
    )
    .await;
    say(&rig, "u1", "set it").await;
    say(&rig, "u2", "continue").await;
    say(&rig, "u3", "continue more").await;
    {
        let seen = rig.seen.lock().unwrap();
        // requests: 0,1 = turn 1 (no goal yet); 2 = turn 2; 3 = turn 3.
        assert_eq!(reminders(&seen[0]), 0);
        assert_eq!(
            reminders(&seen[2]),
            1,
            "added on the first turn after the goal was set"
        );
        assert_eq!(
            reminders(&seen[3]),
            1,
            "still exactly one, not one per turn"
        );
    }
    say(&rig, "u4", "wrap up").await; // closes the goal
    say(&rig, "u5", "after").await;
    let seen = rig.seen.lock().unwrap();
    let last = seen.last().unwrap();
    assert!(
        last.messages
            .iter()
            .any(|m| m.text().contains("ship the release")),
        "history still holds the old reminder"
    );
    let log_reminders = rig
        .agent
        .session()
        .derive_messages()
        .iter()
        .filter(|m| m.text().starts_with("[Session goal:"))
        .count();
    assert_eq!(
        log_reminders, 1,
        "no second reminder was logged after the goal closed"
    );
}

#[tokio::test]
async fn the_goal_comes_back_after_compaction_folds_the_reminder_away() {
    let big = "0123456789 padding text for the history. ".repeat(10);
    let rig = rig(
        json!({
            "goal": {},
            "compaction": { "triggerTokens": 60, "keepRecentTurns": 1, "minReclaimTokens": 1 }
        }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "migrate the database" })),
            Reply::Text("a"),
            Reply::Text("b"),
            Reply::Text("c"),
            Reply::Text("SUMMARY"), // compaction's summary call
            Reply::Text("d"),
        ],
    )
    .await;
    say(&rig, "u1", &format!("set the goal {big}")).await;
    say(&rig, "u2", &format!("work on it {big}")).await;
    say(&rig, "u3", &format!("more work {big}")).await;
    say(&rig, "u4", "next").await;

    let compacted = rig
        .agent
        .session()
        .events()
        .iter()
        .any(|e| matches!(e.data, SessionEventData::Compaction { .. }));
    assert!(compacted, "compaction ran");
    let seen = rig.seen.lock().unwrap();
    let last = seen.last().unwrap();
    assert_eq!(
        reminders(last),
        1,
        "the goal reaches the model again: {:?}",
        last.messages.iter().map(|m| m.text()).collect::<Vec<_>>()
    );
    // And the goal itself is still the current state in the log.
    assert_eq!(goal_events(&rig).last().unwrap().0, "migrate the database");
}

#[tokio::test]
async fn settings_are_validated_and_reminders_can_be_turned_off() {
    let ctx = Context::new();
    let err = install_profile(
        &ctx,
        &json!({ "plugins": { "goal": { "maxNoteChars": 3 } } }),
    )
    .await
    .map(|_| ())
    .unwrap_err();
    assert!(err.contains("maxNoteChars"), "{err}");

    let rig = rig(
        json!({ "goal": { "remind": false } }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "quiet goal" })),
            Reply::Text("a"),
            Reply::Text("b"),
        ],
    )
    .await;
    say(&rig, "u1", "set").await;
    say(&rig, "u2", "go on").await;
    assert_eq!(reminders(rig.seen.lock().unwrap().last().unwrap()), 0);
}

/// Reviewer scenario B3: the reminder must come back after a compaction folded it away
/// whichever plugin is registered first. Profile maps are sorted by name, so the two
/// orders are installed explicitly, one plugin at a time.
#[tokio::test]
async fn the_goal_returns_after_compaction_in_either_plugin_order() {
    use dsh_rs::bundle::install_native_plugins;
    let big = "0123456789 padding text for the history. ".repeat(10);
    let compaction = json!({ "triggerTokens": 60, "keepRecentTurns": 1, "minReclaimTokens": 1 });
    for first_is_goal in [true, false] {
        let ctx = Context::new();
        install_profile(&ctx, &json!({}))
            .await
            .expect("base installs");
        let (a, b) = if first_is_goal {
            (
                json!({ "goal": {} }),
                json!({ "compaction": compaction.clone() }),
            )
        } else {
            (
                json!({ "compaction": compaction.clone() }),
                json!({ "goal": {} }),
            )
        };
        install_native_plugins(&ctx, &a)
            .await
            .expect("first installs");
        install_native_plugins(&ctx, &b)
            .await
            .expect("second installs");

        let seen = Arc::new(Mutex::new(Vec::new()));
        let replies = vec![
            Reply::Tool("goal_set", json!({ "objective": "migrate the database" })),
            Reply::Text("a"),
            Reply::Text("b"),
            Reply::Text("c"),
            Reply::Text("SUMMARY"),
            Reply::Text("d"),
        ];
        let llm = ctx.require::<LlmService>("llm").unwrap();
        llm.unregister_adapter(&["mock"]);
        llm.register_adapter(
            &["mock"],
            Arc::new(Script {
                replies: Mutex::new(replies.into()),
                seen: seen.clone(),
            }),
        )
        .unwrap();
        let agents = ctx.require::<AgentRegistryService>("agents").unwrap();
        let agent = agents
            .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
            .unwrap();
        let rig = Rig { agent, seen };

        say(&rig, "u1", &format!("set the goal {big}")).await;
        say(&rig, "u2", &format!("work on it {big}")).await;
        say(&rig, "u3", &format!("more work {big}")).await;
        say(&rig, "u4", "next").await;
        let order = if first_is_goal {
            "goal first"
        } else {
            "compaction first"
        };
        assert!(
            rig.agent
                .session()
                .events()
                .iter()
                .any(|e| matches!(e.data, SessionEventData::Compaction { .. })),
            "{order}: compaction ran"
        );
        assert_eq!(
            reminders(rig.seen.lock().unwrap().last().unwrap()),
            1,
            "{order}: the goal reached the model again"
        );
        std::mem::forget(ctx);
    }
}

// --- independent verification of an `achieved` claim -------------------------------------------

fn verifying() -> Value {
    json!({ "goal": { "verify": true } })
}

/// Requests whose system prompt marks them as the verifier's.
fn verifier_calls(rig: &Rig) -> Vec<GenerateOptions> {
    rig.seen
        .lock()
        .unwrap()
        .iter()
        .filter(|o| {
            o.system
                .as_deref()
                .is_some_and(|s| s.contains("independent verifier"))
        })
        .cloned()
        .collect()
}

async fn claim_achieved(replies: Vec<Reply>) -> Rig {
    let mut script = vec![
        Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
        Reply::Text("set"),
        Reply::Tool(
            "goal_update",
            json!({ "status": "achieved", "note": "all 12 tests pass" }),
        ),
    ];
    script.extend(replies);
    let rig = rig(verifying(), script).await;
    say(&rig, "u1", "set a goal").await;
    say(&rig, "u2", "finish up").await;
    rig
}

fn goal_status(rig: &Rig) -> GoalStatus {
    goal_events(rig).last().expect("a goal event").1
}

#[tokio::test]
async fn a_verified_claim_closes_the_goal() {
    let rig = claim_achieved(vec![Reply::Text("VERDICT: MET"), Reply::Text("done")]).await;
    assert_eq!(goal_status(&rig), GoalStatus::Achieved);
    assert_eq!(verifier_calls(&rig).len(), 1);
}

#[tokio::test]
async fn a_rejected_claim_keeps_the_goal_active_and_tells_the_model_why() {
    let rig = claim_achieved(vec![
        Reply::Text("VERDICT: NOT_MET\nNo test run appears in the transcript."),
        Reply::Text("understood"),
    ])
    .await;
    assert_eq!(
        goal_status(&rig),
        GoalStatus::Active,
        "no closing event was logged"
    );
    let (text, is_error) = last_tool_result(&rig);
    assert!(is_error);
    assert!(text.contains("independent check did not accept"), "{text}");
    assert!(
        text.contains("No test run appears"),
        "the reason reaches the model: {text}"
    );
}

#[tokio::test]
async fn an_unparseable_verdict_fails_closed() {
    // Mentions both words, but the first line is not a verdict.
    for reply in [
        "The goal looks met to me, VERDICT: MET",
        "",
        "I cannot decide.",
        "VERDICT: MET-ish",
    ] {
        let rig = claim_achieved(vec![Reply::Text(reply), Reply::Text("ok")]).await;
        assert_eq!(
            goal_status(&rig),
            GoalStatus::Active,
            "reply {reply:?} must not close the goal"
        );
        assert!(
            last_tool_result(&rig).0.contains("could not be verified"),
            "reply {reply:?}"
        );
    }
}

#[tokio::test]
async fn the_verifier_sees_the_claim_and_transcript_but_gets_no_tools() {
    let rig = claim_achieved(vec![Reply::Text("VERDICT: MET"), Reply::Text("done")]).await;
    let calls = verifier_calls(&rig);
    let request = &calls[0];
    assert!(
        request.tools.is_none(),
        "the verifier must not be able to act"
    );
    let body = request.messages[0].text();
    assert!(body.contains("make the tests pass"), "objective: {body}");
    // The note also sits in the transcript (as the tool call's arguments), so
    // check the claim section itself.
    assert!(
        body.contains("<claim>\nall 12 tests pass\n</claim>"),
        "claim: {body}"
    );
    assert!(body.contains("finish up"), "transcript: {body}");
    assert_eq!(request.temperature, Some(0.0));
}

#[tokio::test]
async fn abandoning_and_progress_notes_are_not_verified() {
    let rig = rig(
        verifying(),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
            Reply::Tool(
                "goal_update",
                json!({ "status": "active", "note": "halfway" }),
            ),
            Reply::Text("noted"),
            Reply::Tool(
                "goal_update",
                json!({ "status": "abandoned", "note": "blocked" }),
            ),
            Reply::Text("closed"),
        ],
    )
    .await;
    say(&rig, "u1", "go").await;
    say(&rig, "u2", "progress").await;
    say(&rig, "u3", "give up").await;
    assert_eq!(goal_status(&rig), GoalStatus::Abandoned);
    assert!(
        verifier_calls(&rig).is_empty(),
        "only an `achieved` claim is checked"
    );
}

#[tokio::test]
async fn verification_is_off_unless_asked_for() {
    let rig = rig(
        json!({ "goal": {} }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
            Reply::Tool("goal_update", json!({ "status": "achieved" })),
            Reply::Text("closed"),
        ],
    )
    .await;
    say(&rig, "u1", "go").await;
    say(&rig, "u2", "finish").await;
    assert_eq!(goal_status(&rig), GoalStatus::Achieved);
    assert!(verifier_calls(&rig).is_empty());
}

#[tokio::test]
async fn half_a_verifier_model_is_refused_at_load() {
    let ctx = Context::new();
    let outcome = install_profile(
        &ctx,
        &json!({ "plugins": { "goal": { "verifierProvider": "openai" } } }),
    )
    .await;
    let error = format!("{:?}", outcome.err().expect("the profile must not install"));
    assert!(
        error.contains("verifierProvider and verifierModel"),
        "{error}"
    );
}

// --- bounded continuation ----------------------------------------------------------------------

fn rounding(max: u32) -> Value {
    json!({ "goal": { "maxRounds": max } })
}

fn round_messages(rig: &Rig) -> Vec<String> {
    rig.agent
        .session()
        .derive_messages()
        .iter()
        .filter(|m| m.text().starts_with("[Goal round"))
        .map(|m| m.text())
        .collect()
}

/// Wait until the agent has been idle and nothing more arrives for a moment.
async fn settle(rig: &Rig) {
    for _ in 0..40 {
        rig.agent.when_idle().await;
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        if rig.agent.status() == dsh_rs::types::AgentStatus::Idle && !rig.agent.driver_busy() {
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            if !rig.agent.driver_busy() {
                return;
            }
        }
    }
}

#[tokio::test]
async fn an_active_goal_is_continued_until_the_round_cap_and_then_stops_visibly() {
    let rig = rig(
        rounding(2),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
        ],
    )
    .await;
    rig.agent
        .followup(Message::user("u1", vec![ContentBlock::text("set a goal")]));
    settle(&rig).await;

    let rounds = round_messages(&rig);
    assert_eq!(rounds.len(), 2, "exactly the cap: {rounds:?}");
    assert!(rounds[0].contains("round 1 of 2") && rounds[1].contains("round 2 of 2"));
    assert!(
        rounds[0].contains("\"make the tests pass\""),
        "objective is quoted: {}",
        rounds[0]
    );

    let last = goal_events(&rig).pop().unwrap();
    assert_eq!(last.1, GoalStatus::Active);
    assert!(
        last.2.unwrap().contains("round limit reached"),
        "the stop is recorded"
    );
    let exhausted = goal_events(&rig)
        .iter()
        .filter(|e| e.2.as_deref().is_some_and(|n| n.contains("round limit")))
        .count();
    assert_eq!(exhausted, 1, "recorded once, not on every idle");
}

#[tokio::test]
async fn a_goal_closed_by_the_model_is_not_continued() {
    let rig = rig(
        rounding(5),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
            // Round 1: the model finishes the goal.
            Reply::Tool(
                "goal_update",
                json!({ "status": "achieved", "note": "done" }),
            ),
            Reply::Text("closed"),
        ],
    )
    .await;
    rig.agent
        .followup(Message::user("u1", vec![ContentBlock::text("go")]));
    settle(&rig).await;
    assert_eq!(
        round_messages(&rig).len(),
        1,
        "one round, then the goal closed"
    );
    assert_eq!(goal_status(&rig), GoalStatus::Achieved);
}

#[tokio::test]
async fn continuation_is_off_by_default() {
    let rig = rig(
        json!({ "goal": {} }),
        vec![
            Reply::Tool("goal_set", json!({ "objective": "make the tests pass" })),
            Reply::Text("set"),
        ],
    )
    .await;
    rig.agent
        .followup(Message::user("u1", vec![ContentBlock::text("go")]));
    settle(&rig).await;
    assert!(round_messages(&rig).is_empty());
}
