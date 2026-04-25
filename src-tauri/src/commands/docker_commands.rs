use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tracing::{error, info};

use cubelit_core::events::{CoreEvent, EventSink, ServerCreateProgress};

use crate::db::models::Cubelit;
use crate::docker;
use crate::error::CoreError;
use crate::event_sink::TauriEventSink;
use crate::state::AppState;

fn validate_env_vars(env: &HashMap<String, String>) -> Result<(), CoreError> {
    for (key, value) in env {
        if key.contains('\0') {
            return Err(CoreError::Validation(format!("Env var key '{}' contains NUL byte", key)));
        }
        if value.contains('\0') {
            return Err(CoreError::Validation(format!("Env var '{}' value contains NUL byte", key)));
        }
        if value.len() > 4096 {
            return Err(CoreError::Validation(format!("Env var '{}' value exceeds 4096 bytes", key)));
        }
    }
    Ok(())
}

async fn verify_container_status(docker: &bollard::Docker, container_id: &str) -> &'static str {
    match docker.inspect_container(container_id, None).await {
        Ok(info) => {
            let running = info.state.and_then(|s| s.running).unwrap_or(false);
            if running { "running" } else { "error" }
        }
        Err(_) => "error",
    }
}

/// Returns the log pattern that signals a game server is fully ready, if any.
/// None means the container being "running" is sufficient — no log watching needed.
///
/// The pattern MUST be unique to the actual game-server "ready" signal, not to
/// intermediate setup steps.  For Minecraft, the itzg entrypoint scripts also
/// print lines containing "Done" (e.g. "Done downloading pack"), so we use the
/// full suffix that only the Minecraft server itself emits:
///   `Done (11.117s)! For help, type "help"`
fn readiness_pattern(recipe_id: &str) -> Option<&'static str> {
    match recipe_id {
        // This exact phrase appears on the SAME line as "Done (X.Xs)!" and is
        // emitted by net.minecraft.server.dedicated.DedicatedServer across all
        // versions and mod loaders (Vanilla, Forge, NeoForge, Fabric, FTB…).
        // It is never printed by the itzg setup scripts or by mod init code.
        "minecraft-java" => Some(r#"! For help, type "help""#),
        _ => None,
    }
}

/// Spawns a background task that tails container logs and promotes the server
/// status from "starting" → "running" once the readiness `pattern` is found.
/// Times out after 10 minutes (promotes anyway) to avoid hanging forever.
fn spawn_readiness_watcher(
    docker: bollard::Docker,
    pool: sqlx::SqlitePool,
    events: Arc<dyn EventSink>,
    server_id: String,
    container_id: String,
    pattern: &'static str,
) {
    tokio::spawn(async move {
        use futures_util::StreamExt;
        use std::time::SystemTime;

        // Only fetch logs produced from ~10 seconds before we start watching
        // (catches any fast startup lines) while avoiding old "Done" lines from
        // a previous run.
        let since = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - 10)
            .max(0);

        let opts = bollard::container::LogsOptions::<String> {
            stdout: true,
            stderr: true,
            follow: true,
            since,
            ..Default::default()
        };

        let mut stream = docker.logs(&container_id, Some(opts));
        let timeout = tokio::time::sleep(std::time::Duration::from_secs(600));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                biased;
                _ = &mut timeout => {
                    // 10-minute limit reached — promote anyway
                    let _ = crate::db::queries::update_cubelit_status(
                        &pool, &server_id, "running", None,
                    ).await;
                    events.emit(CoreEvent::ServerStatusChanged {
                        server_id: server_id.clone(),
                    });
                    break;
                }
                item = stream.next() => {
                    match item {
                        Some(Ok(log)) => {
                            if log.to_string().contains(pattern) {
                                let _ = crate::db::queries::update_cubelit_status(
                                    &pool, &server_id, "running", None,
                                ).await;
                                events.emit(CoreEvent::ServerStatusChanged {
                                    server_id: server_id.clone(),
                                });
                                break;
                            }
                        }
                        // Stream ended (container stopped) or error — exit silently;
                        // sync_single_server will correct the status on the next poll.
                        Some(Err(_)) | None => break,
                    }
                }
            }
        }
    });
}

