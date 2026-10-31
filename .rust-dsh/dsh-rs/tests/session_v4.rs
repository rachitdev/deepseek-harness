//! TS-compatible session log (format v4): codec, file store, and the real TS
//! validator as the oracle. Real logs under `~/.dsh/sessions` are only READ.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use dsh_rs::core::agent::user_message_with_text;
use dsh_rs::session::v4::{decode_log, encode_event, EncodeState, V4Header, V4Row};
use dsh_rs::session::v4_store::{
    decode_segment, encode_segment, project_key, read_log_bytes, read_log_file, session_dir,
    Compression, V4Persistence,
};
use dsh_rs::session::{SessionPersistence, TurnEndReason};
use dsh_rs::types::{
    GoalBlockReason, GoalChange, GoalOperation, GoalPhase, GoalRef, GoalSnapshot,
    ContentBlock, InboxOutcome, InboxTarget, Message, MessageSource, Role, SessionEvent,
    SessionEventData as D, TodoItem, TodoStatus, TokenUsage,
};
use serde_json::{json, Value};

fn oracle_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/oracle/ts_validate.mjs")
}

fn node() -> Option<PathBuf> {
    ["/opt/homebrew/bin/node", "/usr/local/bin/node"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

/// Validate with the real TS reader; `None` when the TS build is unavailable
/// (the test then says so instead of passing silently).
fn oracle(file: &Path) -> Option<Value> {
    let node = node()?;
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if !repo
        .join("packages/session/session-format-catalog/lib/index.js")
        .exists()
    {
        return None;
    }
    let out = Command::new(node)
        .arg(oracle_script())
        .arg(file)
        .arg(&repo)
        .output()
        .ok()?;
    serde_json::from_slice(out.stdout.split(|b| *b == b'\n').next()?).ok()
}

fn assert_oracle_ok(file: &Path, expect_events: usize) {
    match oracle(file) {
        Some(v) => assert_eq!(v["ok"], json!(true), "TS validator rejected {file:?}: {v}"),
        None => eprintln!("SKIPPED TS oracle (node or built TS packages missing) for {file:?}"),
    }
    let _ = expect_events;
}

fn msg(id: &str, role: Role, content: Vec<ContentBlock>, source: MessageSource) -> Message {
    Message {
        id: id.into(),
        role,
        content,
        source,
    }
}

fn ev(seq: u64, data: D) -> SessionEvent {
    SessionEvent::new(seq, 1_000 + seq, data)
}

fn model() -> MessageSource {
    MessageSource::Model {
        provider: "mock".into(),
        model: "mock-1".into(),
    }
}

/// A full turn with a tool call, a todo list and a Rust-only event of each kind.
fn sample_events() -> Vec<SessionEvent> {
    let call = ContentBlock::ToolCall {
        id: "c1".into(),
        name: "bash".into(),
        arguments: "{\"command\":\"ls\"}".into(),
    };
    let header = dsh_rs::types::EpochHeader {
        config: dsh_rs::types::LlmCallConfig {
            provider: "mock".into(),
            model: "mock-1".into(),
            temperature: Some(0.5),
            max_tokens: Some(1024),
            stop: None,
        },
        system: Some("be brief".into()),
        tools: Some(vec![dsh_rs::types::ToolSchema {
            name: "bash".into(),
            description: "run".into(),
            parameters: json!({"type": "object"}),
        }]),
    };
    vec![
        ev(0, D::TurnStart { turn: 1 }),
        ev(1, D::StepStart { turn: 1, step: 1 }),
        ev(
            2,
            D::UserMessage {
                message: msg(
                    "u1",
                    Role::User,
                    vec![ContentBlock::text("hello")],
                    MessageSource::User { rpc_id: None },
                ),
            },
        ),
        ev(3, D::RequestHeader { header }),
        ev(
            4,
            D::AssistantMessage {
                turn: 1,
                step: 1,
                message: msg(
                    "a1",
                    Role::Assistant,
                    vec![
                        ContentBlock::Reasoning {
                            text: "think".into(),
                        },
                        call,
                    ],
                    model(),
                ),
                usage: Some(TokenUsage {
                    input_tokens: 10,
                    output_tokens: 5,
                    cache_read_tokens: Some(2),
                    ..Default::default()
                }),
                interrupted: None,
            },
        ),
        ev(
            5,
            D::ToolCall {
                turn: 1,
                step: 1,
                call_id: "c1".into(),
                name: "bash".into(),
                arguments: "{\"command\":\"ls\"}".into(),
            },
        ),
        ev(
            6,
            D::ToolResult {
                turn: 1,
                step: 1,
                message: msg(
                    "tr-c1",
                    Role::User,
                    vec![ContentBlock::ToolResult {
                        tool_call_id: "c1".into(),
                        content: vec![ContentBlock::text("a.txt")],
                        is_error: Some(false),
                    }],
                    MessageSource::Tool {
                        tool: "bash".into(),
                    },
                ),
            },
        ),
        ev(
            7,
            D::TodoWrite {
                todos: vec![TodoItem {
                    content: "x".into(),
                    status: TodoStatus::InProgress,
                }],
            },
        ),
        ev(
            8,
            D::Goal {
                objective: "ship".into(),
                status: dsh_rs::types::GoalStatus::Active,
                note: Some("n".into()),
            },
        ),
        ev(9, D::StepEnd { turn: 1, step: 1 }),
        ev(10, D::StepStart { turn: 1, step: 2 }),
        ev(
            11,
            D::AssistantMessage {
                turn: 1,
                step: 2,
                message: msg(
                    "a2",
                    Role::Assistant,
                    vec![ContentBlock::text("done")],
                    model(),
                ),
                usage: None,
                interrupted: None,
            },
        ),
        ev(12, D::StepEnd { turn: 1, step: 2 }),
        ev(
            13,
            D::TurnEnd {
                turn: 1,
                reason: TurnEndReason::Completed,
            },
        ),
        // Log-only user renames; the TS reader folds the latest one.
        ev(14, D::SessionTitle { title: "first title".into() }),
        ev(15, D::SessionTitle { title: "renamed by hand".into() }),
        // The preset a session runs, chosen after creation.
        ev(
            16,
            D::AgentPresetSelected {
                agent_preset: "reviewer".into(),
            },
        ),
        // A goal created, blocked with a reason, then cleared: the TS reader
        // folds each row strictly.
        ev(
            17,
            D::GoalChange {
                change: GoalChange::Snapshot {
                    operation: GoalOperation::Create,
                    goal: GoalSnapshot {
                        id: "goal-1".into(),
                        revision: 1,
                        objective: "ship it".into(),
                        phase: GoalPhase::Active,
                        blocked_reason: None,
                        max_goal_rounds: 3,
                    },
                    rounds_started: 0,
                    created_at: 1_000,
                    updated_at: 1_000,
                },
            },
        ),
        ev(
            18,
            D::GoalChange {
                change: GoalChange::Snapshot {
                    operation: GoalOperation::Block,
                    goal: GoalSnapshot {
                        id: "goal-1".into(),
                        revision: 2,
                        objective: "ship it".into(),
                        phase: GoalPhase::Blocked,
                        blocked_reason: Some(GoalBlockReason {
                            code: "needs-review".into(),
                            message: "waiting for a human".into(),
                        }),
                        max_goal_rounds: 3,
                    },
                    rounds_started: 0,
                    created_at: 1_000,
                    updated_at: 1_001,
                },
            },
        ),
        ev(
            19,
            D::GoalChange {
                change: GoalChange::Clear {
                    cleared: GoalRef { id: "goal-1".into(), revision: 3 },
                    cleared_at: 1_002,
                },
            },
        ),
        // Human feedback is log-only: a remark, then a rating put and deleted.
        ev(
            20,
            D::FeedbackRecord {
                text: Some("hello".into()),
                category: Some("other".into()),
            },
        ),
        ev(21, D::FeedbackRecord { text: None, category: None }),
        ev(
            22,
            D::FeedbackMessagePut {
                session_id: "s".into(),
                item: dsh_rs::types::FeedbackItem {
                    message_id: "a1".into(),
                    rating: "negative".into(),
                    note: Some("too long".into()),
                    category: Some("task-result".into()),
                    version: "339981d1-7840-4a11-b635-f4db2d4bafb0".into(),
                    created_at: 1_000,
                    updated_at: 1_001,
                },
            },
        ),
        ev(
            23,
            D::FeedbackMessageDelete {
                session_id: "s".into(),
                message_id: "a1".into(),
            },
        ),
        // A slash command's lifecycle: log-only, paired by id.
        ev(
            24,
            D::CommandRun {
                command_id: "cmd-t-1".into(),
                name: "goal".into(),
                args: Some(" do it".into()),
            },
        ),
        ev(
            25,
            D::CommandDone {
                command_id: "cmd-t-1".into(),
                success: true,
                text: Some("Goal created".into()),
                source_event_seq: Some(7),
            },
        ),
        ev(26, D::CommandRun { command_id: "cmd-t-2".into(), name: "feedback".into(), args: None }),
        ev(
            27,
            D::CommandDone { command_id: "cmd-t-2".into(), success: false, text: None, source_event_seq: None },
        ),
        // Permission knobs and plan mode: log-only rows the TS reader folds.
        ev(28, D::PermissionPreset { preset: "workspace-write".into() }),
        ev(29, D::SandboxMode { mode: "workspace-write".into() }),
        ev(30, D::ApprovalPolicy { policy: "ask".into(), source: None }),
        ev(31, D::ApprovalPolicy { policy: "never".into(), source: Some("delegation".into()) }),
        ev(32, D::PlanMode { active: true }),
        ev(33, D::PlanMode { active: false }),
    ]
}

/// Flush a resumed backend. A concurrently running test may be between `fork` and
/// `exec` of the TS oracle, holding a copy of the just-dropped first backend's lease
/// descriptor, so the lock frees a moment late; a real second owner never frees it.
fn flush_after_handover(backend: &V4Persistence, id: &String) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match backend.flush(id) {
            Ok(()) => return,
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(err) => panic!("resumed flush: {err}"),
        }
    }
}

