//! `LocalServerHost` — the only `ServerRunner` + `ServerLifecycle`
//! implementation in v0.1.8.
//!
//! It owns:
//!   * a `bollard::Docker` connected to the local Docker socket,
//!   * a `sqlx::SqlitePool` pointing at `data_dir/cubelit.db`,
//!   * `data_dir` (where Cubelit-managed sidecar data lives),
//!   * `recipes_dir` (read-only directory of bundled recipe JSON files).
//!
//! The struct fields are public so the desktop `lib.rs` can pull out the
//! Docker handle and DB pool when it spawns the crash watcher. They form
//! a stable internal API, not a public-API stability surface.
//!
//! Construction goes through `LocalServerHost::new`, which:
//!   1. Connects to the local Docker daemon.
//!   2. Creates `data_dir` if it doesn't exist.
//!   3. Opens the SQLite pool with WAL journaling.
//!   4. Runs migrations.
//!
//! All the `create_server` / FiveM-sidecar logic that used to live in
//! `src-tauri/src/commands/docker_commands.rs` lives here now, so the
//! Tauri command modules collapse to ~10-line shims.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tracing::{error, info, warn};

use crate::db::{models::Cubelit, queries, run_migrations};
use crate::docker::{containers, images, stats::ContainerStats};
use crate::error::{CoreError, CoreResult};
use crate::events::{CoreEvent, EventSink, ServerCreateProgress};
use crate::recipes;

use super::lifecycle::ServerLifecycle;
use super::minecraft;
use super::runner::ServerRunner;
use super::types::CreateServerConfig;
use super::watchers::{
    spawn_readiness_watcher, validate_env_vars, verify_container_status,
};

pub struct LocalServerHost {
    pub docker: bollard::Docker,
    pub db: SqlitePool,
    pub data_dir: PathBuf,
    pub recipes_dir: PathBuf,
}

/// Tracks partial resources created during [`LocalServerHost::create_server`] so
/// they can be rolled back if provisioning fails at any later step.
///
/// Cleanup order: main container → sidecar container → Docker network →
/// DB row → auto-generated volume directory.  Errors during cleanup are
/// logged but never propagated — the caller always receives the original
/// create error.
struct CreateGuard {
    id: String,
    /// `Some(path)` only when CubeLit auto-generated the volume path AND the
    /// directory did not exist before this create attempt.  `None` means either
    /// the user supplied the path (never remove) or the directory already existed
    /// (don't wipe pre-existing data).
    auto_volume_path: Option<String>,
    db_row_inserted: bool,
    main_container_id: Option<String>,
    sidecar_container_id: Option<String>,
    network_name: Option<String>,
}

impl CreateGuard {
    fn new(id: String, auto_volume_path: Option<String>) -> Self {
        Self {
            id,
            auto_volume_path,
            db_row_inserted: false,
            main_container_id: None,
            sidecar_container_id: None,
            network_name: None,
        }
    }

    async fn cleanup(self, docker: &bollard::Docker, db: &SqlitePool) {
        // Remove main container (force=true handles containers that are still running)
        if let Some(ref cid) = self.main_container_id {
            if let Err(e) = containers::remove_container(docker, cid).await {
                error!(server_id = %self.id, container_id = %cid, error = %e,
                    "Cleanup: failed to remove main container");
            }
        }

        // Stop and remove sidecar container
        if let Some(ref sidecar_id) = self.sidecar_container_id {
            let _ = containers::stop_container(docker, sidecar_id).await;
            if let Err(e) = containers::remove_container(docker, sidecar_id).await {
                error!(server_id = %self.id, container_id = %sidecar_id, error = %e,
                    "Cleanup: failed to remove sidecar container");
            }
        }

        // Remove Docker network (after containers are detached/removed so no active endpoints)
        if let Some(ref net) = self.network_name {
            if let Err(e) = docker.remove_network(net).await {
                error!(server_id = %self.id, network = %net, error = %e,
                    "Cleanup: failed to remove Docker network");
            }
        }

        // Delete DB row
        if self.db_row_inserted {
            if let Err(e) = queries::delete_cubelit(db, &self.id).await {
                error!(server_id = %self.id, error = %e,
                    "Cleanup: failed to delete DB row");
            }
        }

        // Remove auto-generated volume dir only when CubeLit created it fresh this attempt
        if let Some(ref path) = self.auto_volume_path {
            if let Err(e) = std::fs::remove_dir_all(path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    error!(server_id = %self.id, path = %path, error = %e,
                        "Cleanup: failed to remove auto volume dir");
                }
            }
        }
    }
}

