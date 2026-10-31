//! The browser boot host: the module graph, the injected page, and the
//! `/plugins` resources, against a synthetic checkout and the real one.

use std::fs;
use std::path::{Path, PathBuf};

use cordis::Context;
use dsh_rs::gui::boot::{BootConfig, BootHost};
use dsh_rs::gui::{Auth, DefaultModel, Gateway, Router, RouterConfig};
use serde_json::{json, Value};

const TOKEN: &str = "launch-token-abc123";
const MODULES: &str = "@deepseek-ai/dsh-client-modules";

/// Write one client package under `<root>/packages/<group>/<dir>`.
fn package(root: &Path, dir: &str, name: &str, client: Value, bundle: Option<&str>) -> PathBuf {
    let pkg = root.join("packages").join("g").join(dir);
    fs::create_dir_all(pkg.join("lib")).unwrap();
    let manifest = json!({
        "name": name, "version": "1.2.3",
        "exports": { "./client": { "default": "./lib/client.js" } },
        "dsh": { "client": client },
    });
    fs::write(pkg.join("package.json"), manifest.to_string()).unwrap();
    if let Some(source) = bundle {
        fs::write(pkg.join("lib/client.js"), source).unwrap();
    }
    pkg
}

fn web(inject: &[&str], immediately: bool, external: &[&str]) -> Value {
    json!({ "platform": "web", "inject": inject, "immediately": immediately, "external": external })
}

/// modules <- api (immediate), ui (injects api, requests api's bundle).
fn checkout() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    package(root.path(), "m", MODULES, web(&[], true, &[]), Some("MODULES_BODY()\n//# sourceMappingURL=client.js.map\n"));
    package(root.path(), "a", "pkg-api", web(&[], true, &[]), Some("API_BODY()\n"));
    let ui = package(
        root.path(),
        "u",
        "pkg-ui",
        web(&["pkg-api"], false, &["pkg-api/client"]),
        Some("UI_BODY()"),
    );
    fs::write(ui.join("lib/client.extra.js"), "EXTRA_CHUNK()\n").unwrap();
    fs::write(ui.join("lib/client.node.js"), "NODE_ONLY()\n").unwrap();
    root
}

fn config(root: &Path) -> BootConfig {
    BootConfig {
        client_root: root.to_path_buf(),
        packages: None,
        exclude: vec![],
        include_experimental: false,
    }
}

fn host(root: &Path) -> BootHost {
    BootHost::load(&config(root)).expect("the synthetic checkout composes")
}

fn load_error(root: &Path) -> String {
    BootHost::load(&config(root)).expect_err("this checkout must not compose")
}

fn text(served: Option<dsh_rs::gui::boot::Served>) -> String {
    String::from_utf8(served.expect("resource is served").body).unwrap()
}

#[test]
fn the_module_system_boots_alone_and_every_other_package_follows_in_dependency_order() {
    let root = checkout();
    let graph = host(root.path()).graph().clone();
    let batches = graph["batches"].as_array().unwrap();
    let phases: Vec<&str> = batches.iter().map(|b| b["phase"].as_str().unwrap()).collect();
    assert_eq!(phases, ["bootstrap", "application"]);
    assert_eq!(batches[0]["entries"], json!([MODULES]));
    // `pkg-ui` requests `pkg-api`'s bundle, so the api row must come first.
    assert_eq!(batches[1]["entries"], json!(["pkg-api", "pkg-ui"]));
    let ui = graph["entries"].as_array().unwrap().iter().find(|e| e["id"] == "pkg-ui").unwrap();
    assert_eq!(ui["inject"], json!(["pkg-api"]));
    assert_eq!(ui["external"], json!(["pkg-api/client"]));
    assert!(ui.get("immediately").is_none(), "only immediate rows carry the flag");
    assert_eq!(graph["release"]["version"], "1.2.3");
}