pub async fn sync_single_server(
    docker: &bollard::Docker,
    db: &sqlx::SqlitePool,
    cubelit: &Cubelit,
) -> Result<String, CoreError> {
    let new_status = if let Some(container_id) = &cubelit.container_id {
        match docker.inspect_container(container_id, None).await {
            Ok(info) => {
                let running = info.state.and_then(|s| s.running).unwrap_or(false);
                if running {
                    // Preserve "starting" — the readiness watcher will promote it
                    // to "running" once the game server signals it is fully ready.
                    if cubelit.status == "starting" { "starting" } else { "running" }
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
        crate::db::queries::update_cubelit_status(db, &cubelit.id, new_status, None).await?;
    }

    Ok(new_status.to_string())
}

pub async fn sync_all_servers(
    docker: &bollard::Docker,
    db: &sqlx::SqlitePool,
) -> Result<Vec<Cubelit>, CoreError> {
    let cubelits = crate::db::queries::list_cubelits(db).await?;
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
    crate::db::queries::list_cubelits(db).await
}

#[derive(Debug, Deserialize)]
pub struct CreateServerConfig {
    pub name: String,
    pub recipe_id: String,
    pub port_overrides: Option<HashMap<String, u16>>,
    pub env_overrides: Option<HashMap<String, String>>,
    pub volume_path: Option<String>,
    /// Override the recipe's default_tag (e.g. "java17", "java8", "latest")
    pub tag_override: Option<String>,
}

#[tauri::command]
pub async fn check_docker_status(
    state: State<'_, AppState>,
) -> Result<docker::health::DockerStatus, CoreError> {
    Ok(docker::health::check_docker_status(&state.docker).await)
}

#[tauri::command]
pub async fn create_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    config: CreateServerConfig,
) -> Result<Cubelit, CoreError> {
    info!(name = %config.name, recipe = %config.recipe_id, "Creating server");
    let recipe = cubelit_core::recipes::get_recipe(&state.recipes_dir, &config.recipe_id)?;
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle.clone());

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
        let home = dirs::home_dir().unwrap_or_else(|| state.data_dir.clone());
        let sanitized = config.name.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "");
        let base_path = home.join("Cubelit").join(&sanitized);
        if config.recipe_id == "fivem"
            && base_path.exists()
            && base_path.read_dir().is_ok_and(|mut d| d.next().is_some())
        {
            home.join("Cubelit").join(format!("{}-{}", sanitized, &id[..8]))
                .to_string_lossy().to_string()
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

    let tag = config.tag_override.as_deref().unwrap_or(&recipe.default_tag);
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

    crate::db::queries::insert_cubelit(&state.db, &cubelit).await?;

    events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
        step: "pulling".into(),
        progress: Some(0.2),
        message: format!("Pulling image {}...", image),
    }));

    docker::images::pull_image(&state.docker, &image, events.as_ref()).await?;

    // FiveM sidecar: MariaDB + Docker network
    if cubelit.recipe_id == "fivem" {
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
        docker::images::pull_image(&state.docker, mariadb_image, events.as_ref()).await?;

        // Create Docker network
        let network_name = format!("cubelit-{}-net", id);
        let network_config = bollard::network::CreateNetworkOptions {
            name: network_name.clone(),
            driver: "bridge".to_string(),
            ..Default::default()
        };
        let _ = state.docker.create_network(network_config).await;

        // Create MariaDB data directory
        let db_data_dir = state.data_dir.join("servers").join(&id).join("db");
        std::fs::create_dir_all(&db_data_dir)?;

        // Create MariaDB container
        let db_container_name = format!("cubelit-{}-db", id);
        let mut db_labels = HashMap::new();
        db_labels.insert("cubelit.id".to_string(), id.clone());
        db_labels.insert("cubelit.role".to_string(), "database".to_string());
        db_labels.insert("cubelit.managed".to_string(), "true".to_string());

        let mut db_env = vec!["MYSQL_DATABASE=fivem".to_string()];
        if db_password.is_empty() {
            db_env.push("MARIADB_ALLOW_EMPTY_ROOT_PASSWORD=1".to_string());
        } else {
            db_env.push(format!("MARIADB_ROOT_PASSWORD={}", db_password));
        }

        let mut db_port_bindings = std::collections::HashMap::new();
        db_port_bindings.insert(
            "3306/tcp".to_string(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("127.0.0.1".to_string()),
                host_port: Some(db_host_port.to_string()),
            }]),
        );

        let db_host_config = bollard::models::HostConfig {
            binds: Some(vec![format!("{}:/var/lib/mysql", db_data_dir.to_string_lossy())]),
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

        let db_response = state.docker.create_container(Some(db_create_opts), db_config).await?;
        let sidecar_id = db_response.id;

        // Connect MariaDB container to the network
        let _ = state.docker.connect_network(
            &network_name,
            bollard::network::ConnectNetworkOptions {
                container: db_container_name.clone(),
                ..Default::default()
            },
        ).await;

        // Start MariaDB
        docker::containers::start_container(&state.docker, &sidecar_id).await?;

        // Update cubelit with sidecar info
        crate::db::queries::update_cubelit_sidecar(
            &state.db, &id, &sidecar_id, mariadb_image,
        ).await?;

        // Add MySQL connection string to FiveM env (root user, password may be empty)
        let conn_str = if db_password.is_empty() {
            format!("mysql://root@{}:3306/fivem", db_container_name)
        } else {
            format!("mysql://root:{}@{}:3306/fivem", db_password, db_container_name)
        };
        env.insert("MYSQL_CONNECTION_STRING".to_string(), conn_str);

        // txAdmin mode: skip server.cfg and let txAdmin manage the server via its web UI.
        // This MUST be set, and LICENSE_KEY MUST NOT be passed to the container — the
        // spritsail/fivem entrypoint exits with error if both are present.
        env.insert("NO_DEFAULT_CONFIG".to_string(), "1".to_string());

        // Create txAdmin data directory (mounted separately at /txData inside the container)
        let txdata_dir = state.data_dir.join("servers").join(&id).join("txdata");
        std::fs::create_dir_all(&txdata_dir)?;

        // Update the cubelit's environment in DB
        let updated_env = serde_json::to_string(&env).unwrap_or_default();
        crate::db::queries::update_cubelit_environment(&state.db, &id, &updated_env).await?;
    }

    events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
        step: "creating".into(),
        progress: Some(0.7),
        message: "Creating container...".into(),
    }));

    // Re-read cubelit from DB to get updated env (with sidecar connection string)
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    // FiveM: also bind /txData so txAdmin persists its config across container restarts
    let extra_binds: Vec<String> = if cubelit.recipe_id == "fivem" {
        let txdata_dir = state.data_dir.join("servers").join(&cubelit.id).join("txdata");
        vec![format!("{}:/txData", txdata_dir.to_string_lossy())]
    } else {
        vec![]
    };
    let container_id = docker::containers::create_container(&state.docker, &cubelit, &extra_binds).await?;

    // If FiveM, connect the main container to the network too
    if cubelit.recipe_id == "fivem" {
        let network_name = format!("cubelit-{}-net", id);
        let container_name = format!("cubelit-{}", id);
        let _ = state.docker.connect_network(
            &network_name,
            bollard::network::ConnectNetworkOptions {
                container: container_name,
                ..Default::default()
            },
        ).await;
    }

    events.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
        step: "starting".into(),
        progress: Some(0.9),
        message: "Starting server...".into(),
    }));

    docker::containers::start_container(&state.docker, &container_id).await?;

    // Post-start verification: wait 2s then check if container is actually running
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let running = verify_container_status(&state.docker, &container_id).await == "running";

    if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
        // Game needs log-based readiness detection — keep "starting" until Done
        crate::db::queries::update_cubelit_status(
            &state.db, &id, "starting", Some(&container_id),
        ).await?;
        spawn_readiness_watcher(
            state.docker.clone(),
            state.db.clone(),
            events.clone(),
            id.clone(),
            container_id.clone(),
            pattern,
        );
    } else {
        let status = if running { "running" } else { "error" };
        crate::db::queries::update_cubelit_status(
            &state.db, &id, status, Some(&container_id),
        ).await?;
    }

    let updated = crate::db::queries::get_cubelit(&state.db, &id).await?;

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
        error!(server_id = %id, container_id = %container_id, "Server created but container did not start");
    }

    Ok(updated)
}

