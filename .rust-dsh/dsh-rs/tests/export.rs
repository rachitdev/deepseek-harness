//! `GET /api/session.export`: the archive's entries, the checks on the request,
//! and the `/export` command that asks the browser to download it.

use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::Arc;

use cordis::Context;
use dsh_rs::api::services::{AgentRegistryService, SessionService};
use dsh_rs::attachments::{AttachmentStore, ATTACHMENTS_SERVICE};
use dsh_rs::bundle::install_profile;
use dsh_rs::core::AgentOptions;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use dsh_rs::images::{ImageConfig, ImageModels};
use dsh_rs::session::store::SESSIONS_SERVICE;
use dsh_rs::types::{
    ContentBlock, CreateSessionOptions, Message, MessageSource, Role, SessionEventData,
};
use http::{HeaderMap, Method};
use serde_json::{json, Value};

const TOKEN: &str = "export-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Gateway,
    store: AttachmentStore,
    _dirs: Vec<PathBuf>,
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dsh-export-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let raster = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x * 7 % 256) as u8, (y * 5 % 256) as u8, 90])
    });
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(raster)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

async fn rig(name: &str, level: u8) -> Rig {
    let logs = scratch(&format!("{name}-logs"));
    let attachments = scratch(&format!("{name}-att"));
    let ctx = Context::new();
    install_profile(
        &ctx,
        &json!({ "config": { "store_dir": logs.to_string_lossy(), "store_format": "v4-plain" } }),
    )
    .await
    .unwrap();
    let store = AttachmentStore::open(&attachments).unwrap();
    ctx.provide(ATTACHMENTS_SERVICE, store.clone()).await.unwrap();
    let router = Router::new(RouterConfig {
        default_model: DefaultModel {
            provider: "mock".to_string(),
            model: "m".to_string(),
        },
        export_compression_level: level,
        ..RouterConfig::default()
    });
    Rig {
        ctx,
        gateway: Gateway::new(
            router,
            Auth {
                token: TOKEN.to_string(),
            },
            None,
        ),
        store,
        _dirs: vec![logs, attachments],
    }
}

fn sessions(rig: &Rig) -> Arc<SessionService> {
    rig.ctx.require::<SessionService>(SESSIONS_SERVICE).unwrap()
}

fn user(id: &str, content: Vec<ContentBlock>) -> Message {
    Message {
        id: id.to_string(),
        role: Role::User,
        content,
        source: MessageSource::User { rpc_id: None },
    }
}

async fn get(rig: &Rig, method: Method, query: &str) -> (u16, HeaderMap, Vec<u8>, Option<String>) {
    let mut headers = HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, extra) = rig
        .gateway
        .route_with_query(
            &rig.ctx,
            &method,
            "/api/session.export",
            Some(query),
            &headers,
            &[],
        )
        .await;
    (
        reply.status,
        extra,
        reply.body,
        Some(reply.content_type.to_string()),
    )
}

fn entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("a ZIP archive");
    (0..archive.len())
        .map(|index| {
            let mut file = archive.by_index(index).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            (file.name().to_string(), data)
        })
        .collect()
}

fn names(bytes: &[u8]) -> Vec<String> {
    entries(bytes).into_iter().map(|(name, _)| name).collect()
}

async fn open_agent(rig: &Rig, text: &str) -> String {
    let agent = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
    agent
        .session()
        .append(SessionEventData::UserMessage {
            message: user("m1", vec![ContentBlock::text(text)]),
        });
    agent.id().to_string()
}