fn write_log(
    dir: &Path,
    events: &[SessionEvent],
    compression: Compression,
) -> (V4Persistence, PathBuf) {
    let backend = V4Persistence::new(dir, compression);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1_000,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "s-1".to_string();
    backend.on_session_open(&id, &header, &[]);
    for event in events {
        backend.on_event(&id, event);
    }
    backend.flush(&id).unwrap();
    let file =
        session_dir(dir, Some("/tmp/proj"), "s-1").join(if compression == Compression::Zstd {
            "session.v4.jsonl.zstd"
        } else {
            "session.v4.jsonl"
        });
    (backend, file)
}

#[test]
fn path_encoding_matches_ts_directory_names() {
    assert_eq!(
        project_key("/Users/rachit/code/pilots/codegraph-dsh"),
        "--Users-rachit-code-pilots-codegraph-dsh--"
    );
    assert_eq!(project_key("/"), "--root--");
    assert_eq!(project_key("/a b/c~d"), "--a~0020b-c~007Ed--");
    assert_eq!(encode_segment("session-1/../x"), "session-1~002F..~002Fx");
    assert_eq!(encode_segment(".."), "~002E~002E");
    for raw in ["plain", "a/b", "~", "naïve", "😀"] {
        assert_eq!(decode_segment(&encode_segment(raw)).as_deref(), Some(raw));
    }
    assert_eq!(decode_segment("bad~12"), None);
}

