//! Smoke-test harness for game server recipes.
//!
//! `run_smoke` boots each requested recipe through the full `LocalServerHost`
//! lifecycle (create → readiness watch → delete) and returns a `SmokeReport`
//! describing whether each server reached the joinable state.
//!
//! The harness is consumed by two callers:
//!   - `cubelit smoke-test` CLI subcommand (Phase 3)
//!   - `#[ignore]` integration tests in `crates/cli/tests/integration.rs` (Phase 4)
//!
//! Running all 10 games sequentially is the safe default; some images (CS2,
//! ARK) are 15–35 GB and pulling them in parallel would saturate the host.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use tracing::info;

use crate::error::{CoreError, CoreResult};
use crate::events::EventSink;
use crate::recipes;
use crate::server::{CreateServerConfig, LocalServerHost, ServerLifecycle};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Options controlling a smoke run.
pub struct SmokeOptions {
    /// Recipe IDs to test. An empty vec means "all `available == true` recipes".
    pub recipe_ids: Vec<String>,
    /// Per-game wall-clock cap. Overrides the recipe's own `timeout_secs` for
    /// the polling loop (the readiness watcher still uses its own recipe timeout
    /// internally, but the smoke runner won't wait longer than this).
    pub overall_timeout: Duration,
    /// When true, failed servers are not deleted so their logs can be inspected.
    pub keep_on_failure: bool,
    /// Added to every recipe's default host port to avoid clashing with real
    /// servers running on the host. Also applied to FiveM's `DB_HOST_PORT` env
    /// var so the MariaDB sidecar doesn't collide with any host database.
    pub port_offset: u16,
    /// How many games to run concurrently. Default and recommended value is 1.
    pub parallel: usize,
}

impl Default for SmokeOptions {
    fn default() -> Self {
        Self {
            recipe_ids: vec![],
            overall_timeout: Duration::from_secs(480),
            keep_on_failure: false,
            port_offset: 10_000,
            parallel: 1,
        }
    }
}

/// The outcome for a single recipe's smoke run.
#[derive(Debug)]
pub enum SmokeOutcome {
    /// The readiness log pattern was matched — server is joinable.
    Ready { duration: Duration },
    /// Container came up but the recipe has no readiness pattern configured.
    /// The smoke runner considered it passing after the 2s post-start check.
    Started { duration: Duration },
    /// Image pull or container creation failed before the server could start.
    ImagePullFailed(String),
    /// The container exited or crashed before the readiness pattern matched.
    ContainerCrashed { last_logs: Vec<String> },
    /// The overall_timeout elapsed without the server becoming ready.
    Timeout { last_logs: Vec<String> },
}

impl SmokeOutcome {
    /// Returns true for outcomes that are considered "passing" (the server ran).
    pub fn is_passing(&self) -> bool {
        matches!(self, SmokeOutcome::Ready { .. } | SmokeOutcome::Started { .. })
    }

    pub fn label(&self) -> &'static str {
        match self {
            SmokeOutcome::Ready { .. } => "ready",
            SmokeOutcome::Started { .. } => "started",
            SmokeOutcome::ImagePullFailed(_) => "pull-failed",
            SmokeOutcome::ContainerCrashed { .. } => "crashed",
            SmokeOutcome::Timeout { .. } => "timeout",
        }
    }
}

/// Result for a single recipe.
pub struct SmokeResult {
    pub recipe_id: String,
    /// `docker_image:tag` string used during this run.
    pub image: String,
    pub outcome: SmokeOutcome,
}