#[tokio::test]
async fn the_archive_holds_the_log_under_the_current_generation_name() {
    let rig = rig("plain", 6).await;
    let id = open_agent(&rig, "hello export").await;
    let (status, headers, body, content_type) =
        get(&rig, Method::GET, &format!("sessionId={id}")).await;
    assert_eq!(status, 200);
    assert_eq!(content_type.as_deref(), Some("application/zip"));
    assert_eq!(
        headers.get("content-disposition").unwrap(),
        &format!("attachment; filename=\"dsh-session-{id}.zip\"")
    );
    let files = entries(&body);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "session.v4.jsonl");
    let text = String::from_utf8(files[0].1.clone()).unwrap();
    let mut lines = text.lines();
    let header: Value = serde_json::from_str(lines.next().unwrap()).unwrap();
    assert_eq!(header["type"], "session");
    assert_eq!(header["id"], id.as_str());
    assert!(text.contains("hello export"), "the log carries the prompt");
    assert!(text.ends_with('\n'));
}

#[tokio::test]
async fn rows_a_live_session_has_not_written_yet_are_in_the_archive() {
    let rig = rig("live", 6).await;
    let id = open_agent(&rig, "first").await;
    let agent = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .get(&id)
        .unwrap();
    agent.session().append(SessionEventData::UserMessage {
        message: user("m2", vec![ContentBlock::text("second, still buffered")]),
    });
    let (_, _, body, _) = get(&rig, Method::GET, &format!("sessionId={id}")).await;
    let text = String::from_utf8(entries(&body).remove(0).1).unwrap();
    assert!(text.contains("second, still buffered"));
}

#[tokio::test]
async fn images_and_files_the_log_references_are_archived_once() {
    let rig = rig("attachments", 6).await;
    let id = open_agent(&rig, "with attachments").await;
    let file = rig.store.save_file(b"file bytes", Some("notes.txt")).unwrap();
    let prepared = {
        let admission = dsh_rs::gui::images::ImageAdmission::new(
            Some(rig.store.clone()),
            &ImageConfig::default(),
            ImageModels::default(),
        );
        let wire = dsh_rs::gui::images::WireImage {
            media_type: "image/png".into(),
            data: dsh_rs::gui::base64::encode(&png(8, 8)),
            name: Some("shot.png".into()),
        };
        admission.admit(&[wire]).unwrap().remove(0)
    };
    let agent = rig
        .ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .get(&id)
        .unwrap();
    for n in 0..2 {
        agent.session().append(SessionEventData::UserMessage {
            message: user(
                &format!("a{n}"),
                vec![
                    ContentBlock::File {
                        attachment: file.clone(),
                    },
                    ContentBlock::Image {
                        attachment: prepared.clone(),
                    },
                ],
            ),
        });
    }
    let (status, _, body, _) = get(&rig, Method::GET, &format!("sessionId={id}")).await;
    assert_eq!(status, 200);
    let digest = file.attachment_id.trim_start_matches("sha256:");
    assert_eq!(
        names(&body),
        vec![
            "session.v4.jsonl".to_string(),
            format!("media/{}.png", prepared.attachment_id),
            format!("files/{}/{digest}/notes.txt", &digest[..2]),
        ]
    );
    let files = entries(&body);
    assert_eq!(files[1].1, png_stored(&rig, &prepared));
    assert_eq!(files[2].1, b"file bytes");
}

fn png_stored(rig: &Rig, image: &dsh_rs::types::ImageRef) -> Vec<u8> {
    rig.store.read_image(image).unwrap()
}