#[test]
fn the_index_gets_the_loader_queue_before_the_graph_and_the_ready_signal_last() {
    let root = checkout();
    let html = host(root.path()).render_index(
        "<!doctype html><html><head><title>x</title></head><body><div id=\"root\"></div></body></html>",
    );
    let queue = html.find("window.__ModuleLoader__").expect("queue script");
    let preload = html.find("rel=\"preload\"").expect("application preload");
    let bootstrap = html.find("<script src=\"plugins/??").expect("blocking bootstrap script");
    let graph = html.find("globalThis[\"__DSH_BOOT__\"]").expect("graph global");
    let ready = html.find("__DSH_BOOT_READY__").expect("ready signal");
    assert!(queue < preload && preload < bootstrap && bootstrap < graph, "{html}");
    assert!(html.find("<body>").unwrap() < ready, "ready lands inside body");
    assert!(html.find("<head>").unwrap() < queue, "rows land inside head");
    assert!(html.contains(&format!("plugins/??{MODULES}/client.js&amp;rev=").replace("&amp;", "&")) || html.contains("client.js&amp;rev="));
}

#[test]
fn a_global_value_cannot_close_its_script_element() {
    let root = checkout();
    package(root.path(), "x", "pkg-evil", web(&[], false, &[]), Some("EVIL()"));
    let host = BootHost::load(&BootConfig { packages: None, ..config(root.path()) }).unwrap();
    let html = host.render_index("<html><head></head><body></body></html>");
    // Every `<` inside a JSON-valued script is escaped, so no payload can inject `</script>`.
    let scripts: Vec<&str> = html.split("<script>").skip(1).collect();
    for script in scripts {
        let body = script.split("</script>").next().unwrap();
        if body.starts_with("globalThis[") {
            assert!(!body.contains('<'), "{body}");
        }
    }
}

#[test]
fn a_combo_script_concatenates_bundles_strips_foreign_map_trailers_and_names_its_own_map() {
    let root = checkout();
    let host = host(root.path());
    let batch = &host.graph()["batches"][1];
    let url = batch["url"].as_str().unwrap();
    let (path, query) = url.split_once('?').unwrap();
    let body = text(host.serve(&format!("/{path}"), Some(query)));
    assert!(body.starts_with("API_BODY()\n;\nUI_BODY()\n;\n"), "{body}");
    assert!(body.trim_end().ends_with(&format!("rev={}", batch["rev"].as_str().unwrap())), "{body}");
    assert_eq!(body.matches("sourceMappingURL").count(), 1, "bundle trailers are replaced, not stacked");
    // The bootstrap bundle carried its own trailer, which must not survive.
    let boot = &host.graph()["batches"][0];
    let (p, q) = boot["url"].as_str().unwrap().split_once('?').unwrap();
    let boot_body = text(host.serve(&format!("/{p}"), Some(q)));
    assert!(!boot_body.contains("sourceMappingURL=client.js.map"), "{boot_body}");
}

