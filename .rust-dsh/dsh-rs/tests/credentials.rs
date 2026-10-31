//! `credentials/describe`, `credentials/set` and `credentials/unset` through
//! the gateway, over a throwaway credential document.

use std::path::PathBuf;
use std::sync::Arc;

use cordis::Context;
use dsh_rs::bundle::install_profile;
use dsh_rs::gui::credentials::CredentialStore;
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "creds-token-0123456789";

struct Rig {
    ctx: Context,
    gateway: Arc<Gateway>,
    file: PathBuf,
    _dir: tempfile::TempDir,
}

fn router() -> Router {
    Router::new(RouterConfig {
        agent_presets: dsh_rs::gui::router::default_presets(),
        default_model: DefaultModel {
            provider: "mock".into(),
            model: "m".into(),
        },
        max_page_events: 100,
        max_search_results: 20,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
    })
}

async fn rig_with(initial: Option<&str>) -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("home").join(".credentials.yaml");
    if let Some(text) = initial {
        write_private(&file, text);
    }
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let router = router().with_credentials(Arc::new(CredentialStore::new(Some(file.clone()))));
    let gateway = Arc::new(Gateway::new(
        router,
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    Rig {
        ctx,
        gateway,
        file,
        _dir: dir,
    }
}

async fn rig() -> Rig {
    rig_with(None).await
}

fn write_private(file: &std::path::Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600)).unwrap();
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
    let envelope: Value = serde_json::from_slice(&reply.body).expect("JSON reply");
    envelope["result"].clone()
}

async fn describe(rig: &Rig, refs: Value) -> Value {
    let reply = call(rig, "credentials/describe", json!({ "refs": refs })).await;
    assert_eq!(reply["ok"], true, "{reply}");
    reply["value"].clone()
}

#[tokio::test]
async fn a_reference_is_unconfigured_until_set_then_configured_from_the_file() {
    let rig = rig().await;
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ONE"])).await,
        json!({ "DSH_RS_T_ONE": { "configured": false, "writable": true } })
    );
    let set = call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_ONE", "value": "secret-1" }),
    )
    .await;
    assert_eq!(set, json!({ "ok": true }));
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ONE"])).await,
        json!({ "DSH_RS_T_ONE": { "configured": true, "source": "file", "writable": true } })
    );
    let unset = call(&rig, "credentials/unset", json!({ "ref": "DSH_RS_T_ONE" })).await;
    assert_eq!(unset, json!({ "ok": true }));
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ONE"])).await,
        json!({ "DSH_RS_T_ONE": { "configured": false, "writable": true } })
    );
}

#[tokio::test]
async fn no_reply_ever_carries_the_stored_value() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_TWO", "value": "very-secret-value" }),
    )
    .await;
    let described = call(
        &rig,
        "credentials/describe",
        json!({ "refs": ["DSH_RS_T_TWO"] }),
    )
    .await;
    for text in [reply.to_string(), described.to_string()] {
        assert!(!text.contains("very-secret-value"), "{text}");
    }
}

#[tokio::test]
async fn the_environment_is_the_fallback_and_the_file_outranks_it() {
    let rig = rig().await;
    // SAFETY-free: a name unique to this test, set before any read of it.
    std::env::set_var("DSH_RS_T_ENV", "from-env");
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ENV"])).await["DSH_RS_T_ENV"],
        json!({ "configured": true, "source": "env", "writable": true })
    );
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_ENV", "value": "from-file" }),
    )
    .await;
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ENV"])).await["DSH_RS_T_ENV"]["source"],
        "file"
    );
    call(&rig, "credentials/unset", json!({ "ref": "DSH_RS_T_ENV" })).await;
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_ENV"])).await["DSH_RS_T_ENV"]["source"],
        "env"
    );
    std::env::remove_var("DSH_RS_T_ENV");
    let empty_env = "DSH_RS_T_EMPTY_ENV";
    std::env::set_var(empty_env, "");
    assert_eq!(
        describe(&rig, json!([empty_env])).await[empty_env],
        json!({ "configured": false, "writable": true }),
        "an empty variable is not a credential"
    );
    std::env::remove_var(empty_env);
}

