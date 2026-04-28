//! Docker-backed integration tests (opt-in).
//!
//! Run with Docker available:
//!   `CUBELIT_INTEGRATION_TESTS=1 cargo test -p cubelit-cli -- --ignored`
//!
//! Tests allocate real containers and pull images; they clean up after
//! themselves via `delete_server`. Do NOT run these in CI without Docker.

use std::sync::Arc;

use cubelit_core::events::NoopSink;
use cubelit_core::server::{CreateServerConfig, LocalServerHost, ServerLifecycle};

#[test]
#[ignore]
fn install_start_stop_remove_minecraft() {
    // Requires: Docker daemon running locally + network access for image pull.
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let recipes_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../src-tauri/recipes");

        let host = LocalServerHost::new(tmp.path().to_path_buf(), recipes_dir)
            .await
            .expect("LocalServerHost::new");

        let sink: Arc<dyn cubelit_core::events::EventSink> = Arc::new(NoopSink);

        // --- Install ---
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

        // --- Verify listed ---
        let servers = host.list_servers().await.expect("list_servers after install");
        assert!(
            servers.iter().any(|s| s.id == server.id),
            "newly created server should appear in list"
        );

        // --- Start ---
        host.start_server(&server.id, sink.clone())
            .await
            .expect("start_server");

        // --- Stop ---
        host.stop_server(&server.id).await.expect("stop_server");

        // --- Remove (delete data too) ---
        host.delete_server(&server.id, true)
            .await
            .expect("delete_server");

        // --- Verify gone ---
        let servers = host.list_servers().await.expect("list_servers after remove");
        assert!(
            !servers.iter().any(|s| s.id == server.id),
            "removed server should not appear in list"
        );
    });
}
