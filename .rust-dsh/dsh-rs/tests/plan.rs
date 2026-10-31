//! Plan mode: the `/plan` command, the `plan` projection, how a choice made
//! during a turn is applied at the next step, the guidance in the system
//! prompt, and the `exit_plan_mode` review.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cordis::plugin::BoxFuture;
use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, AgentView, LlmService, ToolsService};
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::llm::{LlmAdapter, LlmError, MessageSource, StreamChunk};
use dsh_rs::plugins::ask_user::{Answer, QuestionBroker, Reply, USER_QUESTIONS_SERVICE};
use dsh_rs::session::SessionEventData;
use dsh_rs::types::{
    AgentOptions, CancelToken, ContentBlock, ContextForm, GenerateOptions, Message,
    ToolExecutionResult, ToolRunContext,
};
use serde_json::{json, Value};

mod common;

const TOKEN: &str = "plan-token-0123456789";
const GUIDANCE: &str = "PLAN-GUIDANCE: explore, then call exit_plan_mode.";

/// Replays one chunk sequence per request, recording every request; each
/// request waits for a permit when a gate is set.
struct Recorder {
    script: Mutex<VecDeque<Vec<StreamChunk>>>,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
    gate: Option<Arc<tokio::sync::Semaphore>>,
}

impl LlmAdapter for Recorder {
    fn name(&self) -> &'static str {
        "recorder"
    }

    fn stream(
        &self,
        options: GenerateOptions,
    ) -> BoxFuture<Result<dsh_rs::llm::BoxStream<StreamChunk>, LlmError>> {
        self.seen.lock().unwrap().push(options);
        let chunks = self
            .script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| common::text_response("done"));
        let gate = self.gate.clone();
        Box::pin(async move {
            if let Some(gate) = gate {
                gate.acquire().await.expect("the gate stays open").forget();
            }
            Ok(dsh_rs::llm::runtime::stream_from_chunks(chunks))
        })
    }
}

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    agent: Arc<dyn AgentView>,
    tools: ToolsService,
    seen: Arc<Mutex<Vec<GenerateOptions>>>,
}

async fn rig(plugins: Value, script: Vec<Vec<StreamChunk>>, gate: Option<Arc<tokio::sync::Semaphore>>) -> Rig {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": plugins })).await.unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let llm = ctx.require::<LlmService>("llm").unwrap();
    llm.unregister_adapter(&["mock"]);
    llm.register_adapter(
        &["mock"],
        Arc::new(Recorder { script: Mutex::new(script.into()), seen: seen.clone(), gate }),
    )
    .unwrap();
    let agent = ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
    let router = Router::new(RouterConfig {
        default_model: DefaultModel { provider: "mock".into(), model: "m".into() },
        ..RouterConfig::default()
    });
    let gateway = Arc::new(Gateway::new(router, Auth { token: TOKEN.into() }, None));
    let tools = (*ctx.require::<ToolsService>("tools").unwrap()).clone();
    Rig { ctx, gateway, agent, tools, seen }
}

fn plan_profile() -> Value {
    json!({ "plan": { "section": GUIDANCE }, "ask-user": {} })
}

async fn call(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &http::Method::POST, "/api", &headers, &body)
        .await;
    let envelope: Value = serde_json::from_slice(&reply.body).expect("the reply is JSON");
    envelope["result"].clone()
}

async fn command(rig: &Rig, line: &str) -> (String, String) {
    let reply = call(
        rig,
        "commands/execute",
        json!({ "agentId": rig.agent.id(), "line": line, "submittedAttachments": [] }),
    )
    .await;
    assert_eq!(reply["ok"], true, "{reply}");
    let result = &reply["value"]["result"];
    assert!(result.is_object(), "no result for {line}: {reply}");
    (
        result["kind"].as_str().unwrap().to_string(),
        result["text"].as_str().unwrap_or("").to_string(),
    )
}

async fn plan_projection(rig: &Rig) -> Value {
    let reply = call(
        rig,
        "session/projections",
        json!({ "request": { "sessionId": rig.agent.id() } }),
    )
    .await;
    reply["value"]["values"]["plan"].clone()
}

fn plan_events(rig: &Rig) -> Vec<bool> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|event| match event.data {
            SessionEventData::PlanMode { active } => Some(active),
            _ => None,
        })
        .collect()
}