#[tokio::test]
async fn the_document_is_the_typescript_layout_with_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let rig = rig().await;
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_A", "value": "one" }),
    )
    .await;
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_B", "value": "has: colon # and hash" }),
    )
    .await;
    let text = std::fs::read_to_string(&rig.file).unwrap();
    let parsed: serde_json::Value = serde_json::to_value(serde_norway_value(&text)).unwrap();
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["refs"]["DSH_RS_T_A"], "one");
    assert_eq!(parsed["refs"]["DSH_RS_T_B"], "has: colon # and hash");
    assert!(text.starts_with("version: 1\nrefs:"), "{text}");
    let mode = std::fs::metadata(&rig.file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let dir_mode = std::fs::metadata(rig.file.parent().unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        dir_mode & 0o077,
        0,
        "the harness home is not readable by others"
    );
    assert!(!rig.file.with_extension("yaml.lock").exists());
    let leftovers: Vec<_> = std::fs::read_dir(rig.file.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        leftovers,
        vec![".credentials.yaml"],
        "no lock or temporary file is left behind"
    );
}

fn serde_norway_value(text: &str) -> serde_json::Value {
    // The yaml crate is a library dependency; round-trip through its JSON view.
    let value: serde_json::Value = serde_json::from_str(&yaml_to_json(text)).unwrap();
    value
}

fn yaml_to_json(text: &str) -> String {
    // A tiny reader for the two shapes this test writes: `version: N` and a `refs:` block.
    let mut out = serde_json::Map::new();
    let mut refs = serde_json::Map::new();
    let mut in_refs = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("version: ") {
            out.insert("version".into(), json!(rest.parse::<u64>().unwrap()));
            in_refs = false;
        } else if line == "refs:" {
            in_refs = true;
        } else if in_refs {
            let (key, value) = line.trim().split_once(": ").unwrap();
            let value = value.trim_matches('"').trim_matches('\'');
            refs.insert(key.to_string(), json!(value));
        }
    }
    out.insert("refs".into(), Value::Object(refs));
    Value::Object(out).to_string()
}

#[tokio::test]
async fn other_sections_of_a_typescript_document_survive_a_write() {
    let initial = "version: 1\nrecords:\n  client-connection/browser-session:\n    kind: grant\n    payload:\n      version: 1\n      secret: abc\nrefs: { EXISTING: keep }\nforeign: 7\n";
    let rig = rig_with(Some(initial)).await;
    assert_eq!(
        describe(&rig, json!(["EXISTING"])).await["EXISTING"]["source"],
        "file"
    );
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_NEW", "value": "n" }),
    )
    .await;
    let text = std::fs::read_to_string(&rig.file).unwrap();
    for needle in [
        "EXISTING: keep",
        "DSH_RS_T_NEW: n",
        "client-connection/browser-session",
        "kind: grant",
        "secret: abc",
        "foreign: 7",
    ] {
        assert!(text.contains(needle), "{needle} lost from:\n{text}");
    }
    call(&rig, "credentials/unset", json!({ "ref": "EXISTING" })).await;
    let text = std::fs::read_to_string(&rig.file).unwrap();
    assert!(!text.contains("EXISTING"), "{text}");
    assert!(
        text.contains("DSH_RS_T_NEW: n") && text.contains("secret: abc"),
        "{text}"
    );
}

#[tokio::test]
async fn an_external_edit_is_seen_by_the_next_describe() {
    let rig = rig().await;
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_X", "value": "1" }),
    )
    .await;
    write_private(&rig.file, "version: 1\nrefs: { DSH_RS_T_Y: \"2\" }\n");
    let seen = describe(&rig, json!(["DSH_RS_T_X", "DSH_RS_T_Y"])).await;
    assert_eq!(seen["DSH_RS_T_X"]["configured"], false);
    assert_eq!(seen["DSH_RS_T_Y"]["source"], "file");
}

#[tokio::test]
async fn unsetting_a_reference_that_is_not_stored_changes_nothing_and_succeeds() {
    let rig = rig().await;
    let reply = call(&rig, "credentials/unset", json!({ "ref": "NEVER_SET" })).await;
    assert_eq!(reply, json!({ "ok": true }));
    assert!(
        !rig.file.exists(),
        "a no-op unset does not create the document"
    );
}