#[test]
fn rust_written_log_passes_the_ts_validator_and_round_trips() {
    for compression in [Compression::Zstd, Compression::None] {
        let tmp = tempfile::tempdir().unwrap();
        let events = sample_events();
        let (backend, file) = write_log(tmp.path(), &events, compression);
        assert_oracle_ok(&file, events.len());

        let loaded = backend.load(&"s-1".to_string()).unwrap();
        assert_eq!(
            loaded, events,
            "round trip must be lossless for {compression:?}"
        );
    }
}

#[test]
fn written_rows_use_ts_vocabulary() {
    let mut state = EncodeState::default();
    let rows: Vec<V4Row> = sample_events()
        .iter()
        .map(|e| encode_event(e, &mut state).unwrap())
        .collect();
    let by = |kind: &str| {
        rows.iter()
            .find(|r| r.kind == kind)
            .unwrap_or_else(|| panic!("no {kind} row"))
    };
    assert_eq!(by("tool/result").source_event_seqs, Some(vec![5]));
    assert_eq!(by("tool/result").data["message"]["role"], "tool");
    assert_eq!(by("todo/write").data["todos"][0]["status"], "in_progress");
    assert!(
        by("request/header").data["header"].get("system").is_none(),
        "retired field must not be written"
    );
    assert_eq!(by("request/header").data["reason"], "initial");
    let goal = by("plugin:dsh-rs/goal");
    assert_eq!(goal.ignorable, Some(true));
    for row in &rows {
        if row.kind.starts_with("plugin:") {
            assert_eq!(row.ignorable, Some(true), "{} must be ignorable", row.kind);
        }
    }
}

#[test]
fn every_flush_appends_an_independent_frame_and_resume_continues_seq() {
    let tmp = tempfile::tempdir().unwrap();
    let events = sample_events();
    let backend = V4Persistence::new(tmp.path(), Compression::Zstd);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1_000,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "s-1".to_string();
    backend.on_session_open(&id, &header, &[]);
    for (n, event) in events.iter().enumerate() {
        backend.on_event(&id, event);
        if n % 4 == 3 {
            backend.flush(&id).unwrap();
        }
    }
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), Some("/tmp/proj"), "s-1").join("session.v4.jsonl.zstd");
    assert_oracle_ok(&file, events.len());
    assert_eq!(read_log_file(&file).unwrap().rows.len(), events.len());

    drop(backend); // releases the lease, as process exit would
                   // A new process resumes: seed carries the stored events, appends continue.
    let resumed = V4Persistence::new(tmp.path(), Compression::Zstd);
    let stored = resumed.load(&id).unwrap();
    resumed.on_session_open(&id, &header, &stored);
    let next = stored.last().unwrap().seq + 1;
    resumed.on_event(&id, &ev(next, D::TurnStart { turn: 2 }));
    resumed.on_event(
        &id,
        &ev(
            next + 1,
            D::TurnEnd {
                turn: 2,
                reason: TurnEndReason::Aborted {
                    cause: "user".into(),
                },
            },
        ),
    );
    flush_after_handover(&resumed, &id);
    assert_oracle_ok(&file, events.len() + 2);
    assert_eq!(read_log_file(&file).unwrap().rows.len(), events.len() + 2);
}