fn prompt(rig: &Rig, id: &str, text: &str) {
    rig.agent.followup(Message::user(id, vec![ContentBlock::text(text)]));
}

fn system_of(options: &GenerateOptions) -> String {
    options.system.clone().unwrap_or_default()
}

/// The notices the plan plugin put in the log.
fn notices(rig: &Rig) -> Vec<String> {
    rig.agent
        .session()
        .events()
        .into_iter()
        .filter_map(|event| match event.data {
            SessionEventData::UserMessage { message } => match &message.source {
                MessageSource::Plugin { plugin, form: Some(ContextForm::Notice { summary }) }
                    if plugin == "plan-mode" =>
                {
                    assert_eq!(&message.text(), summary, "the notice is its own summary");
                    Some(summary.clone())
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn run_ctx(rig: &Rig) -> ToolRunContext {
    ToolRunContext {
        ctx: rig.ctx.clone(),
        signal: CancelToken::new(),
        agent_id: Some(rig.agent.id().to_string()),
        cwd: None,
    }
}

async fn exit_plan(rig: &Rig, plan: &str) -> ToolExecutionResult {
    rig.tools
        .execute("c1".into(), "exit_plan_mode".into(), json!({ "plan": plan }), run_ctx(rig))
        .await
}

fn error_of(result: &ToolExecutionResult) -> (String, String) {
    match result {
        ToolExecutionResult::Error { code, message, .. } => (code.clone(), message.clone()),
        other => panic!("expected an error, got {other:?}"),
    }
}

#[tokio::test]
async fn without_the_plugin_there_is_no_command_and_no_tool() {
    let rig = rig(json!({}), vec![], None).await;
    let list = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    assert!(list["value"].as_array().unwrap().iter().all(|item| item["name"] != "plan"));
    assert!(!rig.tools.list().contains(&"exit_plan_mode".to_string()));
    assert_eq!(plan_projection(&rig).await, json!({ "active": false, "pending": false }));
}

#[tokio::test]
async fn the_command_is_listed_like_the_typescript_one() {
    let rig = rig(plan_profile(), vec![], None).await;
    let list = call(&rig, "commands/list", json!({ "agentId": rig.agent.id() })).await;
    let item = list["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "plan")
        .expect("offered")
        .clone();
    assert_eq!(item["definitionId"], "@deepseek-ai/dsh-plan-mode");
    assert_eq!(item["description"], "Enter or leave plan mode");
    assert_eq!(item["input"], json!({ "hint": "[off|message]", "attachments": true }));
    assert!(rig.tools.list().contains(&"exit_plan_mode".to_string()));
}

#[tokio::test]
async fn an_idle_session_switches_at_once() {
    let rig = rig(plan_profile(), vec![], None).await;
    assert_eq!(
        command(&rig, "/plan").await,
        ("success".into(), "Plan mode on. Use /plan off to leave.".into())
    );
    assert_eq!(plan_events(&rig), [true]);
    assert_eq!(plan_projection(&rig).await, json!({ "active": true, "pending": false }));
    // Like the TypeScript host, only a committed switch reads "on".
    assert_eq!(
        command(&rig, "/plan").await.1,
        "Entering plan mode (applies from the next step). Use /plan off to leave."
    );
    assert_eq!(plan_events(&rig), [true], "entering twice logs once");
    assert_eq!(command(&rig, "/plan off").await.1, "Plan mode off.");
    assert_eq!(command(&rig, "/plan off").await.1, "Plan mode is already inactive.");
    assert_eq!(plan_events(&rig), [true, false]);
    assert_eq!(plan_projection(&rig).await, json!({ "active": false, "pending": false }));
}

#[tokio::test]
async fn off_refuses_attachments() {
    let rig = rig(plan_profile(), vec![], None).await;
    let reply = call(
        &rig,
        "commands/execute",
        json!({
            "agentId": rig.agent.id(), "line": "/plan off",
            "submittedAttachments": [{ "type": "file", "receiptId": "nope" }],
        }),
    )
    .await;
    // Attachments need a store; without one the command says so first.
    assert_eq!(reply["value"]["result"]["kind"], "error", "{reply}");
}

#[tokio::test]
async fn a_message_after_the_command_steers_the_agent() {
    let rig = rig(plan_profile(), vec![], None).await;
    command(&rig, "/plan map the auth flow").await;
    assert_eq!(plan_events(&rig), [true]);
    rig.agent.when_idle().await;
    let requests = rig.seen.lock().unwrap().clone();
    let last = requests.last().expect("the steering message started a request");
    assert!(last.messages.iter().any(|m| m.text() == "map the auth flow"));
    assert!(system_of(last).contains(GUIDANCE), "plan mode was on for that request");
}

#[tokio::test]
async fn the_guidance_is_in_the_prompt_only_while_plan_mode_is_on() {
    let rig = rig(plan_profile(), vec![], None).await;
    prompt(&rig, "u1", "hello");
    rig.agent.when_idle().await;
    command(&rig, "/plan").await;
    prompt(&rig, "u2", "again");
    rig.agent.when_idle().await;
    command(&rig, "/plan off").await;
    prompt(&rig, "u3", "once more");
    rig.agent.when_idle().await;
    let requests = rig.seen.lock().unwrap().clone();
    let has_guidance: Vec<bool> = requests.iter().map(|r| system_of(r).contains(GUIDANCE)).collect();
    assert_eq!(has_guidance, [false, true, false]);
}

async fn wait_for_requests(rig: &Rig, count: usize) {
    for _ in 0..300 {
        if rig.seen.lock().unwrap().len() >= count {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("the model was not asked {count} times");
}

#[tokio::test]
async fn a_choice_made_mid_turn_waits_for_the_next_step_and_is_narrated() {
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let script = vec![common::tool_call_response("t1", "glob", json!({ "pattern": "zzz-nothing-*" }))];
    let rig = rig(plan_profile(), script, Some(gate.clone())).await;
    prompt(&rig, "u1", "work");
    wait_for_requests(&rig, 1).await;
    // Step 1's request is held at the gate: the turn is open.
    let (_, text) = command(&rig, "/plan").await;
    assert_eq!(text, "Entering plan mode (applies from the next step). Use /plan off to leave.");
    assert!(plan_events(&rig).is_empty(), "nothing is logged until the next step");
    assert_eq!(plan_projection(&rig).await, json!({ "active": false, "pending": true }));

    gate.add_permits(8);
    rig.agent.when_idle().await;
    assert_eq!(plan_events(&rig), [true]);
    assert_eq!(notices(&rig), ["The user switched this session to plan mode."]);
    let requests = rig.seen.lock().unwrap().clone();
    assert!(!system_of(&requests[0]).contains(GUIDANCE), "step 1 ran as before");
    assert!(system_of(&requests[1]).contains(GUIDANCE), "step 2 runs in plan mode");
    assert!(
        requests[1].messages.iter().any(|m| m.text() == "The user switched this session to plan mode."),
        "the model is told, and the notice is in the log"
    );
    assert_eq!(plan_projection(&rig).await, json!({ "active": true, "pending": false }));
}

#[tokio::test]
async fn changing_your_mind_mid_turn_cancels_the_pending_choice() {
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let script = vec![common::tool_call_response("t1", "glob", json!({ "pattern": "zzz-nothing-*" }))];
    let rig = rig(plan_profile(), script, Some(gate.clone())).await;
    prompt(&rig, "u1", "work");
    wait_for_requests(&rig, 1).await;
    command(&rig, "/plan").await;
    let mode = rig.ctx.get::<dsh_rs::plugins::plan::PlanMode>(dsh_rs::plugins::plan::PLAN_SERVICE).unwrap();
    assert!(mode.has_pending(rig.agent.id()));
    assert_eq!(command(&rig, "/plan off").await.1, "Plan mode entry cancelled.");
    assert!(!mode.has_pending(rig.agent.id()), "a cancelled choice leaves nothing behind");
    assert_eq!(plan_projection(&rig).await, json!({ "active": false, "pending": false }));
    gate.add_permits(8);
    rig.agent.when_idle().await;
    assert!(plan_events(&rig).is_empty());
    assert!(notices(&rig).is_empty());
}

#[tokio::test]
async fn approving_the_plan_leaves_plan_mode_silently_at_the_next_step() {
    let plan = "# Ship it\n\n1. do the thing";
    let script = vec![
        common::tool_call_response("p1", "exit_plan_mode", json!({ "plan": plan })),
        common::text_response("on it"),
    ];
    let rig = rig(plan_profile(), script, None).await;
    command(&rig, "/plan").await;
    let broker = (*rig.ctx.require::<QuestionBroker>(USER_QUESTIONS_SERVICE).unwrap()).clone();
    let mut inbox = broker.attach();
    prompt(&rig, "u1", "plan it");
    let pending = tokio::time::timeout(std::time::Duration::from_secs(5), inbox.questions.recv())
        .await
        .expect("the review is shown")
        .expect("the inbox is open");
    let question = &pending.questions[0];
    assert_eq!(question["id"], "plan-review");
    assert_eq!(question["header"], "Plan review");
    assert_eq!(question["question"], "Approve this plan and leave plan mode?");
    assert_eq!(question["detail"], plan);
    assert_eq!(question["options"][0]["label"], "Approve");
    assert_eq!(question["options"][1]["label"], "Keep planning");
    assert_eq!(question["intent"]["kind"], "plan-review");
    assert_eq!(question["intent"]["approve"], "Approve");
    assert_eq!(question["intent"]["callId"], "p1");
    pending.answer(Reply::Answered(vec![Answer {
        id: "plan-review".into(),
        selected: vec!["Approve".into()],
        custom: None,
    }]));
    rig.agent.when_idle().await;

    assert_eq!(plan_events(&rig), [true, false]);
    assert!(notices(&rig).is_empty(), "the tool result already says so");
    assert_eq!(plan_projection(&rig).await, json!({ "active": false, "pending": false }));
    let requests = rig.seen.lock().unwrap().clone();
    assert!(system_of(&requests[0]).contains(GUIDANCE));
    assert!(!system_of(&requests[1]).contains(GUIDANCE), "implementation starts after approval");
}

#[tokio::test]
async fn keeping_planning_returns_the_feedback_and_stays_in_plan_mode() {
    let rig = rig(plan_profile(), vec![], None).await;
    command(&rig, "/plan").await;
    let broker = (*rig.ctx.require::<QuestionBroker>(USER_QUESTIONS_SERVICE).unwrap()).clone();
    let mut inbox = broker.attach();
    let review = tokio::spawn({
        let (tools, run) = (rig.tools.clone(), run_ctx(&rig));
        async move {
            tools
                .execute("c1".into(), "exit_plan_mode".into(), json!({ "plan": "# P\nbody" }), run)
                .await
        }
    });
    let pending = inbox.questions.recv().await.unwrap();
    pending.answer(Reply::Answered(vec![Answer {
        id: "plan-review".into(),
        selected: vec!["Keep planning".into()],
        custom: Some("cover rollback".into()),
    }]));
    let (code, message) = error_of(&review.await.unwrap());
    assert_eq!(code, "KEEP_PLANNING");
    assert_eq!(message, "The user chose to keep planning; their feedback: cover rollback");
    assert_eq!(plan_events(&rig), [true]);
}

#[tokio::test]
async fn the_exit_tool_refuses_what_it_cannot_review() {
    let rig = rig(plan_profile(), vec![], None).await;
    let (code, message) = error_of(&exit_plan(&rig, "# P").await);
    assert_eq!(code, "NOT_PLANNING");
    assert_eq!(message, "exit_plan_mode is only available in plan mode");

    command(&rig, "/plan").await;
    let (code, message) = error_of(&exit_plan(&rig, "no heading here").await);
    assert_eq!(code, "BAD_ARGS");
    assert!(message.contains("starting with a # heading"), "{message}");

    // Nobody is attached to answer.
    let (code, message) = error_of(&exit_plan(&rig, "# P").await);
    assert_eq!(code, "NO_PROVIDER");
    assert!(message.contains("answerer") || message.contains("answer"), "{message}");
}

#[tokio::test]
async fn without_ask_user_the_review_channel_is_missing() {
    let rig = rig(json!({ "plan": {} }), vec![], None).await;
    command(&rig, "/plan").await;
    let (code, message) = error_of(&exit_plan(&rig, "# P").await);
    assert_eq!(code, "NO_PROVIDER");
    assert!(message.contains("no user-questions channel"), "{message}");
}

#[tokio::test]
async fn an_empty_section_stops_the_profile() {
    let ctx = Context::new();
    let Err(err) = install_profile(&ctx, &json!({ "plugins": { "plan": { "section": "  " } } })).await else {
        panic!("an empty section must stop the profile");
    };
    assert!(err.contains("section"), "{err}");
}