#[test]
fn a_combo_source_map_offsets_each_section_by_the_lines_before_it() {
    let root = checkout();
    let host = host(root.path());
    let batch = &host.graph()["batches"][1];
    let url = batch["url"].as_str().unwrap();
    let ids: Vec<String> = batch["entries"].as_array().unwrap().iter().map(|e| e.as_str().unwrap().to_string()).collect();
    let map_query = format!("?{}&rev={}", ids.iter().map(|i| format!("{i}/client.js.map")).collect::<Vec<_>>().join(","), batch["rev"].as_str().unwrap());
    let _ = url;
    let map: Value = serde_json::from_str(&text(host.serve("/plugins/", Some(&map_query)))).unwrap();
    assert_eq!(map["version"], 3);
    let sections = map["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0]["offset"]["line"], 0);
    // "API_BODY()\n" plus the ";\n" separator is two lines.
    assert_eq!(sections[1]["offset"]["line"], 2);
}

#[test]
fn only_advertised_resources_are_served() {
    let root = checkout();
    let host = host(root.path());
    let batch = &host.graph()["batches"][1];
    let rev = batch["rev"].as_str().unwrap();
    let good = format!("?pkg-api/client.js,pkg-ui/client.js&rev={rev}");
    assert!(host.serve("/plugins/", Some(&good)).is_some());
    for bad in [
        format!("?pkg-api/client.js,pkg-ui/client.js&rev={}", "0".repeat(12)),
        format!("?pkg-ui/client.js,pkg-api/client.js&rev={rev}"),
        format!("?pkg-ui/client.js&rev={rev}"),
        "?../../etc/passwd/client.js&rev=x".to_string(),
    ] {
        assert!(host.serve("/plugins/", Some(&bad)).is_none(), "{bad} must not be served");
    }
    assert!(host.serve("/plugins/", None).is_none());
    assert!(host.serve("/other", None).is_none());
}

#[test]
fn chunks_need_their_revision_and_the_node_entry_is_never_served() {
    let root = checkout();
    let host = host(root.path());
    let ui_rev = host.graph()["entries"].as_array().unwrap().iter().find(|e| e["id"] == "pkg-ui").unwrap()["rev"].as_str().unwrap().to_string();
    let chunk = text(host.serve("/plugins/pkg-ui/client.extra.js", Some(&format!("rev={ui_rev}"))));
    assert!(chunk.starts_with("EXTRA_CHUNK()"), "{chunk}");
    assert!(chunk.contains(&format!("sourceMappingURL=client.extra.js.map?rev={ui_rev}")));
    assert!(host.serve("/plugins/pkg-ui/client.extra.js", Some("rev=stale")).is_none());
    assert!(host.serve("/plugins/pkg-ui/client.extra.js", None).is_none());
    assert!(host.serve("/plugins/pkg-ui/client.node.js", Some(&format!("rev={ui_rev}"))).is_none(), "the Node entry must stay private");
    assert!(host.serve("/plugins/pkg-ui/client.missing.js", Some(&format!("rev={ui_rev}"))).is_none());
    assert!(host.serve("/plugins/pkg-ui/..%2f..%2fpackage.json", Some(&format!("rev={ui_rev}"))).is_none());
    assert!(host.serve("/plugins/pkg-ui/package.json", Some(&format!("rev={ui_rev}"))).is_none());
}

#[test]
fn a_changed_bundle_changes_its_revision() {
    let root = checkout();
    let before = host(root.path()).graph()["rev"].clone();
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(root.path().join("packages/g/a/lib/client.js"), "API_BODY_V2()\n// longer\n").unwrap();
    assert_ne!(host(root.path()).graph()["rev"], before, "a stale cache key would serve old code");
}

#[test]
fn a_roster_that_would_boot_half_a_ui_fails_loud() {
    // A name no client package declares is a platform module or host service.
    let platform = checkout();
    package(platform.path(), "w", "pkg-wants", web(&["react", "pkg-host-service"], false, &[]), Some("W()"));
    assert_eq!(host(platform.path()).package_count(), 4);

    let no_bundle = checkout();
    package(no_bundle.path(), "n", "pkg-unbuilt", web(&[], false, &[]), None);
    let error = load_error(no_bundle.path());
    assert!(error.contains("pkg-unbuilt") && error.contains("build the TS checkout"), "{error}");

    let cycle = checkout();
    package(cycle.path(), "c1", "pkg-c1", web(&[], false, &["pkg-c2"]), Some("1"));
    package(cycle.path(), "c2", "pkg-c2", web(&[], false, &["pkg-c1"]), Some("2"));
    assert!(load_error(cycle.path()).contains("cycle"));

    let own = checkout();
    package(own.path(), "s", "pkg-self", web(&[], false, &["pkg-self"]), Some("1"));
    assert!(load_error(own.path()).contains("its own package"));

    let headless = tempfile::tempdir().unwrap();
    package(headless.path(), "a", "pkg-api", web(&[], false, &[]), Some("1"));
    assert!(load_error(headless.path()).contains(MODULES));

    let empty = tempfile::tempdir().unwrap();
    fs::create_dir_all(empty.path().join("packages")).unwrap();
    assert!(load_error(empty.path()).contains("declares dsh.client"));
    assert!(load_error(Path::new("/nonexistent-root")).contains("no packages/"));
}

#[test]
fn an_allowlist_names_the_roster_and_an_unknown_name_is_refused() {
    let root = checkout();
    let only = BootHost::load(&BootConfig {
        packages: Some(vec![MODULES.into(), "pkg-api".into()]),
        ..config(root.path())
    })
    .unwrap();
    assert_eq!(only.package_count(), 2);
    let error = BootHost::load(&BootConfig {
        packages: Some(vec![MODULES.into(), "pkg-typo".into()]),
        ..config(root.path())
    })
    .unwrap_err();
    assert!(error.contains("pkg-typo"), "{error}");
    // `pkg-ui` injects `pkg-api`; dropping the dependency is caught, not ignored.
    let error = BootHost::load(&BootConfig {
        packages: Some(vec![MODULES.into(), "pkg-ui".into()]),
        ..config(root.path())
    })
    .unwrap_err();
    assert!(error.contains("pkg-ui needs pkg-api"), "{error}");
}

#[test]
fn experimental_packages_and_excluded_ones_stay_out_unless_asked() {
    let root = checkout();
    package(root.path(), "e", "@deepseek-ai/dsh-experimental-inspector", web(&[], false, &[]), Some("E()"));
    assert_eq!(host(root.path()).package_count(), 3);
    let with = BootHost::load(&BootConfig { include_experimental: true, ..config(root.path()) }).unwrap();
    assert_eq!(with.package_count(), 4);
    let without_api = BootHost::load(&BootConfig { exclude: vec!["pkg-ui".into()], ..config(root.path()) }).unwrap();
    assert_eq!(without_api.package_count(), 2);
}

fn gateway(root: &Path, assets: &Path) -> Gateway {
    Gateway::new(
        Router::new(RouterConfig {
            agent_presets: dsh_rs::gui::router::default_presets(),
            default_model: DefaultModel { provider: "p".into(), model: "m".into() },
            max_page_events: 10,
            max_search_results: 5,
        max_title_bytes: 120,
        max_bytes: 2 * 1024 * 1024,
        max_file_bytes: 32 * 1024 * 1024,
        max_lines: 5_000,
        max_entries: 2_000,
        default_max_goal_rounds: 256,
        export_compression_level: 6,
        models: Vec::new(),
        }),
        Auth { token: TOKEN.to_string() },
        Some(assets.to_path_buf()),
    )
    .with_boot(host(root))
}

fn headers(authed: bool) -> http::HeaderMap {
    let mut h = http::HeaderMap::new();
    h.insert(http::header::HOST, "127.0.0.1:3081".parse().unwrap());
    if authed {
        h.insert(http::header::AUTHORIZATION, format!("Bearer {TOKEN}").parse().unwrap());
    }
    h
}

#[tokio::test]
async fn the_gateway_injects_the_page_and_guards_the_bundles_with_the_launch_token() {
    let root = checkout();
    let assets = tempfile::tempdir().unwrap();
    fs::write(assets.path().join("index.html"), "<html><head></head><body><div id=\"root\"></div></body></html>").unwrap();
    let gateway = gateway(root.path(), assets.path());
    let ctx = Context::new();

    // The shell page is the one route that answers without a token, and it is injected.
    let (page, extra) = gateway.route(&ctx, &http::Method::GET, "/", &headers(false), b"").await;
    assert_eq!(page.status, 200);
    let page = String::from_utf8(page.body).unwrap();
    assert!(page.contains("__DSH_BOOT__"), "{page}");
    assert!(extra.get(http::header::SET_COOKIE).is_some());

    let batch = gateway_batch_url(&page);
    let (path, query) = batch.split_once('?').unwrap();
    let path = format!("/{path}");
    let (denied, _) = gateway.route_with_query(&ctx, &http::Method::GET, &path, Some(query), &headers(false), b"").await;
    assert_eq!(denied.status, 401, "bundles are not public");
    let (ok, extra) = gateway.route_with_query(&ctx, &http::Method::GET, &path, Some(query), &headers(true), b"").await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.content_type, "text/javascript; charset=utf-8");
    assert_eq!(extra.get(http::header::CACHE_CONTROL).unwrap(), "public, max-age=31536000, immutable");
    let (unknown, _) = gateway.route_with_query(&ctx, &http::Method::GET, &path, Some("??nope/client.js&rev=1"), &headers(true), b"").await;
    assert_eq!(unknown.status, 404);
    let (post, _) = gateway.route_with_query(&ctx, &http::Method::POST, &path, Some(query), &headers(true), b"").await;
    assert_eq!(post.status, 405);
    let (head, _) = gateway.route_with_query(&ctx, &http::Method::HEAD, &path, Some(query), &headers(true), b"").await;
    assert_eq!((head.status, head.body.len()), (200, 0));
}