#[test]
fn resuming_at_the_wrong_seq_refuses_to_write() {
    let tmp = tempfile::tempdir().unwrap();
    let events = sample_events();
    let (_backend, file) = write_log(tmp.path(), &events, Compression::Zstd);
    let before = std::fs::read(&file).unwrap();

    let other = V4Persistence::new(tmp.path(), Compression::Zstd);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1_000,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "s-1".to_string();
    // Resume with only a prefix of the stored events: the log has rows Rust cannot account for.
    other.on_session_open(&id, &header, &events[..5]);
    other.on_event(&id, &ev(5, D::TurnStart { turn: 9 }));
    let err = other.flush(&id).expect_err("must refuse");
    assert!(err.to_string().contains("diverged"), "{err}");
    assert_eq!(
        std::fs::read(&file).unwrap(),
        before,
        "file must be untouched"
    );
}

#[test]
fn a_second_writer_on_the_same_session_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let (first, _file) = write_log(tmp.path(), &sample_events(), Compression::Zstd);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1_000,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "s-1".to_string();
    let stored = first.load(&id).unwrap();
    let second = V4Persistence::new(tmp.path(), Compression::Zstd);
    second.on_session_open(&id, &header, &stored);
    second.on_event(&id, &ev(stored.len() as u64, D::TurnStart { turn: 2 }));
    let err = second
        .flush(&id)
        .expect_err("flock must refuse a second owner");
    assert!(
        err.to_string().contains("owned by another process"),
        "{err}"
    );
}

#[test]
fn torn_tail_is_dropped_and_mid_file_corruption_is_loud() {
    let tmp = tempfile::tempdir().unwrap();
    let events = sample_events();
    let (_b, file) = write_log(tmp.path(), &events, Compression::Zstd);
    let bytes = std::fs::read(&file).unwrap();

    // Second frame torn mid-write: first frame still reads, torn bytes are reported.
    let first_len = zstd_frame_len(&bytes);
    let mut two = bytes.clone();
    let extra = zstd::encode_all(
        &b"{\"type\":\"turn/start\",\"seq\":99,\"time\":1,\"data\":{\"turn\":9}}\n"[..],
        3,
    )
    .unwrap();
    two.extend_from_slice(&extra[..extra.len() - 3]);
    let log = read_log_bytes(&two, true).unwrap();
    assert!(log.torn_tail_bytes > 0);
    assert_eq!(log.rows.len(), events.len());
    let _ = first_len;

    // A plain-text torn final line is dropped too.
    let plain_dir = tempfile::tempdir().unwrap();
    let (_b, plain) = write_log(plain_dir.path(), &events, Compression::None);
    let mut text = std::fs::read_to_string(&plain).unwrap();
    text.push_str("{\"type\":\"turn/start\",\"se");
    let log = read_log_bytes(text.as_bytes(), false).unwrap();
    assert!(log.torn_tail_bytes > 0);
    assert_eq!(log.rows.len(), events.len());

    // A corrupt line in the middle is an error, not a silent skip.
    let intact = std::fs::read_to_string(&plain).unwrap();
    let mut lines: Vec<&str> = intact.lines().collect();
    lines[3] = "{not json";
    let err = read_log_bytes(lines.join("\n").as_bytes(), false).unwrap_err();
    assert!(err.to_string().contains("not a v4 row"), "{err}");

    // A damaged frame header is corruption. (A flip inside a frame body is not
    // always caught: frames carry no checksum, so only the header is pinned.)
    let mut flipped = bytes.clone();
    flipped[0] ^= 0xFF;
    assert!(read_log_bytes(&flipped, true).is_err());
}

fn zstd_frame_len(bytes: &[u8]) -> usize {
    zstd::zstd_safe::find_frame_compressed_size(bytes).unwrap()
}