#[tauri::command]
pub async fn start_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), CoreError> {
    info!(server_id = %id, "Starting server");
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let container_id = cubelit
        .container_id
        .ok_or_else(|| CoreError::NotFound("No container associated with this server".into()))?;

    // Also start sidecar if present
    if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
        let _ = docker::containers::start_container(&state.docker, sidecar_id).await;
    }

    docker::containers::start_container(&state.docker, &container_id).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let running = verify_container_status(&state.docker, &container_id).await == "running";

    if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
        crate::db::queries::update_cubelit_status(&state.db, &id, "starting", None).await?;
        spawn_readiness_watcher(
            state.docker.clone(),
            state.db.clone(),
            TauriEventSink::shared(app_handle),
            id.clone(),
            container_id.clone(),
            pattern,
        );
    } else {
        let status = if running { "running" } else { "error" };
        crate::db::queries::update_cubelit_status(&state.db, &id, status, None).await?;
    }

    if running {
        info!(server_id = %id, container_id = %container_id, "Server started");
    } else {
        error!(server_id = %id, container_id = %container_id, "Server started but container did not come up");
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CoreError> {
    info!(server_id = %id, "Stopping server");
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let container_id = cubelit
        .container_id
        .ok_or_else(|| CoreError::NotFound("No container associated with this server".into()))?;

    docker::containers::stop_container(&state.docker, &container_id).await?;

    // Also stop sidecar if present
    if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
        let _ = docker::containers::stop_container(&state.docker, sidecar_id).await;
    }

    crate::db::queries::update_cubelit_status(&state.db, &id, "stopped", None).await?;
    info!(server_id = %id, "Server stopped");

    Ok(())
}