/// The bootstrap script's `src` from a rendered page.
fn gateway_batch_url(page: &str) -> String {
    let start = page.find("<script src=\"").unwrap() + "<script src=\"".len();
    let end = page[start..].find('"').unwrap();
    page[start..start + end].replace("&amp;", "&")
}

/// The checked-out TS tree, when this machine has one built.
fn real_checkout() -> Option<PathBuf> {
    let root = std::env::var_os("DSH_TS_CHECKOUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    root.join("packages/client/modules/lib/client.js").is_file().then_some(root)
}

#[test]
fn the_real_client_roster_composes_and_every_advertised_url_is_served() {
    let Some(root) = real_checkout() else {
        eprintln!("skipped: no built TS checkout (set DSH_TS_CHECKOUT)");
        return;
    };
    let exclude = ["@deepseek-ai/dsh-client-hmr", "@deepseek-ai/dsh-client-product-analytics"];
    let host = BootHost::load(&BootConfig {
        client_root: root,
        packages: None,
        exclude: exclude.iter().map(|s| s.to_string()).collect(),
        include_experimental: false,
    })
    .expect("the real roster composes");
    assert!(host.package_count() >= 60, "{}", host.package_count());
    let graph = host.graph();
    let in_batches: usize = graph["batches"].as_array().unwrap().iter().map(|b| b["entries"].as_array().unwrap().len()).sum();
    assert_eq!(in_batches, host.package_count(), "every entry belongs to exactly one batch");
    for batch in graph["batches"].as_array().unwrap() {
        let (path, query) = batch["url"].as_str().unwrap().split_once('?').unwrap();
        let body = host.serve(&format!("/{path}"), Some(query)).expect("advertised batch is served");
        assert!(!body.body.is_empty());
        let (map_path, _) = (path, query);
        let ids: Vec<&str> = batch["entries"].as_array().unwrap().iter().map(|e| e.as_str().unwrap()).collect();
        let map_query = format!("?{}&rev={}", ids.iter().map(|i| format!("{i}/client.js.map")).collect::<Vec<_>>().join(","), batch["rev"].as_str().unwrap());
        let _ = map_path;
        let map: Value = serde_json::from_slice(&host.serve("/plugins/", Some(&map_query)).expect("map served").body).unwrap();
        assert_eq!(map["sections"].as_array().unwrap().len(), ids.len());
    }
    // The module system is alone in the blocking bootstrap batch.
    assert_eq!(graph["batches"][0]["phase"], "bootstrap");
    assert_eq!(graph["batches"][0]["entries"], json!([MODULES]));
}