#[test]
fn non_v4_header_and_seq_gaps_are_rejected() {
    let bad = br#"{"type":"session","version":3,"id":"x","createdAt":1}"#;
    assert!(read_log_bytes(bad, false).is_err());
    let rows: Vec<V4Row> = {
        let mut state = EncodeState::default();
        sample_events()
            .iter()
            .map(|e| encode_event(e, &mut state).unwrap())
            .collect()
    };
    let mut gapped = rows.clone();
    gapped.remove(4);
    assert!(decode_log(&gapped)
        .unwrap_err()
        .to_string()
        .contains("seq gap"));
    let header = V4Header::new("x", 1, None, None, false);
    assert_eq!(header.version, 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_real_agent_turn_through_the_bundle_writes_a_valid_v4_log() {
    let tmp = tempfile::tempdir().unwrap();
    let ctx = cordis::Context::new();
    let config = dsh_rs::bundle::BaseConfig {
        store_dir: Some(tmp.path().to_path_buf()),
        store_format: Some("v4".into()),
        ..Default::default()
    };
    dsh_rs::bundle::install_base(&ctx, config).await.unwrap();
    common::boot_scripted(
        &ctx,
        false,
        vec![
            common::tool_call_response("call-1", "bash", json!({ "command": "echo hi-v4" })),
            common::text_response("all done"),
        ],
    )
    .await;
    let agent = common::create_agent(&ctx).await;
    agent.followup(user_message_with_text("u-1", "run it"));
    agent.when_idle().await;
    let sessions = ctx
        .require::<dsh_rs::api::services::SessionService>(dsh_rs::api::SESSIONS_SERVICE)
        .unwrap();
    let session = sessions.get(agent.id()).unwrap();
    sessions.flush(agent.id()).await.unwrap();

    let file = session_dir(tmp.path(), Some("/tmp"), agent.id()).join("session.v4.jsonl.zstd");
    assert!(file.exists(), "log not at the TS path: {file:?}");
    assert_oracle_ok(&file, 0);

    let stored = read_log_file(&file).unwrap();
    let decoded = decode_log(&stored.rows).unwrap();
    assert_eq!(
        decoded.events,
        session.events(),
        "the stored log must replay to the live session"
    );
    assert!(decoded
        .events
        .iter()
        .any(|e| matches!(e.data, D::ToolResult { .. })));
    assert!(
        decoded.skipped.is_empty(),
        "nothing in a Rust-written log should be skipped: {:?}",
        decoded.skipped
    );
}

#[test]
fn unknown_store_format_fails_install() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let err = rt.block_on(async {
        let ctx = cordis::Context::new();
        let config = dsh_rs::bundle::BaseConfig {
            store_dir: Some(std::env::temp_dir()),
            store_format: Some("v9".into()),
            ..Default::default()
        };
        dsh_rs::bundle::install_base(&ctx, config).await
    });
    match err {
        Err(message) => assert!(message.contains("v9"), "{message}"),
        Ok(_) => panic!("an unknown store_format must fail the install"),
    }
}

#[test]
fn the_oracle_rejects_a_log_that_breaks_a_ts_rule() {
    // Negative control: without it a skipped or always-ok oracle would pass the suite.
    let Some(_) = node() else { return };
    let tmp = tempfile::tempdir().unwrap();
    let mut events = sample_events();
    events.remove(5); // drop the tool/call row: its result now has no call
    for (i, e) in events.iter_mut().enumerate() {
        e.seq = i as u64;
    }
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "bad".to_string();
    backend.on_session_open(&id, &header, &[]);
    for e in &events {
        backend.on_event(&id, e);
    }
    // The encoder itself refuses a result with no call row.
    let err = backend.flush(&id).expect_err("encoder must refuse");
    assert!(err.to_string().contains("no tool/call row"), "{err}");

    // Hand-write the same defect past the encoder: the TS validator must reject it.
    let mut state = EncodeState::default();
    let mut lines = vec![serde_json::to_string(&V4Header::new(
        "bad2",
        1,
        Some("/tmp/proj".into()),
        None,
        false,
    ))
    .unwrap()];
    for e in sample_events() {
        let row = encode_event(&e, &mut state).unwrap();
        if row.kind == "tool/call" {
            continue;
        }
        let mut row = row;
        row.seq -= u64::from(row.seq > 5);
        lines.push(serde_json::to_string(&row).unwrap());
    }
    let file = tmp.path().join("bad2.v4.jsonl");
    std::fs::write(&file, lines.join("\n") + "\n").unwrap();
    if let Some(verdict) = oracle(&file) {
        assert_eq!(
            verdict["ok"],
            json!(false),
            "oracle accepted a result without its call: {verdict}"
        );
    }
}

/// Every real TS log on this machine decodes, and every row is accounted for.
/// Reads `~/.dsh/sessions` only; absent directory means the test has nothing to do.
#[test]
fn every_real_ts_log_decodes_with_all_rows_accounted_for() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let root = Path::new(&home).join(".dsh/sessions");
    let Ok(projects) = std::fs::read_dir(&root) else {
        eprintln!("SKIPPED real-log sweep: {root:?} missing");
        return;
    };
    let (mut logs, mut rows_total, mut events_total, mut approx, mut folded) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut skipped_types = std::collections::BTreeMap::<String, usize>::new();
    let mut failures = Vec::new();
    for project in projects.flatten() {
        let Ok(sessions) = std::fs::read_dir(project.path()) else {
            continue;
        };
        for session in sessions.flatten() {
            let file = session.path().join("session.v4.jsonl.zstd");
            if !file.is_file() {
                continue;
            }
            let outcome = read_log_file(&file)
                .map_err(|e| e.to_string())
                .and_then(|log| {
                    let decoded = decode_log(&log.rows).map_err(|e| e.to_string())?;
                    let skipped: usize = decoded.skipped.values().sum();
                    let accounted = decoded.events.len() + skipped + decoded.folded_replaces;
                    if accounted != log.rows.len() {
                        return Err(format!(
                            "{} rows but {accounted} accounted for",
                            log.rows.len()
                        ));
                    }
                    Ok((log.rows.len(), decoded))
                });
            match outcome {
                Ok((rows, decoded)) => {
                    logs += 1;
                    rows_total += rows;
                    events_total += decoded.events.len();
                    approx += decoded.approximate_compactions;
                    folded += decoded.folded_replaces;
                    for (kind, n) in decoded.skipped {
                        *skipped_types.entry(kind).or_default() += n;
                    }
                }
                Err(err) => failures.push(format!("{}: {err}", file.display())),
            }
        }
    }
    eprintln!(
        "real-log sweep: {logs} logs, {rows_total} rows -> {events_total} Rust events, {folded} single-node replaces folded, {approx} compactions approximate, {} skipped row types",
        skipped_types.len()
    );
    assert!(
        failures.is_empty(),
        "{} logs failed:\n{}",
        failures.len(),
        failures
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_forked_session_log_passes_the_ts_validator() {
    let tmp = tempfile::tempdir().unwrap();
    let ctx = cordis::Context::new();
    let store = dsh_rs::session::SessionStore::new(ctx);
    store.attach_persistence(std::sync::Arc::new(V4Persistence::new(
        tmp.path(),
        Compression::Zstd,
    )));
    let parent = store.create(dsh_rs::session::CreateSessionOptions {
        id: Some("parent".into()),
        cwd: Some("/tmp/proj".into()),
        ..Default::default()
    });
    for e in sample_events() {
        parent.append(e.data);
    }
    store.flush(&parent).await.unwrap();
    let child = store.fork(&parent, Some(13), Some("child".into())).unwrap();
    child.append(D::TurnStart { turn: 2 });
    child.append(D::TurnEnd {
        turn: 2,
        reason: TurnEndReason::Completed,
    });
    store.flush(&child).await.unwrap();

    let file = session_dir(tmp.path(), Some("/tmp/proj"), "child").join("session.v4.jsonl.zstd");
    let log = read_log_file(&file).unwrap();
    assert!(log.header.is_seeded);
    assert_eq!(log.header.parent_session.as_deref(), Some("parent"));
    assert_oracle_ok(&file, 0);
}

#[test]
fn resuming_an_unseeded_log_keeps_it_unseeded_for_ts() {
    // Resume seeds the session with stored events, so Rust sees a "seeded" session,
    // but the log on disk is the original unseeded one: a later end-seed row must not
    // become an inherited marker (TS rejects that).
    let tmp = tempfile::tempdir().unwrap();
    let events = sample_events();
    let (first, file) = write_log(tmp.path(), &events, Compression::Zstd);
    drop(first);
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1_000,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let id = "s-1".to_string();
    let resumed = V4Persistence::new(tmp.path(), Compression::Zstd);
    let stored = resumed.load(&id).unwrap();
    resumed.on_session_open(&id, &header, &stored);
    resumed.on_event(&id, &ev(stored.len() as u64, D::SessionEndSeed));
    flush_after_handover(&resumed, &id);
    assert_oracle_ok(&file, 0);
    let log = read_log_file(&file).unwrap();
    assert!(!log.header.is_seeded);
    assert_eq!(log.rows.last().unwrap().data, json!({}));
}

#[test]
fn compaction_and_chunks_survive_as_ignorable_rows() {
    let mut events = sample_events();
    let at = events.len() as u64;
    events.push(ev(at, D::TurnStart { turn: 2 }));
    events.push(ev(at + 1, D::StepStart { turn: 2, step: 1 }));
    events.push(ev(
        at + 2,
        D::AssistantChunk {
            turn: 2,
            step: 1,
            chunk: dsh_rs::types::StreamChunk::TextDelta {
                index: 0,
                text: "hi".into(),
            },
        },
    ));
    events.push(ev(
        at + 3,
        D::Compaction {
            summary: "earlier work".into(),
            before_seq: 7,
        },
    ));
    events.push(ev(at + 4, D::StepEnd { turn: 2, step: 1 }));
    events.push(ev(
        at + 5,
        D::TurnEnd {
            turn: 2,
            reason: TurnEndReason::Aborted {
                cause: "user".into(),
            },
        },
    ));
    let tmp = tempfile::tempdir().unwrap();
    let (backend, file) = write_log(tmp.path(), &events, Compression::Zstd);
    assert_oracle_ok(&file, events.len());
    assert_eq!(backend.load(&"s-1".to_string()).unwrap(), events);
}

#[test]
fn model_selection_is_a_standard_ts_v4_row_and_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "model-selection".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let selected = ev(0, D::ModelSelection {
        provider: "mock".into(), model: "next".into(),
    });
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &selected);
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let stored = read_log_file(&file).unwrap();
    assert_eq!(stored.rows[0].kind, "model/selection");
    assert_eq!(stored.rows[0].data, json!({ "provider": "mock", "model": "next" }));
    assert_eq!(decode_log(&stored.rows).unwrap().events, vec![selected]);
    assert_oracle_ok(&file, 1);
}