impl LocalServerHost {
    /// Connect to the local Docker socket, open the SQLite pool, and run
    /// migrations. Returns ready-to-use `LocalServerHost`.
    pub async fn new(data_dir: PathBuf, recipes_dir: PathBuf) -> CoreResult<Self> {
        let docker = bollard::Docker::connect_with_local_defaults()?;

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("cubelit.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        sqlx::query("PRAGMA journal_mode=WAL;").execute(&db).await?;

        run_migrations(&db).await?;

        Ok(Self {
            docker,
            db,
            data_dir,
            recipes_dir,
        })
    }

    /// Returns `true` if `path` exists and contains at least one entry.
    /// Used by FiveM provisioning to detect leftover volume content from
    /// a previously-deleted server with the same name.
    fn dir_has_contents(path: &std::path::Path) -> bool {
        path.exists()
            && path
                .read_dir()
                .is_ok_and(|mut d| d.next().is_some())
    }

    /// Compute the FiveM-only `txData` extra bind list. Standard recipes
    /// return an empty vec.
    fn extra_binds_for(&self, cubelit: &Cubelit) -> Vec<String> {
        if cubelit.recipe_id == "fivem" {
            let txdata_dir = self
                .data_dir
                .join("servers")
                .join(&cubelit.id)
                .join("txdata");
            vec![format!("{}:/txData", txdata_dir.to_string_lossy())]
        } else {
            vec![]
        }
    }

    /// Create host-side directories for recipe volumes beyond the primary (index 0).
    /// Each additional volume gets a subdirectory under `volume_path` named after
    /// the last segment of its container path (e.g. `/opt/valheim` → `{volume_path}/valheim`).
    fn create_additional_volume_dirs(
        volume_path: &str,
        recipe: &recipes::Recipe,
    ) -> CoreResult<()> {
        for v in recipe.volumes.iter().skip(1) {
            let segment = additional_volume_subdir(&v.container_path);
            std::fs::create_dir_all(format!("{}/{}", volume_path, segment))?;
        }
        Ok(())
    }

    fn fivem_mysql_connection_string(db_container_name: &str, db_password: &str) -> String {
        if db_password.is_empty() {
            format!("mysql://root@{}:3306/fivem", db_container_name)
        } else {
            format!(
                "mysql://root:{}@{}:3306/fivem",
                db_password, db_container_name
            )
        }
    }

    /// Provision the FiveM MariaDB sidecar: pulls MariaDB image, creates
    /// the cubelit-{id}-net network, creates and starts the MariaDB
    /// container, persists the sidecar info on the server row, and
    /// inserts `MYSQL_CONNECTION_STRING` + `NO_DEFAULT_CONFIG` into
    /// `env`. The caller's `env` map is mutated so it gets re-persisted
    /// before the primary container is created.
    async fn provision_fivem_sidecar(
        &self,
        id: &str,
        env: &mut HashMap<String, String>,
        events: &dyn EventSink,
        guard: &mut CreateGuard,
    ) -> CoreResult<()> {
        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "creating".into(),
            progress: Some(0.5),
            message: "Setting up MariaDB database...".into(),
        }));

        // Read DB config from env (set by user in the wizard, defaults from recipe)
        let db_password = env.get("DB_PASSWORD").cloned().unwrap_or_default();
        let db_host_port: u16 = env
            .get("DB_HOST_PORT")
            .and_then(|p| p.parse().ok())
            .unwrap_or(3306);

        // Pull MariaDB image
        let mariadb_image = "mariadb:10.11";
        images::pull_image(&self.docker, mariadb_image, events).await?;

        // Create Docker network
        let network_name = format!("cubelit-{}-net", id);
        let network_config = bollard::models::NetworkCreateRequest {
            name: network_name.clone(),
            driver: Some("bridge".to_string()),
            ..Default::default()
        };
        self.docker.create_network(network_config).await?;
        guard.network_name = Some(network_name.clone());

        // Create MariaDB data directory
        let db_data_dir = self.data_dir.join("servers").join(id).join("db");
        std::fs::create_dir_all(&db_data_dir)?;

        // Create MariaDB container
        let db_container_name = format!("cubelit-{}-db", id);
        let mut db_labels = HashMap::new();
        db_labels.insert("cubelit.id".to_string(), id.to_string());
        db_labels.insert("cubelit.role".to_string(), "database".to_string());
        db_labels.insert("cubelit.managed".to_string(), "true".to_string());

        let mut db_env = vec!["MYSQL_DATABASE=fivem".to_string()];
        if db_password.is_empty() {
            db_env.push("MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=1".to_string());
        } else {
            db_env.push(format!("MARIADB_ROOT_PASSWORD={}", db_password));
        }

        let mut db_port_bindings = HashMap::new();
        db_port_bindings.insert(
            "3306/tcp".to_string(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(db_host_port.to_string()),
            }]),
        );

        let db_host_config = bollard::models::HostConfig {
            binds: Some(vec![format!(
                "{}:/var/lib/mysql",
                db_data_dir.to_string_lossy()
            )]),
            port_bindings: Some(db_port_bindings),
            restart_policy: Some(bollard::models::RestartPolicy {
                name: Some(bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            }),
            ..Default::default()
        };

        let db_config = bollard::models::ContainerCreateBody {
            image: Some(mariadb_image.to_string()),
            env: Some(db_env),
            labels: Some(db_labels),
            host_config: Some(db_host_config),
            ..Default::default()
        };

        let db_create_opts = bollard::query_parameters::CreateContainerOptions {
            name: Some(db_container_name.clone()),
            platform: String::from(""),
        };

        let db_response = self
            .docker
            .create_container(Some(db_create_opts), db_config)
            .await?;
        let sidecar_id = db_response.id;
        guard.sidecar_container_id = Some(sidecar_id.clone());

        // Connect MariaDB container to the network
        self.docker
            .connect_network(
                &network_name,
                bollard::models::NetworkConnectRequest {
                    container: db_container_name.clone(),
                    endpoint_config: None,
                },
            )
            .await?;

        // Start MariaDB
        containers::start_container(&self.docker, &sidecar_id).await?;

        // Update cubelit with sidecar info
        queries::update_cubelit_sidecar(&self.db, id, &sidecar_id, mariadb_image).await?;

        // Add MySQL connection string to FiveM env (root user, password may be empty)
        let conn_str = Self::fivem_mysql_connection_string(&db_container_name, &db_password);
        env.insert("MYSQL_CONNECTION_STRING".to_string(), conn_str);

        // txAdmin mode: skip server.cfg and let txAdmin manage the server via its web UI.
        // This MUST be set, and LICENSE_KEY MUST NOT be passed to the container — the
        // spritsail/fivem entrypoint exits with error if both are present.
        env.insert("NO_DEFAULT_CONFIG".to_string(), "1".to_string());

        // Create txAdmin data directory (mounted separately at /txData inside the container)
        let txdata_dir = self.data_dir.join("servers").join(id).join("txdata");
        std::fs::create_dir_all(&txdata_dir)?;

        // Update the cubelit's environment in DB
        let updated_env = serde_json::to_string(env).unwrap_or_default();
        queries::update_cubelit_environment(&self.db, id, &updated_env).await?;

        Ok(())
    }
}

// ─── ServerRunner impl ───────────────────────────────────────────────────────

#[async_trait]
impl ServerRunner for LocalServerHost {
    async fn pull_image(&self, image: &str, events: &dyn EventSink) -> CoreResult<()> {
        images::pull_image(&self.docker, image, events).await
    }

    async fn create_container(
        &self,
        cubelit: &Cubelit,
        extra_binds: &[String],
        server_cmd: Option<Vec<String>>,
    ) -> CoreResult<String> {
        containers::create_container(&self.docker, cubelit, extra_binds, server_cmd, &[]).await
    }

    async fn start_container(&self, container_id: &str) -> CoreResult<()> {
        containers::start_container(&self.docker, container_id).await
    }

    async fn stop_container(&self, container_id: &str) -> CoreResult<()> {
        containers::stop_container(&self.docker, container_id).await
    }

    async fn restart_container(&self, container_id: &str) -> CoreResult<()> {
        containers::restart_container(&self.docker, container_id).await
    }

    async fn remove_container(&self, container_id: &str) -> CoreResult<()> {
        containers::remove_container(&self.docker, container_id).await
    }

    async fn is_running(&self, container_id: &str) -> bool {
        verify_container_status(&self.docker, container_id).await == "running"
    }

