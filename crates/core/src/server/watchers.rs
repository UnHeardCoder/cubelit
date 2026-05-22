//! Background watchers + small helpers used by the server lifecycle.
//!
//! These functions exist as free functions (not trait methods) for two
//! reasons:
//!   1. `validate_env_vars` and `readiness_pattern` are pure — they don't
//!      need a `&self`, and unit-testing them stand-alone is trivial.
//!   2. The watchers are background tokio tasks that own clones of
//!      `Docker` + `SqlitePool`. Putting them on a trait would force
//!      implementors to spawn tasks even when they don't want to.

use std::collections::HashMap;
use std::sync::Arc;

use crate::db::queries;
use crate::error::CoreError;
use crate::events::{CoreEvent, EventSink};

/// Reject env vars that would break Docker, leak into logs, or cause panics
/// downstream. Returns the same `CoreError::Validation` strings the desktop
/// frontend has been displaying since v0.1.7 — do not change wording without
/// also updating the user-facing error toasts.
pub fn validate_env_vars(env: &HashMap<String, String>) -> Result<(), CoreError> {
    for (key, value) in env {
        if key.contains('\0') {
            return Err(CoreError::Validation(format!(
                "Env var key '{}' contains NUL byte",
                key
            )));
        }
        if value.contains('\0') {
            return Err(CoreError::Validation(format!(
                "Env var '{}' value contains NUL byte",
                key
            )));
        }
        if value.len() > 4096 {
            return Err(CoreError::Validation(format!(
                "Env var '{}' value exceeds 4096 bytes",
                key
            )));
        }
    }
    Ok(())
}

/// Inspect a container and return either `"running"` or `"error"`.
/// Used right after `start` / `restart` to confirm the container actually
/// came up rather than crashing immediately.
pub async fn verify_container_status(
    docker: &bollard::Docker,
    container_id: &str,
) -> &'static str {
    match docker.inspect_container(container_id, None).await {
        Ok(info) => {
            let running = info.state.and_then(|s| s.running).unwrap_or(false);
            if running {
                "running"
            } else {
                "error"
            }
        }
        Err(e) => {
            tracing::warn!(
                container_id = %container_id,
                error = %e,
                "verify_container_status: docker inspect failed; reporting error"
            );
            "error"
        }
    }
}

/// Spawns a background task that tails container logs and promotes the server
/// status from `"starting"` → `"running"` once `pattern` appears in the output.
///
/// The pattern and timeout come from `Recipe::readiness` so each game can
/// declare its own "I am joinable" signal in recipe JSON rather than having
/// game-specific logic hardcoded here.
///
/// On timeout the watcher re-inspects the container state before writing DB —
/// a server that crashed mid-startup is reported as `"error"`, not `"running"`.
pub fn spawn_readiness_watcher(
    docker: bollard::Docker,
    pool: sqlx::SqlitePool,
    events: Arc<dyn EventSink>,
    server_id: String,
    container_id: String,
    pattern: String,
    timeout: std::time::Duration,
) {
    tokio::spawn(async move {
        use futures_util::StreamExt;
        use std::time::SystemTime;

        // Only fetch logs produced from ~10 seconds before we start watching
        // (catches any fast startup lines) while avoiding stale lines from a
        // previous run of the same container.
        let since = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - 10)
            .max(0);

        let opts = bollard::query_parameters::LogsOptions {
            stdout: true,
            stderr: true,
            follow: true,
            since: since as i32,
            ..Default::default()
        };

        let mut stream = docker.logs(&container_id, Some(opts));
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        loop {
            tokio::select! {
                biased;
                _ = &mut deadline => {
                    let actual = verify_container_status(&docker, &container_id).await;
                    let _ = queries::update_cubelit_status(
                        &pool, &server_id, actual, None,
                    ).await;
                    events.emit(CoreEvent::ServerStatusChanged {
                        server_id: server_id.clone(),
                    });
                    if actual != "running" {
                        tracing::warn!(
                            server_id = %server_id,
                            timeout_secs = %timeout.as_secs(),
                            "Readiness watcher timed out and container is not running"
                        );
                    }
                    break;
                }
                item = stream.next() => {
                    match item {
                        Some(Ok(log)) => {
                            if log.to_string().contains(pattern.as_str()) {
                                let _ = queries::update_cubelit_status(
                                    &pool, &server_id, "running", None,
                                ).await;
                                events.emit(CoreEvent::ServerStatusChanged {
                                    server_id: server_id.clone(),
                                });
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            tracing::warn!(
                                server_id = %server_id,
                                error = %e,
                                "Readiness watcher log stream errored — exiting; sync_single_server will correct status on next poll"
                            );
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    });
}

/// Spawns a background task that subscribes to Docker events and emits
/// `ServerStatusChanged` when a primary managed container stops or dies.
///
/// The task self-heals: if the Docker event stream ends (daemon restart,
/// socket hiccup), it sleeps 5 seconds and reconnects. There is no exit
/// path — the task lives for the lifetime of the process.
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
            filters.insert(
                "event".to_string(),
                vec!["die".to_string(), "stop".to_string()],
            );
            filters.insert(
                "label".to_string(),
                vec!["cubelit.role=primary".to_string()],
            );

            let opts = bollard::query_parameters::EventsOptions {
                filters: Some(filters),
                ..Default::default()
            };

            let mut stream = docker.events(Some(opts));

            while let Some(item) = stream.next().await {
                match item {
                    Ok(ev) => {
                        if let Some(actor) = ev.actor {
                            if let Some(attrs) = actor.attributes {
                                if let Some(server_id) =
                                    attrs.get("cubelit.id").map(String::as_str)
                                {
                                    if let Err(e) = queries::update_cubelit_status(
                                        &pool, server_id, "stopped", None,
                                    )
                                    .await
                                    {
                                        tracing::warn!(
                                            server_id = %server_id,
                                            error = %e,
                                            "Crash watcher failed to persist stopped status"
                                        );
                                    }
                                    events.emit(CoreEvent::ServerStatusChanged {
                                        server_id: server_id.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Crash watcher Docker event stream errored — reconnecting in 5s"
                        );
                        break;
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
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
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
    fn recipe_readiness_default_timeout() {
        use crate::recipes::RecipeReadiness;
        let r: RecipeReadiness =
            serde_json::from_str(r#"{"log_pattern":"Server started"}"#).unwrap();
        assert_eq!(r.log_pattern, "Server started");
        assert_eq!(r.timeout_secs, 600);
    }

    #[test]
    fn recipe_readiness_custom_timeout() {
        use crate::recipes::RecipeReadiness;
        let r: RecipeReadiness =
            serde_json::from_str(r#"{"log_pattern":"VAC secure","timeout_secs":900}"#).unwrap();
        assert_eq!(r.timeout_secs, 900);
    }

    #[test]
    fn recipe_without_readiness_parses() {
        use crate::recipes::Recipe;
        let json = r#"{
            "id":"test","name":"Test","description":"","icon":"test",
            "docker_image":"example/test","default_tag":"1.0",
            "ports":[],"environment":[],"volumes":[]
        }"#;
        let r: Recipe = serde_json::from_str(json).unwrap();
        assert!(r.readiness.is_none());
    }
}