#[tokio::test]
async fn descendants_are_included_only_when_asked_for() {
    let rig = rig("descendants", 6).await;
    let root = open_agent(&rig, "root prompt").await;
    let child = sessions(&rig).create(CreateSessionOptions {
        id: Some("child-1".into()),
        cwd: Some("/tmp".into()),
        seed: Vec::new(),
        parent_session: Some(root.clone()),
        agent_preset: None,
    });
    child.append(SessionEventData::UserMessage {
        message: user("c1", vec![ContentBlock::text("child prompt")]),
    });
    let grandchild = sessions(&rig).create(CreateSessionOptions {
        id: Some("grand/child".into()),
        cwd: Some("/tmp".into()),
        seed: Vec::new(),
        parent_session: Some("child-1".into()),
        agent_preset: None,
    });
    grandchild.append(SessionEventData::UserMessage {
        message: user("g1", vec![ContentBlock::text("grandchild prompt")]),
    });

    let (_, _, plain, _) = get(&rig, Method::GET, &format!("sessionId={root}")).await;
    assert_eq!(names(&plain), vec!["session.v4.jsonl"]);
    let (_, _, explicit, _) = get(
        &rig,
        Method::GET,
        &format!("sessionId={root}&includeDescendants=false"),
    )
    .await;
    assert_eq!(names(&explicit), vec!["session.v4.jsonl"]);

    let (status, _, all, _) = get(
        &rig,
        Method::GET,
        &format!("sessionId={root}&includeDescendants=true"),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        names(&all),
        vec![
            "session.v4.jsonl",
            "subagents/child-1/session.v4.jsonl",
            "subagents/grand_child/session.v4.jsonl",
        ]
    );
    let files = entries(&all);
    assert!(String::from_utf8_lossy(&files[1].1).contains("child prompt"));
    assert!(String::from_utf8_lossy(&files[2].1).contains("grandchild prompt"));
}

#[tokio::test]
async fn a_stored_session_no_live_agent_covers_is_exported() {
    let logs = scratch("cold-logs");
    let profile = json!({ "config": { "store_dir": logs.to_string_lossy(), "store_format": "v4-plain" } });
    let first = Context::new();
    install_profile(&first, &profile).await.unwrap();
    let agent = first
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .create(None, AgentOptions::mock("m"), Some("/tmp".into()), None, None)
        .unwrap();
    agent.session().append(SessionEventData::UserMessage {
        message: user("m1", vec![ContentBlock::text("from an earlier run")]),
    });
    let id = agent.id().to_string();
    first
        .require::<SessionService>(SESSIONS_SERVICE)
        .unwrap()
        .flush(&id)
        .await
        .unwrap();

    let next = Context::new();
    install_profile(&next, &profile).await.unwrap();
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    );
    let mut headers = HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = gateway
        .route_with_query(
            &next,
            &Method::GET,
            "/api/session.export",
            Some(&format!("sessionId={id}")),
            &headers,
            &[],
        )
        .await;
    assert_eq!(reply.status, 200);
    let text = String::from_utf8(entries(&reply.body).remove(0).1).unwrap();
    assert!(text.contains("from an earlier run"));
}

#[tokio::test]
async fn bad_requests_are_refused_with_the_typescript_texts() {
    let rig = rig("refused", 6).await;
    for query in [
        "",
        "sessionId=",
        "includeDescendants=true",
        "sessionId=x&includeDescendants=yes",
        "sessionId=x&includeDescendants=",
    ] {
        let (status, _, body, _) = get(&rig, Method::GET, query).await;
        assert_eq!(status, 400, "{query}");
        assert_eq!(body, b"missing or invalid sessionId query parameter", "{query}");
    }
    let (status, _, body, _) = get(&rig, Method::GET, "sessionId=ghost").await;
    assert_eq!(status, 404);
    assert_eq!(body, b"session not found");
    let (status, _, _, _) = get(&rig, Method::POST, "sessionId=ghost").await;
    assert_eq!(status, 404, "only GET and HEAD reach the route");
}

