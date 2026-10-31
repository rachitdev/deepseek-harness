//! The approvals plugin gating real tool calls through the registry.

use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::ToolsService;
use dsh_rs::bundle::install_profile;
use dsh_rs::plugins::approvals::{ApprovalBroker, ApprovalDecision, APPROVALS_SERVICE};
use dsh_rs::types::{CancelToken, ToolExecutionResult, ToolRunContext};
use serde_json::{json, Value};

async fn boot(approvals: Value) -> (Context, ToolsService, ApprovalBroker) {
    let ctx = Context::new();
    let profile = json!({ "plugins": { "approvals": approvals } });
    install_profile(&ctx, &profile).await.expect("profile installs");
    let tools = (*ctx.require::<ToolsService>("tools").unwrap()).clone();
    let broker = (*ctx.require::<ApprovalBroker>(APPROVALS_SERVICE).unwrap()).clone();
    (ctx, tools, broker)
}

async fn call(ctx: &Context, tools: &ToolsService, name: &str, args: Value) -> ToolExecutionResult {
    let run = ToolRunContext {
        ctx: ctx.clone(),
        signal: CancelToken::new(),
        agent_id: Some("a1".into()),
        cwd: Some("/tmp".into()),
    };
    tools.execute("c1".into(), name.into(), args, run).await
}

fn error_of(result: &ToolExecutionResult) -> (String, String) {
    match result {
        ToolExecutionResult::Error { code, message, .. } => (code.clone(), message.clone()),
        other => panic!("expected an error, got {other:?}"),
    }
}

#[tokio::test]
async fn an_ask_tool_without_an_approver_is_denied_and_never_runs() {
    let marker = std::env::temp_dir().join(format!("dsh-approvals-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let (ctx, tools, _broker) = boot(json!({})).await;
    let command = format!("touch {}", marker.display());
    let result = call(&ctx, &tools, "bash", json!({ "command": command })).await;
    let (code, message) = error_of(&result);
    assert_eq!(code, "DENIED");
    assert!(message.contains("no interactive approver"), "{message}");
    assert!(!marker.exists(), "a denied command must not have run");
}

#[tokio::test]
async fn an_approver_can_allow_once_allow_for_the_run_or_deny() {
    let (ctx, tools, broker) = boot(json!({ "ask": ["bash"], "timeoutMs": 5000 })).await;
    let mut inbox = broker.attach();

    // Allow once: runs, and asks again next time.
    let first = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move { call(&ctx, &tools, "bash", json!({ "command": "echo one" })).await }
    });
    let q = inbox.questions.recv().await.unwrap();
    assert_eq!(q.tool, "bash");
    assert_eq!(q.agent.as_deref(), Some("a1"));
    assert!(q.arguments.contains("echo one"));
    q.answer(ApprovalDecision::Allow);
    assert!(!first.await.unwrap().is_error());

    // Deny: the tool does not run and the model is told why.
    let second = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move { call(&ctx, &tools, "bash", json!({ "command": "echo two" })).await }
    });
    inbox.questions.recv().await.unwrap().answer(ApprovalDecision::Deny);
    let (code, message) = error_of(&second.await.unwrap());
    assert_eq!(code, "DENIED");
    assert!(message.contains("denied by the user"), "{message}");

    // Allow for the run: the next call is not asked.
    let third = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move { call(&ctx, &tools, "bash", json!({ "command": "echo three" })).await }
    });
    inbox.questions.recv().await.unwrap().answer(ApprovalDecision::AllowForRun);
    assert!(!third.await.unwrap().is_error());
    let fourth = call(&ctx, &tools, "bash", json!({ "command": "echo four" })).await;
    assert!(!fourth.is_error(), "remembered for the run");
    assert!(inbox.questions.try_recv().is_err(), "no further question was asked");
}

#[tokio::test]
async fn deny_and_allow_lists_apply_without_asking() {
    let (ctx, tools, broker) =
        boot(json!({ "ask": [], "deny": ["write_file"], "allow": ["bash"] })).await;
    let mut inbox = broker.attach();
    let denied = call(&ctx, &tools, "write_file", json!({ "path": "/tmp/x", "content": "x" })).await;
    assert!(error_of(&denied).1.contains("denied by the approvals policy"));
    let allowed = call(&ctx, &tools, "bash", json!({ "command": "echo ok" })).await;
    assert!(!allowed.is_error());
    assert!(inbox.questions.try_recv().is_err());
}

#[tokio::test]
async fn an_unanswered_question_times_out_and_denies() {
    let (ctx, tools, broker) = boot(json!({ "ask": ["bash"], "timeoutMs": 1000 })).await;
    let _inbox = broker.attach();
    let started = std::time::Instant::now();
    let result = call(&ctx, &tools, "bash", json!({ "command": "echo hi" })).await;
    let (_, message) = error_of(&result);
    assert!(message.contains("no answer"), "{message}");
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
async fn a_contradictory_policy_fails_the_profile_load() {
    let ctx = Context::new();
    let profile = json!({ "plugins": { "approvals": { "ask": ["bash"], "allow": ["bash"] } } });
    let err = install_profile(&ctx, &profile).await.map(|_| ()).unwrap_err();
    assert!(err.contains("overlaps"), "{err}");
}
