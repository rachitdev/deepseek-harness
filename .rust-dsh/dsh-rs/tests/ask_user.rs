//! The `ask_user_question` tool and its broker through the real tool registry.

use std::time::Duration;

use cordis::Context;
use dsh_rs::api::services::ToolsService;
use dsh_rs::bundle::install_profile;
use dsh_rs::plugins::ask_user::{Answer, QuestionBroker, Reply, USER_QUESTIONS_SERVICE};
use dsh_rs::types::{CancelToken, ToolExecutionResult, ToolRunContext};
use serde_json::{json, Value};

async fn boot(plugins: Value) -> (Context, ToolsService, QuestionBroker) {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": plugins }))
        .await
        .expect("profile installs");
    let tools = (*ctx.require::<ToolsService>("tools").unwrap()).clone();
    let broker = (*ctx
        .require::<QuestionBroker>(USER_QUESTIONS_SERVICE)
        .unwrap())
    .clone();
    (ctx, tools, broker)
}

async fn ask(
    ctx: &Context,
    tools: &ToolsService,
    agent: Option<&str>,
    signal: CancelToken,
    args: Value,
) -> ToolExecutionResult {
    let run = ToolRunContext {
        ctx: ctx.clone(),
        signal,
        agent_id: agent.map(String::from),
        cwd: None,
    };
    tools
        .execute("c1".into(), "ask_user_question".into(), args, run)
        .await
}

fn error_of(result: &ToolExecutionResult) -> (String, String) {
    match result {
        ToolExecutionResult::Error { code, message, .. } => (code.clone(), message.clone()),
        other => panic!("expected an error, got {other:?}"),
    }
}

fn one_question() -> Value {
    json!({ "questions": [{
        "id": "mode", "question": "Which mode?", "multi_select": false,
        "options": [{ "label": "fast" }, { "label": "safe", "description": "slower" }]
    }]})
}

#[tokio::test]
async fn the_answer_comes_back_as_the_tool_result() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {} })).await;
    let mut inbox = broker.attach();
    let call = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move { ask(&ctx, &tools, Some("a1"), CancelToken::new(), one_question()).await }
    });
    let pending = inbox.questions.recv().await.unwrap();
    assert_eq!(pending.agent.as_deref(), Some("a1"));
    // The browser sees the wire name `multiSelect`, not the model's `multi_select`.
    assert_eq!(pending.questions[0]["multiSelect"], false);
    assert_eq!(pending.questions[0]["options"][1]["description"], "slower");
    pending.answer(Reply::Answered(vec![Answer {
        id: "mode".into(),
        selected: vec!["safe".into()],
        custom: Some("but quietly".into()),
    }]));
    match call.await.unwrap() {
        ToolExecutionResult::Success { value, .. } => assert_eq!(
            value,
            json!({ "answers": [{ "id": "mode", "selected": ["safe"], "custom": "but quietly" }] })
        ),
        other => panic!("expected success, got {other:?}"),
    }
}

#[tokio::test]
async fn with_nobody_attached_the_call_fails_instead_of_hanging() {
    let (ctx, tools, _broker) = boot(json!({ "ask-user": {} })).await;
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        ask(&ctx, &tools, Some("a1"), CancelToken::new(), one_question()),
    )
    .await
    .expect("no frontend means an immediate answer");
    assert_eq!(error_of(&result).0, "NO_PROVIDER");
}

#[tokio::test]
async fn an_empty_or_malformed_batch_never_reaches_a_person() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {} })).await;
    let mut inbox = broker.attach();
    for (args, code) in [
        (json!({ "questions": [] }), "EMPTY_QUESTIONS"),
        (json!({}), "BAD_ARGS"),
        (json!({ "questions": [{ "id": "x" }] }), "BAD_ARGS"),
    ] {
        let result = ask(&ctx, &tools, Some("a1"), CancelToken::new(), args).await;
        assert_eq!(error_of(&result).0, code);
    }
    assert!(inbox.questions.try_recv().is_err(), "nothing was asked");
}

#[tokio::test]
async fn a_call_with_no_agent_still_reaches_the_person_like_typescript() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {} })).await;
    let mut inbox = broker.attach();
    let call = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move { ask(&ctx, &tools, None, CancelToken::new(), one_question()).await }
    });
    let pending = inbox.questions.recv().await.unwrap();
    assert_eq!(pending.agent, None);
    pending.answer(Reply::Answered(vec![Answer {
        id: "mode".into(),
        selected: vec![],
        custom: Some("x".into()),
    }]));
    assert!(!call.await.unwrap().is_error());
}

#[tokio::test]
async fn duplicate_question_ids_never_reach_a_person() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {} })).await;
    let mut inbox = broker.attach();
    let args = json!({ "questions": [
        { "id": "a", "question": "one?" }, { "id": "a", "question": "two?" }
    ]});
    let result = ask(&ctx, &tools, Some("a1"), CancelToken::new(), args).await;
    assert_eq!(error_of(&result).0, "BAD_ARGS");
    assert!(inbox.questions.try_recv().is_err());
}

#[tokio::test]
async fn cancelling_the_turn_aborts_the_open_question() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {} })).await;
    let mut inbox = broker.attach();
    let cancel = CancelToken::new();
    let call = tokio::spawn({
        let (ctx, tools, cancel) = (ctx.clone(), tools.clone(), cancel.clone());
        async move { ask(&ctx, &tools, Some("a1"), cancel, one_question()).await }
    });
    let pending = inbox.questions.recv().await.unwrap();
    cancel.cancel();
    assert_eq!(error_of(&call.await.unwrap()).0, "ASK_ABORTED");
    assert!(!pending.is_open(), "the frontend can tell it is stale");
}

#[tokio::test]
async fn a_root_agent_is_not_mistaken_for_a_sub_agent() {
    let (ctx, tools, broker) = boot(json!({ "ask-user": {}, "subagent": {} })).await;
    let mut inbox = broker.attach();
    let lineage = ctx
        .require::<dsh_rs::plugins::subagent::Lineage>(dsh_rs::plugins::subagent::LINEAGE_SERVICE)
        .unwrap();
    assert!(!lineage.is_child("root"), "an unknown agent is a root");
    // A root agent may ask.
    let call = tokio::spawn({
        let (ctx, tools) = (ctx.clone(), tools.clone());
        async move {
            ask(
                &ctx,
                &tools,
                Some("root"),
                CancelToken::new(),
                one_question(),
            )
            .await
        }
    });
    inbox
        .questions
        .recv()
        .await
        .unwrap()
        .answer(Reply::Answered(vec![Answer {
            id: "mode".into(),
            selected: vec![],
            custom: Some("x".into()),
        }]));
    assert!(!call.await.unwrap().is_error());
}