    async fn container_logs(&self, container_id: &str, lines: u64) -> CoreResult<Vec<String>> {
        use bollard::query_parameters::LogsOptions;
        use futures_util::StreamExt;

        let opts = LogsOptions {
            stdout: true,
            stderr: true,
            tail: lines.to_string(),
            ..Default::default()
        };

        let mut stream = self.docker.logs(container_id, Some(opts));
        let mut result = Vec::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(log) => {
                    let line = log.to_string();
                    if !line.is_empty() {
                        result.push(line);
                    }
                }
                Err(_) => break,
            }
        }

        Ok(result)
    }

    async fn container_stats(&self, container_id: &str) -> CoreResult<ContainerStats> {
        crate::docker::stats::get_container_stats(&self.docker, container_id).await
    }
}

// ─── ServerLifecycle impl ────────────────────────────────────────────────────

#[async_trait]
impl ServerLifecycle for LocalServerHost {
    async fn create_server(
        &self,
        config: CreateServerConfig,
        events: Arc<dyn EventSink>,
    ) -> CoreResult<Cubelit> {
        info!(name = %config.name, recipe = %config.recipe_id, "Creating server");
        let recipe = recipes::get_recipe(&self.recipes_dir, &config.recipe_id)?;

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "preparing".into(),
            progress: Some(0.0),
            message: "Preparing server configuration...".into(),
        }));

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        // Captured up front — `config` is partially moved further down.
        let readiness_timeout_override = config.readiness_timeout_override_secs;

        // Use user-provided volume path or default to ~/Cubelit/{sanitized-name}.
        // For FiveM: the spritsail/fivem image only copies its default resources when /config is
        // empty on first boot. If the default path already has content (e.g. from a previously
        // deleted server with the same name whose files were kept), fall back to a unique path
        // using the server ID so the image always starts with an empty volume.
        let volume_path_is_auto = config.volume_path.is_none();
        let volume_path = if let Some(ref vp) = config.volume_path {
            vp.clone()
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| self.data_dir.clone());
            let sanitized = config.name.replace(
                |c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_',
                "",
            );
            let base_path = home.join("Cubelit").join(&sanitized);
            if config.recipe_id == "fivem" && Self::dir_has_contents(&base_path) {
                home.join("Cubelit")
                    .join(format!("{}-{}", sanitized, &id[..8]))
                    .to_string_lossy()
                    .to_string()
            } else {
                base_path.to_string_lossy().to_string()
            }
        };
        // Decide cleanup path BEFORE creating the directory: only remove if CubeLit
        // auto-generated it AND it didn't exist yet (don't wipe pre-existing data).
        let cleanup_vol = cleanup_volume_path(&volume_path, !volume_path_is_auto);
        std::fs::create_dir_all(&volume_path)?;
        Self::create_additional_volume_dirs(&volume_path, &recipe)?;

        // Get container mount path from recipe (e.g. "/data" for Minecraft, "/config" for FiveM)
        let container_mount_path = recipe
            .volumes
            .first()
            .map(|v| v.container_path.clone())
            .unwrap_or_else(|| "/data".to_string());

        let mut env: HashMap<String, String> = recipe
            .environment
            .iter()
            .map(|e| (e.key.clone(), e.default_value.clone()))
            .collect();
        if let Some(overrides) = config.env_overrides {
            env.extend(overrides);
        }

        // Validate env vars before touching Docker
        validate_env_vars(&env)?;

        // Seed recipe-declared files into the fresh volume before first boot
        // (some entrypoints only patch config files that already exist).
        write_seed_files(&recipe, std::path::Path::new(&volume_path), &env);

        // Use protocol-aware port keys: "25565/tcp", "30120/udp"
        let mut ports: HashMap<String, u16> = recipe
            .ports
            .iter()
            .map(|p| (format!("{}/{}", p.container_port, p.protocol), p.default_host_port))
            .collect();
        if let Some(overrides) = config.port_overrides {
            ports.extend(overrides);
        }

        let tag = config
            .tag_override
            .as_deref()
            .unwrap_or(&recipe.default_tag);
        let image = format!("{}:{}", recipe.docker_image, tag);

        let cubelit = Cubelit {
            id: id.clone(),
            name: config.name,
            game: recipe.name.clone(),
            recipe_id: config.recipe_id,
            docker_image: image.clone(),
            container_id: None,
            status: "created".into(),
            port_mappings: serde_json::to_string(&ports).unwrap_or_default(),
            environment: serde_json::to_string(&env).unwrap_or_default(),
            volume_path,
            container_mount_path,
            sidecar_container_id: None,
            sidecar_image: None,
            created_at: now.clone(),
            updated_at: now,
        };

        queries::insert_cubelit(&self.db, &cubelit).await?;
        let mut guard = CreateGuard::new(id.clone(), cleanup_vol);
        guard.db_row_inserted = true;

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "pulling".into(),
            progress: Some(0.2),
            message: format!("Pulling image {}...", image),
        }));

        if let Err(e) = images::pull_image(&self.docker, &image, events.as_ref()).await {
            error!(server_id = %id, error = %e, "create_server: image pull failed; cleaning up");
            guard.cleanup(&self.docker, &self.db).await;
            return Err(e);
        }

        // FiveM sidecar: MariaDB + Docker network
        if cubelit.recipe_id == "fivem" {
            if let Err(e) = self
                .provision_fivem_sidecar(&id, &mut env, events.as_ref(), &mut guard)
                .await
            {
                error!(server_id = %id, error = %e, "create_server: FiveM sidecar failed; cleaning up");
                guard.cleanup(&self.docker, &self.db).await;
                return Err(e);
            }
        }

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "creating".into(),
            progress: Some(0.7),
            message: "Creating container...".into(),
        }));

        // Re-read cubelit from DB to get updated env (with sidecar connection string)
        let cubelit = match queries::get_cubelit(&self.db, &id).await {
            Ok(c) => c,
            Err(e) => {
                error!(server_id = %id, error = %e, "create_server: DB refresh failed; cleaning up");
                guard.cleanup(&self.docker, &self.db).await;
                return Err(e);
            }
        };
        let mut extra_binds = self.extra_binds_for(&cubelit);
        extra_binds.extend(additional_volume_binds(&cubelit.volume_path, &recipe));

        let container_id = match containers::create_container(
            &self.docker,
            &cubelit,
            &extra_binds,
            recipe.server_cmd.clone(),
            &recipe.cap_add,
        )
        .await
        {
            Ok(cid) => cid,
            Err(e) => {
                error!(server_id = %id, error = %e, "create_server: container creation failed; cleaning up");
                guard.cleanup(&self.docker, &self.db).await;
                return Err(e);
            }
        };
        guard.main_container_id = Some(container_id.clone());

        // If FiveM, connect the main container to the network too
        if cubelit.recipe_id == "fivem" {
            let network_name = format!("cubelit-{}-net", id);
            let container_name = format!("cubelit-{}", id);
            if let Err(e) = self
                .docker
                .connect_network(
                    &network_name,
                    bollard::models::NetworkConnectRequest {
                        container: container_name,
                        endpoint_config: None,
                    },
                )
                .await
            {
                error!(server_id = %id, error = %e, "create_server: network connect failed; cleaning up");
                guard.cleanup(&self.docker, &self.db).await;
                return Err(e.into());
            }
        }

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "starting".into(),
            progress: Some(0.9),
            message: "Starting server...".into(),
        }));

        if let Err(e) = containers::start_container(&self.docker, &container_id).await {
            error!(server_id = %id, error = %e, "create_server: container start failed; cleaning up");
            guard.cleanup(&self.docker, &self.db).await;
            return Err(e);
        }

        // Post-start: the container is now started. No cleanup on later failures —
        // the server may be running even if status-update DB writes fail.
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        // Track whether a readiness watcher was spawned so the completion
        // event carries an accurate message (watcher ≠ ready).
        let watcher_spawned = if running {
            if let Some(ref r) = recipe.readiness {
                let pattern = r.log_pattern.clone();
                let timeout = std::time::Duration::from_secs(
                    readiness_timeout_override.unwrap_or(r.timeout_secs),
                );
                queries::update_cubelit_status(
                    &self.db,
                    &id,
                    "starting",
                    Some(Some(&container_id)),
                )
                .await?;
                spawn_readiness_watcher(
                    self.docker.clone(),
                    self.db.clone(),
                    events.clone(),
                    id.clone(),
                    container_id.clone(),
                    pattern,
                    timeout,
                );
                true
            } else {
                queries::update_cubelit_status(
                    &self.db,
                    &id,
                    "running",
                    Some(Some(&container_id)),
                )
                .await?;
                false
            }
        } else {
            queries::update_cubelit_status(&self.db, &id, "error", Some(Some(&container_id)))
                .await?;
            false
        };

        let updated = queries::get_cubelit(&self.db, &id).await?;

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "ready".into(),
            progress: Some(1.0),
            message: if !running {
                "Server started but may have encountered an error.".into()
            } else if watcher_spawned {
                // Container is up but the readiness pattern hasn't matched yet.
                // The watcher will emit ServerStatusChanged when it does.
                "Server is starting up — monitoring logs for readiness...".into()
            } else {
                "Server is ready!".into()
            },
        }));

        if running {
            info!(server_id = %id, container_id = %container_id, "Server created and running");
        } else {
            error!(
                server_id = %id,
                container_id = %container_id,
                "Server created but container did not start"
            );
        }

        Ok(updated)
    }

    async fn start_server(&self, id: &str, events: Arc<dyn EventSink>) -> CoreResult<()> {
        info!(server_id = %id, "Starting server");
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let container_id = cubelit
            .container_id
            .ok_or_else(|| {
                CoreError::NotFound("No container associated with this server".into())
            })?;

        let recipe = recipes::get_recipe(&self.recipes_dir, &cubelit.recipe_id).ok();

        // Also start sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::start_container(&self.docker, sidecar_id).await;
        }

        containers::start_container(&self.docker, &container_id).await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        if running {
            if let Some(ref r) = recipe.as_ref().and_then(|r| r.readiness.as_ref()).cloned() {
                let pattern = r.log_pattern.clone();
                let timeout = std::time::Duration::from_secs(r.timeout_secs);
                queries::update_cubelit_status(&self.db, id, "starting", None).await?;
                spawn_readiness_watcher(
                    self.docker.clone(),
                    self.db.clone(),
                    events,
                    id.to_string(),
                    container_id.clone(),
                    pattern,
                    timeout,
                );
            } else {
                queries::update_cubelit_status(&self.db, id, "running", None).await?;
            }
        } else {
            queries::update_cubelit_status(&self.db, id, "error", None).await?;
        }

        if running {
            info!(server_id = %id, container_id = %container_id, "Server started");
        } else {
            error!(
                server_id = %id,
                container_id = %container_id,
                "Server started but container did not come up"
            );
        }

        Ok(())
    }

    async fn stop_server(&self, id: &str) -> CoreResult<()> {
        info!(server_id = %id, "Stopping server");
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let container_id = cubelit
            .container_id
            .ok_or_else(|| {
                CoreError::NotFound("No container associated with this server".into())
            })?;

        containers::stop_container(&self.docker, &container_id).await?;

        // Also stop sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::stop_container(&self.docker, sidecar_id).await;
        }

        queries::update_cubelit_status(&self.db, id, "stopped", None).await?;
        info!(server_id = %id, "Server stopped");

        Ok(())
    }

    async fn restart_server(&self, id: &str, events: Arc<dyn EventSink>) -> CoreResult<()> {
        info!(server_id = %id, "Restarting server");
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let container_id = cubelit
            .container_id
            .ok_or_else(|| {
                CoreError::NotFound("No container associated with this server".into())
            })?;

        let recipe = recipes::get_recipe(&self.recipes_dir, &cubelit.recipe_id).ok();

        // Also restart sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::restart_container(&self.docker, sidecar_id).await;
        }

        containers::restart_container(&self.docker, &container_id).await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        if running {
            if let Some(ref r) = recipe.as_ref().and_then(|r| r.readiness.as_ref()).cloned() {
                let pattern = r.log_pattern.clone();
                let timeout = std::time::Duration::from_secs(r.timeout_secs);
                queries::update_cubelit_status(&self.db, id, "starting", None).await?;
                spawn_readiness_watcher(
                    self.docker.clone(),
                    self.db.clone(),
                    events,
                    id.to_string(),
                    container_id.clone(),
                    pattern,
                    timeout,
                );
            } else {
                queries::update_cubelit_status(&self.db, id, "running", None).await?;
            }
        } else {
            queries::update_cubelit_status(&self.db, id, "error", None).await?;
        }

        if running {
            info!(server_id = %id, container_id = %container_id, "Server restarted");
        } else {
            error!(
                server_id = %id,
                container_id = %container_id,
                "Server restarted but container did not come back up"
            );
        }

        Ok(())
    }

    async fn delete_server(&self, id: &str, delete_data: bool) -> CoreResult<()> {
        info!(server_id = %id, delete_data = %delete_data, "Deleting server");
        let cubelit = queries::get_cubelit(&self.db, id).await?;

        if let Some(container_id) = &cubelit.container_id {
            let _ = containers::stop_container(&self.docker, container_id).await;
            let _ = containers::remove_container(&self.docker, container_id).await;
        }

        // Remove sidecar container if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::stop_container(&self.docker, sidecar_id).await;
            let _ = containers::remove_container(&self.docker, sidecar_id).await;
        }

        // Remove Docker network if it was a FiveM server
        if cubelit.sidecar_container_id.is_some() {
            let network_name = format!("cubelit-{}-net", cubelit.id);
            let _ = self.docker.remove_network(&network_name).await;
        }

        queries::delete_cubelit(&self.db, id).await?;

        if delete_data {
            // Some game images (Valheim, Project Zomboid) write volume files as
            // root, which a plain remove_dir_all can't delete — fall back to a
            // one-shot root container using the server's own image.
            containers::remove_host_dir_as_root(
                &self.docker,
                &cubelit.docker_image,
                &cubelit.volume_path,
            )
            .await;
            // Also remove Cubelit-managed server data (MariaDB data, txAdmin data)
            let server_data_dir = self.data_dir.join("servers").join(&cubelit.id);
            let _ = std::fs::remove_dir_all(&server_data_dir);
        }

        info!(server_id = %id, delete_data = %delete_data, "Server deleted");
        Ok(())
    }

    async fn update_server_settings(
        &self,
        id: &str,
        environment: HashMap<String, String>,
        events: Arc<dyn EventSink>,
    ) -> CoreResult<Cubelit> {
        info!(server_id = %id, "Updating server settings");
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let was_running = cubelit.status == "running" || cubelit.status == "starting";

        // Validate env vars before persisting
        validate_env_vars(&environment)?;

        // Stop and remove the existing container
        if let Some(ref container_id) = cubelit.container_id {
            let _ = containers::stop_container(&self.docker, container_id).await;
            let _ = containers::remove_container(&self.docker, container_id).await;
        }

        // Persist new environment
        let env_json = serde_json::to_string(&environment).unwrap_or_default();
        queries::update_cubelit_environment(&self.db, id, &env_json).await?;

        // Clear stale container_id (the old container was just removed) and
        // mark stopped while we recreate. `Some(None)` writes SQL NULL — the
        // previous `Some("")` left an empty string in the column, which broke
        // any subsequent code that treated `container_id` as `Option`.
        queries::update_cubelit_status(&self.db, id, "stopped", Some(None)).await?;

        // Re-read to get updated env
        let cubelit = queries::get_cubelit(&self.db, id).await?;

        // Load recipe for server_cmd, readiness config, and additional volume binds.
        let recipe = recipes::get_recipe(&self.recipes_dir, &cubelit.recipe_id).ok();
        let server_cmd = recipe.as_ref().and_then(|r| r.server_cmd.clone());
        let cap_add = recipe
            .as_ref()
            .map(|r| r.cap_add.clone())
            .unwrap_or_default();
        let mut extra_binds = self.extra_binds_for(&cubelit);
        if let Some(ref r) = recipe {
            extra_binds.extend(additional_volume_binds(&cubelit.volume_path, r));
        }
        let new_container_id =
            containers::create_container(&self.docker, &cubelit, &extra_binds, server_cmd, &cap_add)
                .await?;

        // Re-connect FiveM containers to their network
        if cubelit.recipe_id == "fivem" {
            let network_name = format!("cubelit-{}-net", id);
            let container_name = format!("cubelit-{}", id);
            self.docker
                .connect_network(
                    &network_name,
                    bollard::models::NetworkConnectRequest {
                        container: container_name,
                        endpoint_config: None,
                    },
                )
                .await?;
        }

        if was_running {
            // Also start sidecar if present
            if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
                let _ = containers::start_container(&self.docker, sidecar_id).await;
            }

            containers::start_container(&self.docker, &new_container_id).await?;

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let running =
                verify_container_status(&self.docker, &new_container_id).await == "running";

            if running {
                if let Some(ref r) = recipe.as_ref().and_then(|r| r.readiness.as_ref()).cloned() {
                    let pattern = r.log_pattern.clone();
                    let timeout = std::time::Duration::from_secs(r.timeout_secs);
                    queries::update_cubelit_status(
                        &self.db,
                        id,
                        "starting",
                        Some(Some(&new_container_id)),
                    )
                    .await?;
                    spawn_readiness_watcher(
                        self.docker.clone(),
                        self.db.clone(),
                        events.clone(),
                        id.to_string(),
                        new_container_id.clone(),
                        pattern,
                        timeout,
                    );
                } else {
                    queries::update_cubelit_status(
                        &self.db,
                        id,
                        "running",
                        Some(Some(&new_container_id)),
                    )
                    .await?;
                }
            } else {
                queries::update_cubelit_status(
                    &self.db,
                    id,
                    "error",
                    Some(Some(&new_container_id)),
                )
                .await?;
            }
        } else {
            queries::update_cubelit_status(
                &self.db,
                id,
                "stopped",
                Some(Some(&new_container_id)),
            )
            .await?;
        }

        let updated = queries::get_cubelit(&self.db, id).await?;
        info!(server_id = %id, container_id = %new_container_id, "Server settings updated");
        events.emit(CoreEvent::ServerStatusChanged {
            server_id: id.to_string(),
        });
        Ok(updated)
    }

    async fn rename_server(&self, id: &str, name: &str) -> CoreResult<Cubelit> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("Server name cannot be empty".into()));
        }
        queries::update_cubelit_name(&self.db, id, name).await?;
        queries::get_cubelit(&self.db, id).await
    }

    async fn sync_single(&self, id: &str) -> CoreResult<Cubelit> {
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        sync_single_server(&self.docker, &self.db, &cubelit).await?;
        queries::get_cubelit(&self.db, id).await
    }

    async fn sync_all(&self) -> CoreResult<Vec<Cubelit>> {
        sync_all_servers(&self.docker, &self.db).await
    }

    async fn list_servers(&self) -> CoreResult<Vec<Cubelit>> {
        queries::list_cubelits(&self.db).await
    }

    async fn get_server(&self, id: &str) -> CoreResult<Cubelit> {
        queries::get_cubelit(&self.db, id).await
    }

    async fn server_logs(&self, id: &str, lines: Option<u64>) -> CoreResult<Vec<String>> {
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let container_id = cubelit
            .container_id
            .ok_or_else(|| {
                CoreError::NotFound("No container associated with this server".into())
            })?;
        self.container_logs(&container_id, lines.unwrap_or(100)).await
    }

    async fn server_stats(&self, id: &str) -> CoreResult<ContainerStats> {
        let cubelit = queries::get_cubelit(&self.db, id).await?;
        let container_id = cubelit
            .container_id
            .ok_or_else(|| {
                CoreError::NotFound("No container associated with this server".into())
            })?;
        crate::docker::stats::get_container_stats(&self.docker, &container_id).await
    }

    async fn send_minecraft_command(&self, id: &str, command: &str) -> CoreResult<String> {
        minecraft::send_minecraft_command(&self.db, id, command).await
    }

    async fn send_server_command(&self, id: &str, command: &str) -> CoreResult<String> {
        super::console::send_server_command(&self.docker, &self.db, &self.recipes_dir, id, command)
            .await
    }

    async fn backup_server(&self, id: &str) -> CoreResult<String> {
        minecraft::backup_server(&self.db, id).await
    }
}