/// Aggregated report returned by `run_smoke`.
pub struct SmokeReport {
    pub results: Vec<SmokeResult>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

impl SmokeReport {
    /// Returns true if every result is passing.
    pub fn all_passing(&self) -> bool {
        self.results.iter().all(|r| r.outcome.is_passing())
    }
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Run the smoke harness against the requested recipes (or all available ones)
/// and return a `SmokeReport`. Callers supply an `EventSink` for progress output
/// during image pulls; pass `Arc::new(NoopSink)` to silence them.
pub async fn run_smoke(
    host: &LocalServerHost,
    opts: SmokeOptions,
    events: Arc<dyn EventSink>,
) -> CoreResult<SmokeReport> {
    let started_at = Utc::now();

    // Resolve the recipe list.
    let all_recipes = recipes::load_recipes(&host.recipes_dir)?;
    let target_recipes: Vec<_> = if opts.recipe_ids.is_empty() {
        all_recipes.into_iter().filter(|r| r.available).collect()
    } else {
        let mut out = Vec::new();
        for id in &opts.recipe_ids {
            match all_recipes.iter().find(|r| &r.id == id) {
                Some(r) => out.push(r.clone()),
                None => {
                    return Err(CoreError::NotFound(format!(
                        "Smoke: recipe '{}' not found",
                        id
                    )))
                }
            }
        }
        out
    };

    let parallel = opts.parallel.max(1);
    let mut results: Vec<SmokeResult> = Vec::new();

    // Simple sequential implementation (parallel=1) with a chunked path for
    // parallel > 1 via tokio::join. For now we always run one at a time since
    // the default is 1 and heavy images don't benefit from parallelism.
    for chunk in target_recipes.chunks(parallel) {
        let mut handles = Vec::new();
        for recipe in chunk {
            let recipe = recipe.clone();
            let events = events.clone();
            let overall_timeout = opts.overall_timeout;
            let keep_on_failure = opts.keep_on_failure;
            let port_offset = opts.port_offset;

            // We can't move `host` into a spawned task (it's `!Send` for the
            // sqlite pool in some configs), so run sequentially within the
            // chunk instead of actually spawning.
            let result =
                smoke_one(host, recipe, overall_timeout, keep_on_failure, port_offset, events)
                    .await;
            handles.push(result);
        }
        results.extend(handles);
    }

    // Persist the JSON report to data_dir/smoke/.
    let smoke_dir = host.data_dir.join("smoke");
    if let Err(e) = std::fs::create_dir_all(&smoke_dir) {
        tracing::warn!(error = %e, "Could not create smoke output directory");
    } else {
        let ts = started_at.format("%Y%m%d-%H%M%S");
        let path = smoke_dir.join(format!("cubelit-smoke-{}.json", ts));
        if let Ok(json) = build_report_json(&results, &started_at, &Utc::now()) {
            let _ = std::fs::write(&path, json);
            info!(path = %path.display(), "Smoke report written");
        }
    }

    Ok(SmokeReport {
        results,
        started_at,
        finished_at: Utc::now(),
    })
}

// ─── Per-recipe logic ─────────────────────────────────────────────────────────

async fn smoke_one(
    host: &LocalServerHost,
    recipe: recipes::Recipe,
    overall_timeout: Duration,
    keep_on_failure: bool,
    port_offset: u16,
    events: Arc<dyn EventSink>,
) -> SmokeResult {
    let image = format!("{}:{}", recipe.docker_image, recipe.default_tag);
    info!(recipe_id = %recipe.id, image = %image, "Smoke: starting");

    // Build port overrides with the offset applied.
    let port_overrides: HashMap<String, u16> = recipe
        .ports
        .iter()
        .map(|p| {
            (
                format!("{}/{}", p.container_port, p.protocol),
                p.default_host_port.saturating_add(port_offset),
            )
        })
        .collect();

    // FiveM: offset the DB_HOST_PORT env var too so the MariaDB sidecar
    // doesn't collide with any host MariaDB instance.
    let env_overrides: Option<HashMap<String, String>> = if recipe.id == "fivem" {
        let db_port = 3306u16.saturating_add(port_offset);
        Some(HashMap::from([(
            "DB_HOST_PORT".to_string(),
            db_port.to_string(),
        )]))
    } else {
        None
    };

    let server_name = format!(
        "smoke-{}-{}",
        recipe.id,
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    let config = CreateServerConfig {
        name: server_name,
        recipe_id: recipe.id.clone(),
        port_overrides: Some(port_overrides),
        env_overrides,
        volume_path: None,
        tag_override: None,
    };

    let has_readiness = recipe.readiness.is_some();
    let wall_start = Instant::now();

    let server = match host.create_server(config, events).await {
        Ok(s) => s,
        Err(e) => {
            info!(recipe_id = %recipe.id, error = %e, "Smoke: create_server failed");
            return SmokeResult {
                recipe_id: recipe.id,
                image,
                outcome: SmokeOutcome::ImagePullFailed(e.to_string()),
            };
        }
    };

    // Poll the DB status every 2s until the server reaches a terminal state
    // or the overall timeout fires.
    let outcome = poll_until_ready(host, &server.id, has_readiness, overall_timeout, wall_start).await;

    let is_failure = !outcome.is_passing();
    if is_failure && keep_on_failure {
        info!(
            recipe_id = %recipe.id,
            server_id = %server.id,
            "Smoke: keeping failed server for inspection"
        );
    } else {
        let _ = host.delete_server(&server.id, true).await;
    }

    info!(
        recipe_id = %recipe.id,
        outcome = %outcome.label(),
        elapsed_secs = %wall_start.elapsed().as_secs(),
        "Smoke: done"
    );

    SmokeResult {
        recipe_id: recipe.id,
        image,
        outcome,
    }
}

async fn poll_until_ready(
    host: &LocalServerHost,
    server_id: &str,
    has_readiness: bool,
    overall_timeout: Duration,
    wall_start: Instant,
) -> SmokeOutcome {
    let poll_interval = Duration::from_secs(2);

    loop {
        // Respect the overall cap.
        if wall_start.elapsed() >= overall_timeout {
            let last_logs = fetch_last_logs(host, server_id).await;
            return SmokeOutcome::Timeout { last_logs };
        }

        let server = match host.get_server(server_id).await {
            Ok(s) => s,
            Err(_) => {
                return SmokeOutcome::ContainerCrashed { last_logs: vec![] };
            }
        };

        match server.status.as_str() {
            "running" => {
                return SmokeOutcome::Ready {
                    duration: wall_start.elapsed(),
                };
            }
            "error" | "stopped" => {
                let last_logs = fetch_last_logs(host, server_id).await;
                return SmokeOutcome::ContainerCrashed { last_logs };
            }
            "starting" => {
                // Readiness watcher is running in background — keep polling.
            }
            _ => {
                // "created" or unknown — still initialising, keep polling.
                if !has_readiness && wall_start.elapsed() > Duration::from_secs(10) {
                    // No readiness pattern and container has been up long enough.
                    // Check if it's actually running by looking at the stored status.
                    if server.status == "running" {
                        return SmokeOutcome::Started {
                            duration: wall_start.elapsed(),
                        };
                    }
                }
            }
        }

        // For recipes without a readiness pattern, once the container is
        // marked "running" (2s post-start) that's our passing signal.
        if !has_readiness && server.status == "running" {
            return SmokeOutcome::Started {
                duration: wall_start.elapsed(),
            };
        }

        tokio::time::sleep(poll_interval).await;
    }
}

async fn fetch_last_logs(host: &LocalServerHost, server_id: &str) -> Vec<String> {
    host.server_logs(server_id, Some(200)).await.unwrap_or_default()
}

// ─── JSON serialisation (no serde dependency on SmokeReport itself) ───────────

fn build_report_json(
    results: &[SmokeResult],
    started_at: &DateTime<Utc>,
    finished_at: &DateTime<Utc>,
) -> Result<String, serde_json::Error> {
    let rows: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let (duration_secs, detail) = match &r.outcome {
                SmokeOutcome::Ready { duration } => (duration.as_secs(), serde_json::Value::Null),
                SmokeOutcome::Started { duration } => (duration.as_secs(), serde_json::Value::Null),
                SmokeOutcome::ImagePullFailed(msg) => {
                    (0, serde_json::Value::String(msg.clone()))
                }
                SmokeOutcome::ContainerCrashed { last_logs } => {
                    (0, serde_json::json!(last_logs))
                }
                SmokeOutcome::Timeout { last_logs } => {
                    (0, serde_json::json!(last_logs))
                }
            };
            serde_json::json!({
                "recipe_id": r.recipe_id,
                "image": r.image,
                "outcome": r.outcome.label(),
                "passing": r.outcome.is_passing(),
                "duration_secs": duration_secs,
                "detail": detail,
            })
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "started_at": started_at.to_rfc3339(),
        "finished_at": finished_at.to_rfc3339(),
        "results": rows,
    }))
}

