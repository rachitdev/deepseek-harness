use dsh_rs::gui::settings::{SettingsStore, MARKET_NAMESPACE, NAMESPACE, SPEECH_NAMESPACE};
use serde_json::{json, Map, Value};

fn section(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn independent_stores_see_writes_and_preserve_other_namespaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/settings.json");
    let first = SettingsStore::new(Some(path.clone())).unwrap();
    let second = SettingsStore::new(Some(path.clone())).unwrap();

    first
        .update(
            SPEECH_NAMESPACE,
            &section(json!({"providerId": "local", "language": "en"})),
            Some(0),
        )
        .unwrap();
    assert_eq!(second.user(SPEECH_NAMESPACE).unwrap()["language"], "en");
    assert_eq!(second.revision(SPEECH_NAMESPACE), Some(1));
    second
        .update(
            MARKET_NAMESPACE,
            &section(json!({"indexes": ["https://example.org/index"]})),
            Some(0),
        )
        .unwrap();
    assert_eq!(
        first.user(MARKET_NAMESPACE).unwrap()["indexes"],
        json!(["https://example.org/index"])
    );
    assert_eq!(first.revision(SPEECH_NAMESPACE), Some(1));

    let document: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(document[SPEECH_NAMESPACE]["user"]["providerId"], "local");
    assert_eq!(document[SPEECH_NAMESPACE]["revision"], 1);
    assert_eq!(document[MARKET_NAMESPACE]["revision"], 1);
    assert!(!path.with_extension("tmp").exists());
}

#[test]
fn revision_checks_use_disk_even_without_a_read_on_the_stale_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let first = SettingsStore::new(Some(path.clone())).unwrap();
    let stale = SettingsStore::new(Some(path.clone())).unwrap();
    first
        .update(
            SPEECH_NAMESPACE,
            &section(json!({"language": "en"})),
            Some(0),
        )
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let patch = section(json!({"language": "fr"}));
    let results = [
        stale.update(SPEECH_NAMESPACE, &patch, Some(0)),
        stale.replace(SPEECH_NAMESPACE, &patch, Some(0)),
        stale.mutate(
            SPEECH_NAMESPACE,
            &[json!({"op": "set", "path": ["language"], "value": "fr"})],
            Some(0),
        ),
    ];
    for result in results {
        let error = result.unwrap_err();
        assert_eq!(error.code, "settings/conflict");
        assert_eq!(
            error.details,
            json!({"ns": SPEECH_NAMESPACE, "expected": 0, "actual": 1})
        );
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn malformed_revisions_and_user_sections_are_refused_without_repair() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let store = SettingsStore::new(Some(path.clone())).unwrap();
    let bad_entries = [
        json!({"user": {}, "revision": "bad"}),
        json!({"user": {}, "revision": -1}),
        json!({"user": {}, "revision": 0.5}),
        json!({"user": {}, "revision": null}),
        json!({"user": {}}),
        json!({"user": "bad", "revision": 0}),
        json!({"user": [], "revision": 0}),
        json!({"user": null, "revision": 0}),
        json!({"revision": 0}),
    ];
    for entry in bad_entries {
        let text = serde_json::to_vec_pretty(&json!({SPEECH_NAMESPACE: entry})).unwrap();
        std::fs::write(&path, &text).unwrap();
        assert!(SettingsStore::new(Some(path.clone())).is_err(), "{entry}");
        assert!(
            store
                .update(
                    MARKET_NAMESPACE,
                    &section(json!({"indexes": ["new"]})),
                    None
                )
                .is_err(),
            "{entry}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), text);
    }
}

#[test]
fn legacy_flat_document_is_read_and_migrated_without_losing_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(
        &path,
        r#"{"ui-settings-general":{"welcomeNoticeVersion":"v1"},"revision":7}"#,
    )
    .unwrap();
    let store = SettingsStore::new(Some(path.clone())).unwrap();
    assert_eq!(store.user(NAMESPACE).unwrap()["welcomeNoticeVersion"], "v1");
    assert_eq!(store.revision(NAMESPACE), Some(7));
    store
        .update(
            MARKET_NAMESPACE,
            &section(json!({"indexes": ["index"]})),
            Some(0),
        )
        .unwrap();
    let reloaded = SettingsStore::new(Some(path)).unwrap();
    assert_eq!(reloaded.user(NAMESPACE), store.user(NAMESPACE));
    assert_eq!(reloaded.revision(NAMESPACE), Some(7));
}