// ─── Volume helpers ───────────────────────────────────────────────────────────

/// Build bind strings for recipe volumes at index 1+ (the primary volume at
/// index 0 is already represented by `cubelit.volume_path`/`container_mount_path`).
///
/// Each additional volume is mapped to a subdirectory under `volume_path`
/// whose name is the last path segment of the container path:
///   `/opt/valheim`          → `{volume_path}/valheim:/opt/valheim`
///   `/project-zomboid-config` → `{volume_path}/project-zomboid-config:/project-zomboid-config`
pub fn additional_volume_binds(volume_path: &str, recipe: &recipes::Recipe) -> Vec<String> {
    recipe
        .volumes
        .iter()
        .skip(1)
        .map(|v| {
            let segment = additional_volume_subdir(&v.container_path);
            format!("{}/{}:{}", volume_path, segment, v.container_path)
        })
        .collect()
}

/// Returns `Some(path)` when CubeLit should remove `path` on create-server
/// failure, or `None` when the path must be left alone.
///
/// We only auto-remove when:
/// - the path was auto-generated by CubeLit (not supplied by the user), AND
/// - the directory did not exist before this create attempt (no pre-existing data).
fn cleanup_volume_path(volume_path: &str, user_provided: bool) -> Option<String> {
    if user_provided || std::path::Path::new(volume_path).exists() {
        None
    } else {
        Some(volume_path.to_string())
    }
}

