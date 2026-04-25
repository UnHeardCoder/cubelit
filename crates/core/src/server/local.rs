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
use tracing::{error, info};

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
    readiness_pattern, spawn_readiness_watcher, validate_env_vars, verify_container_status,
};

pub struct LocalServerHost {
    pub docker: bollard::Docker,
    pub db: SqlitePool,
    pub data_dir: PathBuf,
    pub recipes_dir: PathBuf,
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
        let network_config = bollard::network::CreateNetworkOptions {
            name: network_name.clone(),
            driver: "bridge".to_string(),
            ..Default::default()
        };
        let _ = self.docker.create_network(network_config).await;

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

        let db_config = bollard::container::Config {
            image: Some(mariadb_image.to_string()),
            env: Some(db_env),
            labels: Some(db_labels),
            host_config: Some(db_host_config),
            ..Default::default()
        };

        let db_create_opts = bollard::container::CreateContainerOptions {
            name: db_container_name.clone(),
            platform: None,
        };

        let db_response = self
            .docker
            .create_container(Some(db_create_opts), db_config)
            .await?;
        let sidecar_id = db_response.id;

        // Connect MariaDB container to the network
        let _ = self
            .docker
            .connect_network(
                &network_name,
                bollard::network::ConnectNetworkOptions {
                    container: db_container_name.clone(),
                    ..Default::default()
                },
            )
            .await;

        // Start MariaDB
        containers::start_container(&self.docker, &sidecar_id).await?;

        // Update cubelit with sidecar info
        queries::update_cubelit_sidecar(&self.db, id, &sidecar_id, mariadb_image).await?;

        // Add MySQL connection string to FiveM env (root user, password may be empty)
        let conn_str = if db_password.is_empty() {
            format!("mysql://root@{}:3306/fivem", db_container_name)
        } else {
            format!(
                "mysql://root:{}@{}:3306/fivem",
                db_password, db_container_name
            )
        };
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
    ) -> CoreResult<String> {
        containers::create_container(&self.docker, cubelit, extra_binds).await
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
        use bollard::container::LogsOptions;
        use futures_util::StreamExt;

        let opts = LogsOptions::<String> {
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

        // Use user-provided volume path or default to ~/Cubelit/{sanitized-name}.
        // For FiveM: the spritsail/fivem image only copies its default resources when /config is
        // empty on first boot. If the default path already has content (e.g. from a previously
        // deleted server with the same name whose files were kept), fall back to a unique path
        // using the server ID so the image always starts with an empty volume.
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
        std::fs::create_dir_all(&volume_path)?;

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

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "pulling".into(),
            progress: Some(0.2),
            message: format!("Pulling image {}...", image),
        }));

        images::pull_image(&self.docker, &image, events.as_ref()).await?;

        // FiveM sidecar: MariaDB + Docker network
        if cubelit.recipe_id == "fivem" {
            self.provision_fivem_sidecar(&id, &mut env, events.as_ref())
                .await?;
        }

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "creating".into(),
            progress: Some(0.7),
            message: "Creating container...".into(),
        }));

        // Re-read cubelit from DB to get updated env (with sidecar connection string)
        let cubelit = queries::get_cubelit(&self.db, &id).await?;
        let extra_binds: Vec<String> = self.extra_binds_for(&cubelit);
        let container_id =
            containers::create_container(&self.docker, &cubelit, &extra_binds).await?;

        // If FiveM, connect the main container to the network too
        if cubelit.recipe_id == "fivem" {
            let network_name = format!("cubelit-{}-net", id);
            let container_name = format!("cubelit-{}", id);
            let _ = self
                .docker
                .connect_network(
                    &network_name,
                    bollard::network::ConnectNetworkOptions {
                        container: container_name,
                        ..Default::default()
                    },
                )
                .await;
        }

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "starting".into(),
            progress: Some(0.9),
            message: "Starting server...".into(),
        }));

        containers::start_container(&self.docker, &container_id).await?;

        // Post-start verification: wait 2s then check if container is actually running
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
            // Game needs log-based readiness detection — keep "starting" until Done
            queries::update_cubelit_status(&self.db, &id, "starting", Some(&container_id)).await?;
            spawn_readiness_watcher(
                self.docker.clone(),
                self.db.clone(),
                events.clone(),
                id.clone(),
                container_id.clone(),
                pattern,
            );
        } else {
            let status = if running { "running" } else { "error" };
            queries::update_cubelit_status(&self.db, &id, status, Some(&container_id)).await?;
        }

        let updated = queries::get_cubelit(&self.db, &id).await?;

        events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "ready".into(),
            progress: Some(1.0),
            message: if running {
                "Server is ready!".into()
            } else {
                "Server started but may have encountered an error.".into()
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

        // Also start sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::start_container(&self.docker, sidecar_id).await;
        }

        containers::start_container(&self.docker, &container_id).await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
            queries::update_cubelit_status(&self.db, id, "starting", None).await?;
            spawn_readiness_watcher(
                self.docker.clone(),
                self.db.clone(),
                events,
                id.to_string(),
                container_id.clone(),
                pattern,
            );
        } else {
            let status = if running { "running" } else { "error" };
            queries::update_cubelit_status(&self.db, id, status, None).await?;
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

        // Also restart sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = containers::restart_container(&self.docker, sidecar_id).await;
        }

        containers::restart_container(&self.docker, &container_id).await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&self.docker, &container_id).await == "running";

        if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
            queries::update_cubelit_status(&self.db, id, "starting", None).await?;
            spawn_readiness_watcher(
                self.docker.clone(),
                self.db.clone(),
                events,
                id.to_string(),
                container_id.clone(),
                pattern,
            );
        } else {
            let status = if running { "running" } else { "error" };
            queries::update_cubelit_status(&self.db, id, status, None).await?;
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
            let _ = std::fs::remove_dir_all(&cubelit.volume_path);
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

        // Clear stale container_id and mark stopped while we recreate
        queries::update_cubelit_status(&self.db, id, "stopped", Some("")).await?;

        // Re-read to get updated env
        let cubelit = queries::get_cubelit(&self.db, id).await?;

        let extra_binds: Vec<String> = self.extra_binds_for(&cubelit);
        let new_container_id =
            containers::create_container(&self.docker, &cubelit, &extra_binds).await?;

        // Re-connect FiveM containers to their network
        if cubelit.recipe_id == "fivem" {
            let network_name = format!("cubelit-{}-net", id);
            let container_name = format!("cubelit-{}", id);
            let _ = self
                .docker
                .connect_network(
                    &network_name,
                    bollard::network::ConnectNetworkOptions {
                        container: container_name,
                        ..Default::default()
                    },
                )
                .await;
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

            if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
                queries::update_cubelit_status(
                    &self.db,
                    id,
                    "starting",
                    Some(&new_container_id),
                )
                .await?;
                spawn_readiness_watcher(
                    self.docker.clone(),
                    self.db.clone(),
                    events.clone(),
                    id.to_string(),
                    new_container_id.clone(),
                    pattern,
                );
            } else {
                let status = if running { "running" } else { "error" };
                queries::update_cubelit_status(
                    &self.db,
                    id,
                    status,
                    Some(&new_container_id),
                )
                .await?;
            }
        } else {
            queries::update_cubelit_status(
                &self.db,
                id,
                "stopped",
                Some(&new_container_id),
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

    async fn backup_server(&self, id: &str) -> CoreResult<String> {
        minecraft::backup_server(&self.db, id).await
    }
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