#[tokio::test]
async fn a_head_request_answers_with_the_headers_and_no_body() {
    let rig = rig("head", 6).await;
    let id = open_agent(&rig, "head").await;
    let (status, headers, body, _) = get(&rig, Method::HEAD, &format!("sessionId={id}")).await;
    assert_eq!(status, 200);
    assert!(body.is_empty());
    assert!(headers.get("content-disposition").is_some());
    let (status, _, _, _) = get(&rig, Method::HEAD, "sessionId=ghost").await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn a_missing_attachment_fails_the_export_without_naming_a_path() {
    let rig = rig("missing", 6).await;
    let id = open_agent(&rig, "x").await;
    let file = rig.store.save_file(b"gone soon", Some("a.txt")).unwrap();
    rig.ctx
        .require::<AgentRegistryService>(dsh_rs::api::AGENTS_SERVICE)
        .unwrap()
        .get(&id)
        .unwrap()
        .session()
        .append(SessionEventData::UserMessage {
            message: user("f", vec![ContentBlock::File { attachment: file.clone() }]),
        });
    let path = rig.store.file_path(&file).unwrap();
    std::fs::remove_file(&path).unwrap();
    let (status, _, body, _) = get(&rig, Method::GET, &format!("sessionId={id}")).await;
    assert_eq!(status, 500);
    let text = String::from_utf8(body).unwrap();
    assert_eq!(text, "session log export failed to read an attachment");
    assert!(!text.contains('/'), "no host path reaches the browser");
}

#[tokio::test]
async fn without_a_session_store_the_route_says_so() {
    let ctx = Context::new();
    let gateway = Gateway::new(
        Router::new(RouterConfig::default()),
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    );
    let mut headers = HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = gateway
        .route_with_query(
            &ctx,
            &Method::GET,
            "/api/session.export",
            Some("sessionId=x"),
            &headers,
            &[],
        )
        .await;
    assert_eq!(reply.status, 500);
    assert!(String::from_utf8_lossy(&reply.body).contains("missing session-persistence"));
}

#[tokio::test]
async fn the_compression_level_changes_the_entry_size() {
    let big = "abcdefgh".repeat(20_000);
    let stored = rig("level0", 0).await;
    let id = open_agent(&stored, &big).await;
    let (_, _, none, _) = get(&stored, Method::GET, &format!("sessionId={id}")).await;
    let packed = rig("level9", 9).await;
    let id = open_agent(&packed, &big).await;
    let (_, _, deflated, _) = get(&packed, Method::GET, &format!("sessionId={id}")).await;
    assert!(none.len() > big.len(), "level 0 stores the text");
    assert!(deflated.len() < big.len() / 4, "level 9 deflates it");
}

async fn rpc(rig: &Rig, method: &str, args: Value) -> Value {
    let body = serde_json::to_vec(&json!({
        "type": "client-request", "rpcId": "rpc-1", "method": method,
        "payload": { "args": args },
    }))
    .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    let (reply, _) = rig
        .gateway
        .route(&rig.ctx, &Method::POST, "/api", &headers, &body)
        .await;
    serde_json::from_slice::<Value>(&reply.body).unwrap()["result"].clone()
}

#[tokio::test]
async fn the_export_command_is_listed_by_name_order_and_asks_for_the_download() {
    let rig = rig("command", 6).await;
    let id = open_agent(&rig, "x").await;
    let listed = rpc(&rig, "commands/list", json!({ "agentId": id })).await;
    let commands = listed["value"].as_array().expect("a list");
    let export = commands
        .iter()
        .find(|command| command["name"] == "export")
        .expect("export is offered when sessions are stored");
    assert_eq!(export["definitionId"], "@deepseek-ai/dsh-session-log-export");
    assert_eq!(export["description"], "Download this Session log as a ZIP archive");
    assert!(export.get("input").is_none(), "it takes no input");
    let names: Vec<_> = commands.iter().map(|c| c["name"].as_str().unwrap()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);

    let run = |line: &'static str| rpc(&rig, "commands/execute", json!({
        "agentId": id, "line": line, "submittedAttachments": [],
    }));
    let ok = run("/export").await;
    assert_eq!(ok["value"]["result"]["kind"], "success", "{ok}");
    assert_eq!(ok["value"]["result"]["text"], "Session log download requested.");
    let spaced = run("/export   ").await;
    assert_eq!(spaced["value"]["result"]["kind"], "success");
    let path = run("/export out.zip").await;
    assert_eq!(path["value"]["result"]["kind"], "error");
    assert_eq!(path["value"]["result"]["text"], "The Web /export command does not accept a path.");
}