#[test]
fn an_inbox_splice_keeps_the_role_of_an_inserted_message() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "inbox-role".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let injected = Message {
        id: "m-3".into(),
        role: Role::Assistant,
        content: vec![ContentBlock::text("context")],
        source: MessageSource::User { rpc_id: None },
    };
    let inserted = ev(
        0,
        D::InboxSpliced {
            target: InboxTarget::NextStep,
            start: 0,
            removed_count: None,
            inserted: vec![injected.clone()],
            outcome: None,
        },
    );
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &inserted);
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let stored = read_log_file(&file).unwrap();
    assert_eq!(stored.rows[0].data["inserted"][0]["role"], json!("assistant"));
    let decoded = decode_log(&stored.rows).unwrap();
    let D::InboxSpliced { inserted, .. } = &decoded.events[0].data else {
        panic!("the row decodes as an inbox splice");
    };
    assert_eq!(
        inserted[0].role,
        Role::Assistant,
        "a decoded message keeps the role it was logged with"
    );
}

#[test]
fn an_inbox_splice_is_a_standard_ts_v4_row_and_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "inbox-splice".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let queued = Message {
        id: "m-1".into(),
        role: Role::User,
        content: vec![ContentBlock::text("queued")],
        source: MessageSource::User { rpc_id: None },
    };
    let identified = Message {
        id: "m-2".into(),
        role: Role::User,
        content: vec![ContentBlock::text("identified")],
        source: MessageSource::User { rpc_id: Some("req-1".into()) },
    };
    let inserted = ev(
        0,
        D::InboxSpliced {
            target: InboxTarget::NextTurn,
            start: 0,
            removed_count: None,
            inserted: vec![queued.clone(), identified.clone()],
            outcome: None,
        },
    );
    let removed = ev(
        1,
        D::InboxSpliced {
            target: InboxTarget::NextTurn,
            start: 0,
            removed_count: Some(1),
            inserted: Vec::new(),
            outcome: Some(InboxOutcome::Canceled),
        },
    );
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &inserted);
    backend.on_event(&id, &removed);
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let stored = read_log_file(&file).unwrap();
    assert_eq!(stored.rows[0].kind, "agent/inbox/spliced");
    assert_eq!(stored.rows[0].data["target"], json!("next-turn"));
    assert_eq!(stored.rows[0].data["start"], json!(0));
    assert_eq!(stored.rows[0].data["inserted"][0]["id"], json!("m-1"));
    assert_eq!(stored.rows[0].data["inserted"][0]["source"], json!({ "kind": "user" }));
    assert_eq!(
        stored.rows[0].data["inserted"][1]["source"],
        json!({ "kind": "user", "rpcId": "req-1" }),
        "the client prompt identity is logged with the queued row"
    );
    assert!(
        stored.rows[0].data.get("removedCount").is_none(),
        "an insert removes nothing"
    );
    assert_eq!(stored.rows[1].data["removedCount"], json!(1));
    assert_eq!(stored.rows[1].data["outcome"], json!("canceled"));
    assert_eq!(
        decode_log(&stored.rows).unwrap().events,
        vec![inserted, removed]
    );
    assert_oracle_ok(&file, 2);
}