#[tauri::command]
pub async fn restart_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), CoreError> {
    info!(server_id = %id, "Restarting server");
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let container_id = cubelit
        .container_id
        .ok_or_else(|| CoreError::NotFound("No container associated with this server".into()))?;

    // Also restart sidecar if present
    if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
        let _ = docker::containers::restart_container(&state.docker, sidecar_id).await;
    }

    docker::containers::restart_container(&state.docker, &container_id).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let running = verify_container_status(&state.docker, &container_id).await == "running";

    if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
        crate::db::queries::update_cubelit_status(&state.db, &id, "starting", None).await?;
        spawn_readiness_watcher(
            state.docker.clone(),
            state.db.clone(),
            TauriEventSink::shared(app_handle),
            id.clone(),
            container_id.clone(),
            pattern,
        );
    } else {
        let status = if running { "running" } else { "error" };
        crate::db::queries::update_cubelit_status(&state.db, &id, status, None).await?;
    }

    if running {
        info!(server_id = %id, container_id = %container_id, "Server restarted");
    } else {
        error!(server_id = %id, container_id = %container_id, "Server restarted but container did not come back up");
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_server(
    state: State<'_, AppState>,
    id: String,
    delete_data: bool,
) -> Result<(), CoreError> {
    info!(server_id = %id, delete_data = %delete_data, "Deleting server");
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;

    if let Some(container_id) = &cubelit.container_id {
        let _ = docker::containers::stop_container(&state.docker, container_id).await;
        let _ = docker::containers::remove_container(&state.docker, container_id).await;
    }

    // Remove sidecar container if present
    if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
        let _ = docker::containers::stop_container(&state.docker, sidecar_id).await;
        let _ = docker::containers::remove_container(&state.docker, sidecar_id).await;
    }

    // Remove Docker network if it was a FiveM server
    if cubelit.sidecar_container_id.is_some() {
        let network_name = format!("cubelit-{}-net", cubelit.id);
        let _ = state.docker.remove_network(&network_name).await;
    }

    crate::db::queries::delete_cubelit(&state.db, &id).await?;

    if delete_data {
        let _ = std::fs::remove_dir_all(&cubelit.volume_path);
        // Also remove Cubelit-managed server data (MariaDB data, txAdmin data)
        let server_data_dir = state.data_dir.join("servers").join(&cubelit.id);
        let _ = std::fs::remove_dir_all(&server_data_dir);
    }

    info!(server_id = %id, delete_data = %delete_data, "Server deleted");
    Ok(())
}