fn additional_volume_subdir(container_path: &str) -> String {
    let segment = std::path::Path::new(container_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("data");
    let sanitized: String = segment
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "data".to_string()
    } else {
        sanitized
    }
}

/// Write the recipe's `seed_files` into a fresh volume, substituting
/// `{ENV_KEY}` tokens in both path and content from the resolved env map.
/// Best-effort: existing files are never overwritten, escaping paths are
/// skipped, and IO failures are logged rather than failing the create — a
/// missing seed degrades to the image's own first-boot behavior.
fn write_seed_files(
    recipe: &recipes::Recipe,
    volume_path: &std::path::Path,
    env: &HashMap<String, String>,
) {
    for seed in &recipe.seed_files {
        let rel = substitute_env_tokens(&seed.path, env);
        if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
            warn!(path = %rel, "Seed file path escapes the volume; skipping");
            continue;
        }
        let dest = volume_path.join(&rel);
        if dest.exists() {
            continue;
        }
        let write = || -> std::io::Result<()> {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, substitute_env_tokens(&seed.content, env))
        };
        if let Err(e) = write() {
            warn!(path = %dest.display(), error = %e, "Failed to write seed file");
        }
    }
}

/// Replace `{KEY}` tokens with values from the env map. Unknown tokens are
/// left as-is.
fn substitute_env_tokens(s: &str, env: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in env {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

// ─── Free helpers (used by lifecycle methods + the desktop crash watcher) ───

/// Reconcile a single server's DB row with Docker reality. Returns the
/// status string after any update. Public so the desktop `lib.rs` can call
/// it during startup before constructing the watcher tasks.
pub async fn sync_single_server(
    docker: &bollard::Docker,
    db: &SqlitePool,
    cubelit: &Cubelit,
) -> CoreResult<String> {
    let new_status = if let Some(container_id) = &cubelit.container_id {
        match docker.inspect_container(container_id, None).await {
            Ok(info) => {
                let running = info.state.and_then(|s| s.running).unwrap_or(false);
                if running {
                    // Preserve "starting" — the readiness watcher will promote it
                    // to "running" once the game server signals it is fully ready.
                    if cubelit.status == "starting" {
                        "starting"
                    } else {
                        "running"
                    }
                } else {
                    "stopped"
                }
            }
            Err(_) => "stopped",
        }
    } else {
        &cubelit.status
    };

    if new_status != cubelit.status {
        queries::update_cubelit_status(db, &cubelit.id, new_status, None).await?;
    }

    Ok(new_status.to_string())
}

/// Reconcile every server. Logs each correction so the cubelit.log file
/// captures unexpected stops.
pub async fn sync_all_servers(
    docker: &bollard::Docker,
    db: &SqlitePool,
) -> CoreResult<Vec<Cubelit>> {
    let cubelits = queries::list_cubelits(db).await?;
    info!("Syncing {} server(s) with Docker", cubelits.len());
    for cubelit in &cubelits {
        let old_status = &cubelit.status;
        let new_status = sync_single_server(docker, db, cubelit).await?;
        if new_status != *old_status {
            info!(
                server_id = %cubelit.id,
                name = %cubelit.name,
                old = %old_status,
                new = %new_status,
                "Server status corrected"
            );
        }
    }
    queries::list_cubelits(db).await
}

/// At startup, promote any server stuck in `"starting"` to `"running"` when
/// Docker confirms its container is actually up.
///
/// Readiness watchers are in-process tokio tasks — they die with the process.
/// If CubeLit restarts while a server is still in `"starting"` state, no
/// watcher is re-attached, so the status would be stuck forever. Call this
/// once at startup, **after** `sync_all_servers` and **before** spawning any
/// new readiness watchers, to clean up orphaned starting states.
///
/// Servers whose container is not running (or has no container) are left
/// unchanged; `sync_all_servers` already reconciled them to `"stopped"`.
pub async fn reconcile_orphaned_starting_servers(
    docker: &bollard::Docker,
    db: &SqlitePool,
) -> CoreResult<()> {
    let cubelits = queries::list_cubelits(db).await?;
    for cubelit in &cubelits {
        if cubelit.status != "starting" {
            continue;
        }
        let is_running = match &cubelit.container_id {
            Some(cid) => match docker.inspect_container(cid, None).await {
                Ok(info) => info.state.and_then(|s| s.running).unwrap_or(false),
                Err(_) => false,
            },
            None => false,
        };
        if is_running {
            queries::update_cubelit_status(db, &cubelit.id, "running", None).await?;
            tracing::warn!(
                server_id = %cubelit.id,
                name = %cubelit.name,
                "Server was in 'starting' with no active readiness watcher (process restart?); \
                promoted to 'running' — it may not yet be accepting connections",
            );
        }
    }
    Ok(())
}

// ─── Unit tests (no Docker required) ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Build a `LocalServerHost` backed by a real SQLite DB in a temp dir.
    /// `bollard::Docker::connect_with_local_defaults()` succeeds even without
    /// a daemon running — it only fails when an actual API call is made.
    /// Tests here only exercise DB-level methods so no Docker is needed.
    async fn test_host() -> LocalServerHost {
        let dir = tempdir().unwrap();
        let data_dir = dir.keep(); // prevent temp dir cleanup; PathBuf remains valid
        let recipes_dir = data_dir.join("recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();

        let db_url = format!("sqlite://{}?mode=rwc", data_dir.join("cubelit.db").display());
        let db = sqlx::SqlitePool::connect(&db_url).await.unwrap();
        sqlx::query("PRAGMA journal_mode=WAL").execute(&db).await.unwrap();
        crate::db::run_migrations(&db).await.unwrap();

        LocalServerHost {
            docker: bollard::Docker::connect_with_local_defaults().unwrap(),
            db,
            data_dir,
            recipes_dir,
        }
    }

    #[tokio::test]
    async fn list_servers_empty_db() {
        let host = test_host().await;
        let servers = host.list_servers().await.unwrap();
        assert!(servers.is_empty(), "expected no servers in a fresh DB");
    }

    #[tokio::test]
    async fn get_server_not_found() {
        let host = test_host().await;
        let err = host.get_server("nonexistent-id").await.unwrap_err();
        assert!(
            matches!(err, crate::error::CoreError::NotFound(_)),
            "expected NotFound, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn rename_server_not_found() {
        let host = test_host().await;
        let err = host.rename_server("nonexistent-id", "new-name").await.unwrap_err();
        assert!(
            matches!(err, crate::error::CoreError::NotFound(_)),
            "expected NotFound, got: {:?}",
            err
        );
    }

    fn starting_cubelit(id: &str) -> Cubelit {
        Cubelit {
            id: id.to_string(),
            name: "Test Server".to_string(),
            game: "Test Game".to_string(),
            recipe_id: "test".to_string(),
            docker_image: "test:1.0".to_string(),
            container_id: None,
            status: "starting".to_string(),
            port_mappings: "{}".to_string(),
            environment: "{}".to_string(),
            volume_path: "/tmp".to_string(),
            container_mount_path: "/data".to_string(),
            sidecar_container_id: None,
            sidecar_image: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn sync_single_preserves_starting_when_no_container() {
        // A server in "starting" with no container_id (e.g. created but container
        // not yet assigned) must not be demoted — no Docker call is made.
        let host = test_host().await;
        let cubelit = starting_cubelit("sync-nocontainer");
        queries::insert_cubelit(&host.db, &cubelit).await.unwrap();

        let status = sync_single_server(&host.docker, &host.db, &cubelit)
            .await
            .unwrap();

        assert_eq!(
            status, "starting",
            "no-container starting server must stay starting"
        );
        // DB row also unchanged
        let row = queries::get_cubelit(&host.db, "sync-nocontainer")
            .await
            .unwrap();
        assert_eq!(row.status, "starting");
    }

    #[tokio::test]
    async fn reconcile_orphaned_noop_when_no_container() {
        // With no container_id, Docker inspect is never attempted, so the
        // server stays in "starting" (no promotion).
        let host = test_host().await;
        let cubelit = starting_cubelit("orphan-nocontainer");
        queries::insert_cubelit(&host.db, &cubelit).await.unwrap();

        reconcile_orphaned_starting_servers(&host.docker, &host.db)
            .await
            .unwrap();

        let row = queries::get_cubelit(&host.db, "orphan-nocontainer")
            .await
            .unwrap();
        assert_eq!(
            row.status, "starting",
            "no-container server must not be promoted"
        );
    }

    #[tokio::test]
    async fn reconcile_orphaned_noop_for_non_starting_statuses() {
        // Only "starting" servers are candidates for promotion.
        let host = test_host().await;
        for (id, status) in [
            ("orphan-running", "running"),
            ("orphan-stopped", "stopped"),
            ("orphan-error", "error"),
        ] {
            let mut c = starting_cubelit(id);
            c.status = status.to_string();
            queries::insert_cubelit(&host.db, &c).await.unwrap();
        }

        reconcile_orphaned_starting_servers(&host.docker, &host.db)
            .await
            .unwrap();

        for (id, expected_status) in [
            ("orphan-running", "running"),
            ("orphan-stopped", "stopped"),
            ("orphan-error", "error"),
        ] {
            let row = queries::get_cubelit(&host.db, id).await.unwrap();
            assert_eq!(
                row.status, expected_status,
                "non-starting server '{}' must not be touched",
                id
            );
        }
    }

    #[test]
    fn fivem_mysql_connection_string_handles_empty_and_non_empty_passwords() {
        let container_name = "cubelit-test-db";

        assert_eq!(
            LocalServerHost::fivem_mysql_connection_string(container_name, ""),
            "mysql://root@cubelit-test-db:3306/fivem"
        );
        assert_eq!(
            LocalServerHost::fivem_mysql_connection_string(container_name, "test-secret"),
            "mysql://root:test-secret@cubelit-test-db:3306/fivem"
        );
    }

    fn make_recipe(container_paths: &[&str]) -> crate::recipes::Recipe {
        crate::recipes::Recipe {
            id: "test".into(),
            name: "Test".into(),
            description: "".into(),
            icon: "test".into(),
            docker_image: "test/image".into(),
            default_tag: "latest".into(),
            ports: vec![],
            environment: vec![],
            volumes: container_paths
                .iter()
                .map(|p| crate::recipes::RecipeVolume {
                    container_path: p.to_string(),
                    label: p.to_string(),
                })
                .collect(),
            config_files: vec![],
            mods: None,
            available: true,
            estimated_disk_mb: 0,
            tags: vec![],
            server_cmd: None,
            cap_add: vec![],
            readiness: None,
            dashboard: None,
            seed_files: vec![],
        }
    }

    #[test]
    fn additional_volume_binds_single_volume_returns_empty() {
        let recipe = make_recipe(&["/data"]);
        let binds = additional_volume_binds("/home/user/Cubelit/MyServer", &recipe);
        assert!(binds.is_empty());
    }

    #[test]
    fn additional_volume_binds_valheim_layout() {
        // Valheim: volumes[0]=/config (primary), volumes[1]=/opt/valheim (secondary)
        let recipe = make_recipe(&["/config", "/opt/valheim"]);
        let binds = additional_volume_binds("/home/user/Cubelit/MyServer", &recipe);
        assert_eq!(binds.len(), 1);
        assert_eq!(
            binds[0],
            "/home/user/Cubelit/MyServer/valheim:/opt/valheim"
        );
    }

    #[test]
    fn additional_volume_binds_project_zomboid_layout() {
        // Project Zomboid: volumes[0]=/project-zomboid, volumes[1]=/project-zomboid-config
        let recipe = make_recipe(&["/project-zomboid", "/project-zomboid-config"]);
        let binds = additional_volume_binds("/home/user/Cubelit/MyServer", &recipe);
        assert_eq!(binds.len(), 1);
        assert_eq!(
            binds[0],
            "/home/user/Cubelit/MyServer/project-zomboid-config:/project-zomboid-config"
        );
    }

    #[test]
    fn additional_volume_binds_three_volumes() {
        let recipe = make_recipe(&["/data", "/config", "/logs"]);
        let binds = additional_volume_binds("/srv/servers/test", &recipe);
        assert_eq!(binds.len(), 2);
        assert_eq!(binds[0], "/srv/servers/test/config:/config");
        assert_eq!(binds[1], "/srv/servers/test/logs:/logs");
    }

    #[test]
    fn additional_volume_binds_sanitizes_host_subdir_name() {
        let recipe = make_recipe(&["/data", "/path/with spaces"]);
        let binds = additional_volume_binds("/srv/servers/test", &recipe);
        assert_eq!(
            binds,
            vec!["/srv/servers/test/with_spaces:/path/with spaces"]
        );
    }

    // ─── Seed file tests ──────────────────────────────────────────────────────

    #[test]
    fn substitute_env_tokens_replaces_known_and_keeps_unknown() {
        let env = HashMap::from([
            ("SERVER_NAME".to_string(), "CubelitPZ".to_string()),
            ("RCON_PORT".to_string(), "27015".to_string()),
        ]);
        assert_eq!(
            substitute_env_tokens("Server/{SERVER_NAME}.ini", &env),
            "Server/CubelitPZ.ini"
        );
        assert_eq!(
            substitute_env_tokens("{UNKNOWN}/{RCON_PORT}", &env),
            "{UNKNOWN}/27015"
        );
    }

    #[test]
    fn write_seed_files_writes_templated_file_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let mut recipe = make_recipe(&[]);
        recipe.seed_files = vec![crate::recipes::RecipeSeedFile {
            path: "cfg/Server/{SERVER_NAME}.ini".into(),
            content: "RCONPassword=\nRCONPort={RCON_PORT}\n".into(),
        }];
        let env = HashMap::from([
            ("SERVER_NAME".to_string(), "MyPZ".to_string()),
            ("RCON_PORT".to_string(), "27099".to_string()),
        ]);

        write_seed_files(&recipe, dir.path(), &env);
        let dest = dir.path().join("cfg/Server/MyPZ.ini");
        assert_eq!(
            std::fs::read_to_string(&dest).unwrap(),
            "RCONPassword=\nRCONPort=27099\n"
        );

        // Second write must not clobber existing (user-modified) content.
        std::fs::write(&dest, "RCONPassword=usercustom\n").unwrap();
        write_seed_files(&recipe, dir.path(), &env);
        assert_eq!(
            std::fs::read_to_string(&dest).unwrap(),
            "RCONPassword=usercustom\n"
        );
    }

    #[test]
    fn write_seed_files_skips_escaping_paths() {
        let dir = tempfile::tempdir().unwrap();
        let mut recipe = make_recipe(&[]);
        recipe.seed_files = vec![
            crate::recipes::RecipeSeedFile {
                path: "../outside.ini".into(),
                content: "x".into(),
            },
            crate::recipes::RecipeSeedFile {
                path: "/absolute.ini".into(),
                content: "x".into(),
            },
        ];
        write_seed_files(&recipe, dir.path(), &HashMap::new());
        assert!(!dir.path().parent().unwrap().join("outside.ini").exists());
        assert!(!std::path::Path::new("/absolute.ini").exists());
    }

    // ─── CreateGuard / cleanup_volume_path tests ──────────────────────────────

    #[test]
    fn cleanup_volume_path_user_provided_is_never_removed() {
        assert_eq!(cleanup_volume_path("/user/provided/path", true), None);
    }

    #[test]
    fn cleanup_volume_path_auto_existing_dir_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        assert_eq!(
            cleanup_volume_path(path, false),
            None,
            "pre-existing directory must not be scheduled for removal"
        );
    }

    #[test]
    fn cleanup_volume_path_auto_new_dir_is_removed() {
        // Use a path that reliably doesn't exist; the function is pure so no fs side-effects.
        let path = "/tmp/__cubelit_test_nonexistent_volume_dir__";
        let _ = std::fs::remove_dir_all(path); // ensure clean state
        assert_eq!(
            cleanup_volume_path(path, false),
            Some(path.to_string()),
            "auto-generated, not-yet-existing path must be scheduled for removal"
        );
    }

    #[tokio::test]
    async fn create_guard_cleanup_deletes_db_row() {
        let host = test_host().await;
        let cubelit = starting_cubelit("guard-cleanup-db-test");
        queries::insert_cubelit(&host.db, &cubelit).await.unwrap();

        // Verify row exists before cleanup
        queries::get_cubelit(&host.db, "guard-cleanup-db-test")
            .await
            .unwrap();

        // Guard with only the DB row flagged — no Docker resource IDs, so no
        // Docker API calls are made during cleanup.
        let mut guard = CreateGuard::new("guard-cleanup-db-test".to_string(), None);
        guard.db_row_inserted = true;
        guard.cleanup(&host.docker, &host.db).await;

        let result = queries::get_cubelit(&host.db, "guard-cleanup-db-test").await;
        assert!(
            matches!(result, Err(crate::error::CoreError::NotFound(_))),
            "cleanup must delete the DB row"
        );
    }

    #[tokio::test]
    async fn create_guard_cleanup_skips_db_when_not_inserted() {
        let host = test_host().await;
        // db_row_inserted defaults to false — guard must not attempt a DELETE
        // for a row that was never inserted (which would silently succeed but
        // is wasteful and masks bugs).
        let guard = CreateGuard::new("nonexistent-guard-id".to_string(), None);
        guard.cleanup(&host.docker, &host.db).await; // must not panic
    }
}