#[test]
fn an_uploaded_file_is_a_standard_ts_v4_block_and_round_trips_without_loss() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "uploaded-file".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let reference = dsh_rs::types::FileRef {
        attachment_id: format!("sha256:{}", "ab".repeat(32)),
        name: "notes.txt".into(),
        bytes: 12,
    };
    let message = Message {
        id: "m-1".into(),
        role: Role::User,
        content: vec![
            ContentBlock::text("read this"),
            ContentBlock::File { attachment: reference.clone() },
        ],
        source: MessageSource::User { rpc_id: Some("req-1".into()) },
    };
    let logged = ev(0, D::UserMessage { message });
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &logged);
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let stored = read_log_file(&file).unwrap();
    assert_eq!(
        stored.rows[0].data["content"][1],
        json!({
            "type": "file",
            "attachment": { "attachmentId": reference.attachment_id, "name": "notes.txt", "bytes": 12 },
        })
    );
    let decoded = decode_log(&stored.rows).unwrap();
    assert_eq!(decoded.events, vec![logged]);
    assert_eq!(decoded.lossy.placeholder_blocks, 0, "nothing was reduced to a placeholder");
    assert_oracle_ok(&file, 1);
}

#[test]
fn a_file_block_missing_its_reference_decodes_to_a_counted_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "broken-file".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    let message = user_message_with_text("m-1", "x");
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &ev(0, D::UserMessage { message }));
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let mut stored = read_log_file(&file).unwrap();
    stored.rows[0].data["content"] = json!([{ "type": "file", "attachment": { "name": "n" } }]);
    let decoded = decode_log(&stored.rows).unwrap();
    assert_eq!(decoded.lossy.placeholder_blocks, 1);
}

