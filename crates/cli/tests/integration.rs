//! Docker-backed integration tests (opt-in).
//!
//! All tests here require:
//!   - A running Docker daemon on the local socket.
//!   - Network access to pull images.
//!   - `CUBELIT_SMOKE=1` set in the environment.
//!
//! Run all smoke tests:
//!   `CUBELIT_SMOKE=1 SQLX_OFFLINE=true cargo test -p cubelit-cli -- --ignored`
//!
//! Run a single game:
//!   `CUBELIT_SMOKE=1 SQLX_OFFLINE=true cargo test -p cubelit-cli -- --ignored smoke_minecraft_java`
//!
//! Each test allocates real containers and pulls images; all containers are
//! cleaned up after the test via `delete_server` (or kept on failure when
//! `CUBELIT_SMOKE_KEEP_ON_FAIL=1` is set).

use std::sync::Arc;
use std::time::Duration;

use cubelit_core::events::NoopSink;
use cubelit_core::server::{CreateServerConfig, LocalServerHost, ServerLifecycle};
use cubelit_core::smoke::{run_smoke, SmokeOptions};

/// Returns true when the CUBELIT_SMOKE env var is set to "1".
fn smoke_enabled() -> bool {
    std::env::var("CUBELIT_SMOKE").map(|v| v == "1").unwrap_or(false)
}

fn keep_on_fail() -> bool {
    std::env::var("CUBELIT_SMOKE_KEEP_ON_FAIL")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn recipes_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/recipes")
}

async fn test_host() -> (LocalServerHost, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let host = LocalServerHost::new(tmp.path().to_path_buf(), recipes_dir())
        .await
        .expect("LocalServerHost::new");
    (host, tmp)
}

fn smoke_opts_for(recipe_id: &str, overall_timeout: Duration) -> SmokeOptions {
    SmokeOptions {
        recipe_ids: vec![recipe_id.to_string()],
        overall_timeout,
        keep_on_failure: keep_on_fail(),
        port_offset: 20_000,
        parallel: 1,
    }
}

// ─── Legacy lifecycle test (no Docker smoke, tests the trait surface) ─────────

/// Install → list → start → stop → remove for Minecraft Java.
/// This test exercises the full `ServerLifecycle` trait against a real Docker
/// daemon. It does not use the smoke harness.
#[test]
#[ignore]
fn install_start_stop_remove_minecraft() {
    if !smoke_enabled() {
        return;
    }
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let (host, _tmp) = test_host().await;
        let sink: Arc<dyn cubelit_core::events::EventSink> = Arc::new(NoopSink);

        let config = CreateServerConfig {
            name: "cli-integration-test".into(),
            recipe_id: "minecraft-java".into(),
            port_overrides: None,
            env_overrides: None,
            volume_path: None,
            tag_override: None,
        };
        let server = host
            .create_server(config, sink.clone())
            .await
            .expect("create_server");
        assert!(!server.id.is_empty(), "server id should not be empty");

        let servers = host.list_servers().await.expect("list_servers after install");
        assert!(
            servers.iter().any(|s| s.id == server.id),
            "newly created server should appear in list"
        );

        host.start_server(&server.id, sink.clone())
            .await
            .expect("start_server");
        host.stop_server(&server.id).await.expect("stop_server");
        host.delete_server(&server.id, true)
            .await
            .expect("delete_server");

        let servers = host.list_servers().await.expect("list_servers after remove");
        assert!(
            !servers.iter().any(|s| s.id == server.id),
            "removed server should not appear in list"
        );
    });
}

// ─── Per-recipe smoke tests ───────────────────────────────────────────────────

macro_rules! smoke_test {
    ($fn_name:ident, $recipe_id:literal) => {
        smoke_test!($fn_name, $recipe_id, 600);
    };
    ($fn_name:ident, $recipe_id:literal, $timeout_secs:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $fn_name() {
            if !smoke_enabled() {
                return;
            }
            let (host, _tmp) = test_host().await;
            let opts = smoke_opts_for($recipe_id, Duration::from_secs($timeout_secs));
            let events: Arc<dyn cubelit_core::events::EventSink> = Arc::new(NoopSink);
            let report = run_smoke(&host, opts, events)
                .await
                .expect("run_smoke should not error");
            assert_eq!(report.results.len(), 1);
            let result = &report.results[0];
            assert!(
                result.outcome.is_passing(),
                "smoke test for '{}' failed with outcome: {}",
                $recipe_id,
                result.outcome.label()
            );
        }
    };
}

smoke_test!(smoke_minecraft_java, "minecraft-java");
smoke_test!(smoke_minecraft_bedrock, "minecraft-bedrock");
smoke_test!(smoke_valheim, "valheim");
smoke_test!(smoke_terraria, "terraria");
smoke_test!(smoke_palworld, "palworld");
smoke_test!(smoke_project_zomboid, "project-zomboid");
smoke_test!(smoke_rust_game, "rust-game");
smoke_test!(smoke_ark, "ark");
smoke_test!(smoke_ark_survival_ascended, "ark-survival-ascended", 3600);
smoke_test!(smoke_cs2, "cs2");
smoke_test!(smoke_fivem, "fivem");

// ─── Aggregate smoke test ─────────────────────────────────────────────────────

/// Boot every available recipe sequentially. Fails if any game fails.
///
/// Run with:
///   `CUBELIT_SMOKE=1 SQLX_OFFLINE=true cargo test -p cubelit-cli -- --ignored smoke_all_recipes`
#[tokio::test]
#[ignore]
async fn smoke_all_recipes() {
    if !smoke_enabled() {
        return;
    }

    let (host, _tmp) = test_host().await;
    let opts = SmokeOptions {
        recipe_ids: vec![],
        overall_timeout: Duration::from_secs(600),
        keep_on_failure: keep_on_fail(),
        port_offset: 20_000,
        parallel: 1,
    };
    let events: Arc<dyn cubelit_core::events::EventSink> = Arc::new(NoopSink);
    let report = run_smoke(&host, opts, events)
        .await
        .expect("run_smoke should not error");

    let mut failures = Vec::new();
    for result in &report.results {
        println!(
            "[{}] {} — {}",
            if result.outcome.is_passing() { "PASS" } else { "FAIL" },
            result.recipe_id,
            result.outcome.label()
        );
        if !result.outcome.is_passing() {
            failures.push(result.recipe_id.clone());
        }
    }

    assert!(
        failures.is_empty(),
        "The following recipes failed smoke: {:?}",
        failures
    );
}