#[tauri::command]
pub async fn sync_server_status(
    state: State<'_, AppState>,
    id: String,
) -> Result<Cubelit, CoreError> {
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    sync_single_server(&state.docker, &state.db, &cubelit).await?;
    crate::db::queries::get_cubelit(&state.db, &id).await
}

#[tauri::command]
pub async fn sync_all_statuses(
    state: State<'_, AppState>,
) -> Result<Vec<Cubelit>, CoreError> {
    sync_all_servers(&state.docker, &state.db).await
}

/// Apply new environment variables to an existing server.
/// Stops the old container, removes it, recreates with updated env, and starts
/// it again if the server was previously running.
#[tauri::command]
pub async fn update_server_settings(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
    environment: HashMap<String, String>,
) -> Result<Cubelit, CoreError> {
    info!(server_id = %id, "Updating server settings");
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle.clone());
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let was_running = cubelit.status == "running" || cubelit.status == "starting";

    // Validate env vars before persisting
    validate_env_vars(&environment)?;

    // Stop and remove the existing container
    if let Some(ref container_id) = cubelit.container_id {
        let _ = docker::containers::stop_container(&state.docker, container_id).await;
        let _ = docker::containers::remove_container(&state.docker, container_id).await;
    }

    // Persist new environment
    let env_json = serde_json::to_string(&environment).unwrap_or_default();
    crate::db::queries::update_cubelit_environment(&state.db, &id, &env_json).await?;

    // Clear stale container_id and mark stopped while we recreate
    crate::db::queries::update_cubelit_status(&state.db, &id, "stopped", Some("")).await?;

    // Re-read to get updated env
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;

    // FiveM requires an extra txData bind
    let extra_binds: Vec<String> = if cubelit.recipe_id == "fivem" {
        let txdata_dir = state.data_dir.join("servers").join(&cubelit.id).join("txdata");
        vec![format!("{}:/txData", txdata_dir.to_string_lossy())]
    } else {
        vec![]
    };

    let new_container_id = docker::containers::create_container(&state.docker, &cubelit, &extra_binds).await?;

    // Re-connect FiveM containers to their network
    if cubelit.recipe_id == "fivem" {
        let network_name = format!("cubelit-{}-net", id);
        let container_name = format!("cubelit-{}", id);
        let _ = state.docker.connect_network(
            &network_name,
            bollard::network::ConnectNetworkOptions {
                container: container_name,
                ..Default::default()
            },
        ).await;
    }

    if was_running {
        // Also start sidecar if present
        if let Some(ref sidecar_id) = cubelit.sidecar_container_id {
            let _ = docker::containers::start_container(&state.docker, sidecar_id).await;
        }

        docker::containers::start_container(&state.docker, &new_container_id).await?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let running = verify_container_status(&state.docker, &new_container_id).await == "running";

        if let Some(pattern) = readiness_pattern(&cubelit.recipe_id).filter(|_| running) {
            crate::db::queries::update_cubelit_status(
                &state.db, &id, "starting", Some(&new_container_id),
            ).await?;
            spawn_readiness_watcher(
                state.docker.clone(),
                state.db.clone(),
                events.clone(),
                id.clone(),
                new_container_id.clone(),
                pattern,
            );
        } else {
            let status = if running { "running" } else { "error" };
            crate::db::queries::update_cubelit_status(
                &state.db, &id, status, Some(&new_container_id),
            ).await?;
        }
    } else {
        crate::db::queries::update_cubelit_status(
            &state.db, &id, "stopped", Some(&new_container_id),
        ).await?;
    }

    let updated = crate::db::queries::get_cubelit(&state.db, &id).await?;
    info!(server_id = %id, container_id = %new_container_id, "Server settings updated");
    events.emit(CoreEvent::ServerStatusChanged {
        server_id: id.clone(),
    });
    Ok(updated)
}