fn image_ref(downscaled: bool) -> dsh_rs::types::ImageRef {
    dsh_rs::types::ImageRef {
        attachment_id: format!("sha256:{}", "cd".repeat(32)),
        media_type: "image/jpeg".into(),
        bytes: 4096,
        width: 1672,
        height: 2508,
        name: Some("shot.png".into()),
        original_dimensions: downscaled
            .then_some(dsh_rs::types::ImageSize { width: 2000, height: 3000 }),
    }
}

/// An image the user attached is logged as the TypeScript host logs it, and the
/// TypeScript validator accepts the log.
#[test]
fn an_image_block_is_logged_as_the_typescript_host_logs_it() {
    for downscaled in [false, true] {
        let tmp = tempfile::tempdir().unwrap();
        let backend = V4Persistence::new(tmp.path(), Compression::None);
        let id = format!("uploaded-image-{downscaled}");
        let header = dsh_rs::types::SessionHeader {
            cwd: Some("/tmp/proj".into()),
            created_at: 1,
            parent_session: None,
            seed_length: 0,
            agent_preset: None,
        };
        let reference = image_ref(downscaled);
        let message = Message {
            id: "m-1".into(),
            role: Role::User,
            content: vec![ContentBlock::Image { attachment: reference.clone() }],
            source: MessageSource::User { rpc_id: Some("req-1".into()) },
        };
        let logged = ev(0, D::UserMessage { message });
        backend.on_session_open(&id, &header, &[]);
        backend.on_event(&id, &logged);
        backend.flush(&id).unwrap();
        let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
        let stored = read_log_file(&file).unwrap();
        let mut attachment = json!({
            "attachmentId": reference.attachment_id, "mediaType": "image/jpeg",
            "bytes": 4096, "width": 1672, "height": 2508, "name": "shot.png",
        });
        if downscaled {
            attachment["originalDimensions"] = json!({ "width": 2000, "height": 3000 });
        }
        assert_eq!(
            stored.rows[0].data["content"][0],
            json!({ "type": "image", "attachment": attachment })
        );
        let decoded = decode_log(&stored.rows).unwrap();
        assert_eq!(decoded.events, vec![logged]);
        assert_eq!(decoded.lossy.placeholder_blocks, 0);
        assert_oracle_ok(&file, 1);
    }
}

#[test]
fn an_image_block_missing_its_reference_decodes_to_a_counted_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = V4Persistence::new(tmp.path(), Compression::None);
    let id = "broken-image".to_string();
    let header = dsh_rs::types::SessionHeader {
        cwd: Some("/tmp/proj".into()),
        created_at: 1,
        parent_session: None,
        seed_length: 0,
        agent_preset: None,
    };
    backend.on_session_open(&id, &header, &[]);
    backend.on_event(&id, &ev(0, D::UserMessage { message: user_message_with_text("m-1", "x") }));
    backend.flush(&id).unwrap();
    let file = session_dir(tmp.path(), header.cwd.as_deref(), &id).join("session.v4.jsonl");
    let mut stored = read_log_file(&file).unwrap();
    stored.rows[0].data["content"] = json!([{ "type": "image", "attachment": { "width": 3 } }]);
    let decoded = decode_log(&stored.rows).unwrap();
    assert_eq!(decoded.lossy.placeholder_blocks, 1);
}