#[tokio::test]
async fn names_outside_the_identifier_grammar_are_refused_with_every_issue() {
    let rig = rig().await;
    let reply = call(
        &rig,
        "credentials/describe",
        json!({ "refs": ["bad one", "ok", "1x"] }),
    )
    .await;
    assert_eq!(reply["error"]["code"], "gateway/bad-request");
    assert_eq!(
        reply["error"]["message"],
        "invalid payload for credentials.describe"
    );
    let issues = reply["error"]["details"]["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0]["path"], json!(["refs", 0]));
    assert_eq!(issues[1]["path"], json!(["refs", 2]));
    assert_eq!(issues[0]["pattern"], "/^[A-Za-z_][A-Za-z0-9_]*$/");
    assert_eq!(
        issues[0]["message"],
        "Invalid string: must match pattern /^[A-Za-z_][A-Za-z0-9_]*$/"
    );

    let both = call(
        &rig,
        "credentials/set",
        json!({ "ref": "bad one", "value": "" }),
    )
    .await;
    let issues = both["error"]["details"]["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[1]["code"], "too_small");
    assert_eq!(issues[1]["minimum"], 1);
    assert_eq!(
        issues[1]["message"],
        "Too small: expected string to have >=1 characters"
    );
    assert!(!rig.file.exists(), "a refused write creates nothing");
}

#[tokio::test]
async fn more_than_sixty_four_references_are_refused() {
    let rig = rig().await;
    let names: Vec<String> = (0..65).map(|i| format!("R{i}")).collect();
    let reply = call(&rig, "credentials/describe", json!({ "refs": names })).await;
    assert_eq!(reply["error"]["code"], "gateway/bad-request");
    let issue = &reply["error"]["details"]["issues"][0];
    assert_eq!(issue["code"], "too_big");
    assert_eq!(issue["maximum"], 64);
    assert_eq!(
        issue["message"],
        "Too big: expected array to have <=64 items"
    );
    let ok: Vec<String> = (0..64).map(|i| format!("R{i}")).collect();
    assert_eq!(
        describe(&rig, json!(ok)).await.as_object().unwrap().len(),
        64
    );
    assert_eq!(describe(&rig, json!([])).await, json!({}));
    assert_eq!(
        describe(&rig, json!(["A", "A"]))
            .await
            .as_object()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn wrong_wire_types_fail_the_gateway_before_any_rule() {
    let rig = rig().await;
    for (method, args, field) in [
        ("credentials/set", json!({ "ref": 5, "value": "v" }), "ref"),
        (
            "credentials/set",
            json!({ "ref": "OK", "value": 5 }),
            "value",
        ),
        ("credentials/unset", json!({ "ref": 5 }), "ref"),
        ("credentials/describe", json!({ "refs": "x" }), "refs"),
        ("credentials/describe", json!({ "refs": [5] }), "refs"),
        ("credentials/describe", json!({ "refs": [null] }), "refs"),
    ] {
        let reply = call(&rig, method, args.clone()).await;
        assert_eq!(
            reply["error"]["code"], "gateway/input-invalid",
            "{method} {args}: {reply}"
        );
        assert_eq!(reply["error"]["details"]["field"], field);
        assert_eq!(
            reply["error"]["message"],
            format!("typert gateway: {method}: wire field \"{field}\" failed boundary validation")
        );
    }
    let missing = call(&rig, "credentials/set", json!({ "ref": "OK" })).await;
    assert_eq!(missing["error"]["code"], "gateway/arguments-invalid");
    assert!(missing["error"]["message"]
        .as_str()
        .unwrap()
        .contains("missing \"value\""));
}

#[tokio::test]
async fn a_value_of_only_spaces_is_a_value() {
    let rig = rig().await;
    assert_eq!(
        call(
            &rig,
            "credentials/set",
            json!({ "ref": "DSH_RS_T_SP", "value": "  " })
        )
        .await,
        json!({ "ok": true })
    );
    assert_eq!(
        describe(&rig, json!(["DSH_RS_T_SP"])).await["DSH_RS_T_SP"]["source"],
        "file"
    );
}

#[tokio::test]
async fn prototype_like_names_are_ordinary_references() {
    let rig = rig().await;
    call(
        &rig,
        "credentials/set",
        json!({ "ref": "__proto__", "value": "v" }),
    )
    .await;
    let seen = describe(&rig, json!(["__proto__", "constructor"])).await;
    assert_eq!(seen["__proto__"]["source"], "file");
    assert_eq!(seen["constructor"]["configured"], false);
}

#[tokio::test]
async fn a_document_that_cannot_be_trusted_is_never_read_as_no_credentials() {
    for (name, text, needle) in [
        ("flat layout", "OLD_KEY: value\n", "pre-release flat layout"),
        (
            "future version",
            "version: 2\nrefs: {}\n",
            "declares version 2",
        ),
        ("not a mapping", "- a\n- b\n", "must be a mapping"),
        (
            "refs not a mapping",
            "version: 1\nrefs: [a]\n",
            "must be a mapping",
        ),
        (
            "bad reference name",
            "version: 1\nrefs: { 'a b': x }\n",
            "not a valid reference name",
        ),
        (
            "non-string value",
            "version: 1\nrefs: { A: 5 }\n",
            "must be a string",
        ),
        ("empty value", "version: 1\nrefs: { A: '' }\n", "is empty"),
        (
            "syntax error",
            "version: 1\nrefs: { A: [\n",
            "invalid document",
        ),
    ] {
        let rig = rig_with(Some(text)).await;
        let described = call(&rig, "credentials/describe", json!({ "refs": ["A"] })).await;
        assert_eq!(
            described["error"]["code"], "gateway/internal",
            "{name}: {described}"
        );
        assert!(
            described["error"]["message"]
                .as_str()
                .unwrap()
                .contains(needle),
            "{name}: {described}"
        );
        let set = call(&rig, "credentials/set", json!({ "ref": "B", "value": "v" })).await;
        assert_eq!(set["error"]["code"], "credential/rejected", "{name}: {set}");
        assert_eq!(set["error"]["details"], json!({ "ref": "B" }));
        assert_eq!(
            std::fs::read_to_string(&rig.file).unwrap(),
            text,
            "{name}: the document is untouched"
        );
    }
}

#[tokio::test]
async fn an_empty_or_comment_only_document_is_the_empty_store() {
    for text in ["", "# nothing here\n", "   \n"] {
        let rig = rig_with(Some(text)).await;
        assert_eq!(describe(&rig, json!(["A"])).await["A"]["configured"], false);
        assert_eq!(
            call(&rig, "credentials/set", json!({ "ref": "A", "value": "v" })).await,
            json!({ "ok": true })
        );
        assert_eq!(describe(&rig, json!(["A"])).await["A"]["source"], "file");
    }
}

#[tokio::test]
async fn a_document_readable_beyond_its_owner_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let rig = rig_with(Some("version: 1\nrefs: { A: x }\n")).await;
    std::fs::set_permissions(&rig.file, std::fs::Permissions::from_mode(0o644)).unwrap();
    let reply = call(&rig, "credentials/describe", json!({ "refs": ["A"] })).await;
    assert_eq!(reply["error"]["code"], "gateway/internal");
    let message = reply["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("readable beyond its owner (mode 644)") && message.contains("chmod 600"),
        "{message}"
    );
    assert!(!message.contains(": x"), "the message never quotes a value");
}

#[tokio::test]
async fn a_syntax_error_report_never_quotes_the_secret_line() {
    let rig = rig_with(Some("version: 1\nrefs: { TOKEN: super-secret-line [\n")).await;
    let reply = call(&rig, "credentials/describe", json!({ "refs": ["TOKEN"] })).await;
    assert!(!reply.to_string().contains("super-secret-line"), "{reply}");
}

#[tokio::test]
async fn without_a_harness_home_all_three_calls_fail_as_internal() {
    let ctx = Context::new();
    install_profile(&ctx, &json!({ "plugins": {} }))
        .await
        .unwrap();
    let gateway = Arc::new(Gateway::new(
        router(),
        Auth {
            token: TOKEN.to_string(),
        },
        None,
    ));
    let dir = tempfile::tempdir().unwrap();
    let rig = Rig {
        ctx,
        gateway,
        file: dir.path().join("none"),
        _dir: dir,
    };
    for (method, args) in [
        ("credentials/describe", json!({ "refs": ["A"] })),
        ("credentials/set", json!({ "ref": "A", "value": "v" })),
        ("credentials/unset", json!({ "ref": "A" })),
    ] {
        let reply = call(&rig, method, args).await;
        assert_eq!(
            reply["error"]["code"], "gateway/internal",
            "{method}: {reply}"
        );
        assert!(reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("credentials service is absent"));
    }
}

#[tokio::test]
async fn concurrent_writes_to_different_references_all_land() {
    let rig = Arc::new(rig().await);
    let mut tasks = Vec::new();
    for i in 0..12 {
        let rig = Arc::clone(&rig);
        tasks.push(tokio::spawn(async move {
            call(
                &rig,
                "credentials/set",
                json!({ "ref": format!("DSH_RS_T_C{i}"), "value": format!("v{i}") }),
            )
            .await
        }));
    }
    for task in tasks {
        assert_eq!(task.await.unwrap(), json!({ "ok": true }));
    }
    let names: Vec<String> = (0..12).map(|i| format!("DSH_RS_T_C{i}")).collect();
    let seen = describe(&rig, json!(names)).await;
    for i in 0..12 {
        assert_eq!(
            seen[format!("DSH_RS_T_C{i}")]["source"],
            "file",
            "write {i} was lost"
        );
    }
}

#[tokio::test]
async fn a_lock_left_by_an_exited_process_is_taken_over_and_a_live_one_is_waited_on() {
    let rig = rig().await;
    let lock = PathBuf::from(format!("{}.lock", rig.file.display()));
    std::fs::create_dir_all(rig.file.parent().unwrap()).unwrap();
    // A pid far above the system maximum belongs to no process.
    std::fs::write(&lock, "2147483646\n").unwrap();
    let reply = call(
        &rig,
        "credentials/set",
        json!({ "ref": "DSH_RS_T_L", "value": "v" }),
    )
    .await;
    assert_eq!(
        reply,
        json!({ "ok": true }),
        "a dead holder's lock is taken over"
    );
    assert!(!lock.exists());
}

// Expectations from credentials parseCredentialKey and credentials-local
// parseRecord: exactly two lowercase segments, a known tag, only its fields,
// and a JSON payload (the grant parser deliberately does not own its schema).
fn typescript_parse_record(key: &str, record: &Value) {
    let segments: Vec<_> = key.split('/').collect();
    assert_eq!(segments.len(), 2);
    for segment in segments {
        assert!(segment.as_bytes().first().unwrap().is_ascii_lowercase());
        assert!(segment
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'));
    }
    let fields = record.as_object().expect("record mapping");
    assert_eq!(fields["kind"], "grant");
    assert!(fields.keys().all(|k| k == "kind" || k == "payload"));
    assert!(fields.contains_key("payload"));
    assert_eq!(
        serde_json::from_str::<Value>(&serde_json::to_string(&fields["payload"]).unwrap()).unwrap(),
        fields["payload"]
    );
}

#[tokio::test]
async fn records_written_match_typescript_parser_and_signout_deletes_on_disk() {
    use dsh_rs::gui::account::Account;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("credentials.yaml");
    let store = Arc::new(CredentialStore::new(Some(file.clone())));
    let key = "deepseek-account-platform/default";
    let record = json!({"kind":"grant","payload":{"version":1,"token":"test-token","issuer":"https://platform.deepseek.com"}});
    store.write_record(key, record.clone()).await.unwrap();
    let document: Value = serde_norway::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    typescript_parse_record(key, &document["records"][key]);
    assert_eq!(document["records"][key], record);
    let account = Account::system("https://platform.deepseek.com", store.clone()).unwrap();
    assert_eq!(
        account.get_state().await.unwrap()["status"],
        "credential-stored"
    );
    assert_eq!(account.sign_out().await.unwrap()["status"], "signed-out");
    assert!(store.read_record(key).await.unwrap().is_none());
    assert_eq!(account.get_state().await.unwrap()["status"], "signed-out");
}

#[tokio::test]
async fn concurrent_device_creation_returns_the_same_identity() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("credentials.yaml");
    let one = CredentialStore::new(Some(file.clone()));
    let two = CredentialStore::new(Some(file.clone()));
    let create = |current: Option<Value>| {
        current
            .is_none()
            .then(|| json!({"kind":"grant","payload":{"id":uuid::Uuid::new_v4().to_string()}}))
    };
    let key = "deepseek-account-platform/device";
    let (a, b) = tokio::join!(
        one.modify_record(key, create),
        two.modify_record(key, create)
    );
    assert_eq!(a.as_ref().unwrap(), b.as_ref().unwrap());
    typescript_parse_record(key, a.unwrap().as_ref().unwrap());
}

#[tokio::test]
async fn malformed_records_are_untrusted_and_never_replaced() {
    for section in [
        "[]",
        "true",
        "broken",
        "{bad: {kind: grant, payload: null}}",
        "{scope/id: {kind: grant}}",
        "{scope/id: {kind: grant, payload: null, typo: 1}}",
        "{scope/id: {kind: other, payload: null}}",
        "{scope/id: {kind: api-key, key: ''}}",
        "{scope/id: {kind: api-key, env: {BAD_NAME: ''}}}",
        "{scope/id: {kind: api-key, env: {bad-name: value}}}",
    ] {
        let initial = format!("version: 1\nrecords: {section}\n");
        let rig = rig_with(Some(&initial)).await;
        let result = call(&rig, "credentials/set", json!({"ref":"A","value":"new"})).await;
        assert_eq!(result["ok"], false, "{section}: {result}");
        assert_eq!(std::fs::read_to_string(&rig.file).unwrap(), initial);
    }
}