#[test]
fn every_namespace_and_revision_survives_reload_in_declaration_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let store = SettingsStore::new(Some(path.clone())).unwrap();
    store
        .update(
            NAMESPACE,
            &section(json!({"welcomeNoticeVersion": "v2"})),
            Some(0),
        )
        .unwrap();
    store
        .update(
            SPEECH_NAMESPACE,
            &section(json!({"providerId": "reader", "language": "en"})),
            Some(0),
        )
        .unwrap();
    store
        .mutate(
            SPEECH_NAMESPACE,
            &[json!({"op": "set", "path": ["language"], "value": "fr"})],
            Some(1),
        )
        .unwrap();
    store
        .replace(
            MARKET_NAMESPACE,
            &section(json!({"indexes": ["one", "two"]})),
            Some(0),
        )
        .unwrap();
    let expected = store.describe();
    drop(store);
    let reloaded = SettingsStore::new(Some(path)).unwrap();
    assert_eq!(reloaded.describe(), expected);
    let entries = expected["namespaces"].as_array().unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|e| e["ns"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![NAMESPACE, SPEECH_NAMESPACE, MARKET_NAMESPACE]
    );
    assert_eq!(
        entries
            .iter()
            .map(|e| e["revision"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 2, 1]
    );
}

#[test]
fn concurrent_writers_preserve_independent_updates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let writers: Vec<_> = [
        (SPEECH_NAMESPACE, json!({"language": "en"})),
        (MARKET_NAMESPACE, json!({"indexes": ["index"]})),
    ]
    .into_iter()
    .map(|(ns, value)| {
        let store = SettingsStore::new(Some(path.clone())).unwrap();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            store.update(ns, &section(value), Some(0)).unwrap();
        })
    })
    .collect();
    barrier.wait();
    for writer in writers {
        writer.join().unwrap();
    }
    let store = SettingsStore::new(Some(path)).unwrap();
    assert_eq!(store.user(SPEECH_NAMESPACE).unwrap()["language"], "en");
    assert_eq!(
        store.user(MARKET_NAMESPACE).unwrap()["indexes"],
        json!(["index"])
    );
}

#[test]
fn in_memory_operations_keep_rejections_and_noop_revisions() {
    let store = SettingsStore::default();
    for result in [
        store.update("missing", &Map::new(), None),
        store.replace("missing", &Map::new(), None),
        store.mutate("missing", &[], None),
    ] {
        let error = result.unwrap_err();
        assert_eq!(error.code, "settings/rejected");
        assert_eq!(error.message, "No configurable plugin entry \"missing\"");
    }
    assert_eq!(
        store
            .update(
                SPEECH_NAMESPACE,
                &section(json!({"notVolatile": "x"})),
                None
            )
            .unwrap_err()
            .code,
        "settings/rejected"
    );
    assert_eq!(
        store
            .mutate(
                SPEECH_NAMESPACE,
                &[json!({"op": "unset", "path": ["notVolatile"]})],
                None
            )
            .unwrap_err()
            .code,
        "settings/rejected"
    );
    store
        .update(
            SPEECH_NAMESPACE,
            &section(json!({"language": "en"})),
            Some(0),
        )
        .unwrap();
    store
        .update(
            SPEECH_NAMESPACE,
            &section(json!({"language": "en"})),
            Some(1),
        )
        .unwrap();
    assert_eq!(store.revision(SPEECH_NAMESPACE), Some(1));
    store
        .replace(SPEECH_NAMESPACE, &Map::new(), Some(1))
        .unwrap();
    assert_eq!(store.revision(SPEECH_NAMESPACE), Some(2));
}