#[tauri::command]
pub async fn get_server_stats(
    state: State<'_, AppState>,
    id: String,
) -> Result<crate::docker::stats::ContainerStats, CoreError> {
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let container_id = cubelit
        .container_id
        .ok_or_else(|| CoreError::NotFound("No container associated with this server".into()))?;

    crate::docker::stats::get_container_stats(&state.docker, &container_id).await
}

#[tauri::command]
pub async fn get_server_logs(
    id: String,
    lines: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, CoreError> {
    use bollard::container::LogsOptions;
    use futures_util::StreamExt;

    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let container_id = cubelit
        .container_id
        .ok_or_else(|| CoreError::NotFound("No container associated with this server".into()))?;

    let tail = lines.unwrap_or(100).to_string();
    let opts = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail,
        ..Default::default()
    };

    let mut stream = state.docker.logs(&container_id, Some(opts));
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

/// Spawns a background task that subscribes to Docker events and emits
/// `server-status-changed` when a primary managed container stops or dies.
pub fn spawn_crash_watcher(
    docker: bollard::Docker,
    pool: sqlx::SqlitePool,
    events: Arc<dyn EventSink>,
) {
    tokio::spawn(async move {
        use futures_util::StreamExt;
        loop {
            let mut filters = std::collections::HashMap::new();
            filters.insert("type".to_string(), vec!["container".to_string()]);
            filters.insert("event".to_string(), vec!["die".to_string(), "stop".to_string()]);
            filters.insert("label".to_string(), vec!["cubelit.role=primary".to_string()]);

            let opts = bollard::system::EventsOptions::<String> {
                filters,
                ..Default::default()
            };

            let mut stream = docker.events(Some(opts));

            while let Some(Ok(ev)) = stream.next().await {
                if let Some(actor) = ev.actor {
                    if let Some(attrs) = actor.attributes {
                        if let Some(server_id) = attrs.get("cubelit.id").map(String::as_str) {
                            let _ = crate::db::queries::update_cubelit_status(
                                &pool, server_id, "stopped", None,
                            ).await;
                            events.emit(CoreEvent::ServerStatusChanged {
                                server_id: server_id.to_string(),
                            });
                        }
                    }
                }
            }

            // Stream ended (Docker restarted or disconnected) — wait before reconnecting
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn valid_env_passes() {
        assert!(validate_env_vars(&env(&[("KEY", "value"), ("PORT", "25565")])).is_ok());
    }

    #[test]
    fn nul_in_key_rejected() {
        let err = validate_env_vars(&env(&[("KE\0Y", "value")])).unwrap_err();
        assert!(err.to_string().contains("NUL byte"));
    }

    #[test]
    fn nul_in_value_rejected() {
        let err = validate_env_vars(&env(&[("KEY", "val\0ue")])).unwrap_err();
        assert!(err.to_string().contains("NUL byte"));
    }

    #[test]
    fn value_too_long_rejected() {
        let long = "x".repeat(4097);
        let err = validate_env_vars(&env(&[("KEY", &long)])).unwrap_err();
        assert!(err.to_string().contains("4096"));
    }

    #[test]
    fn value_exactly_4096_passes() {
        let max = "x".repeat(4096);
        assert!(validate_env_vars(&env(&[("KEY", &max)])).is_ok());
    }

    #[test]
    fn empty_env_passes() {
        assert!(validate_env_vars(&env(&[])).is_ok());
    }

    #[test]
    fn readiness_pattern_minecraft_java() {
        assert_eq!(readiness_pattern("minecraft-java"), Some(r#"! For help, type "help""#));
    }

    #[test]
    fn readiness_pattern_unknown_returns_none() {
        assert_eq!(readiness_pattern("valheim"), None);
        assert_eq!(readiness_pattern("fivem"), None);
        assert_eq!(readiness_pattern(""), None);
    }
}
