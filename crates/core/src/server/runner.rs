//! `ServerRunner` — the trait that abstracts "where containers actually run".
//!
//! v0.1.8 ships a single implementation (`LocalServerHost` driving the local
//! Docker socket via bollard). The trait exists so that v0.1.9's CLI and the
//! v0.2.0+ remote agent can swap in alternative runtimes (e.g. an HTTP/WS
//! Docker proxy) without disturbing the orchestration code in `lifecycle.rs`.
//!
//! The trait is deliberately narrow: only the runtime container ops that are
//! transport-substitutable. Anything that touches the SQLite pool or the
//! recipe directory belongs on `ServerLifecycle` instead.
//!
//! `async-trait` is used so the trait remains object-safe. We do not yet
//! consume it as `dyn ServerRunner`, but keeping the option open is cheap
//! and matches the EventSink pattern.

use async_trait::async_trait;

use crate::db::models::Cubelit;
use crate::docker::stats::ContainerStats;
use crate::error::CoreResult;
use crate::events::EventSink;

#[async_trait]
pub trait ServerRunner: Send + Sync {
    /// Pull a Docker image, streaming progress through `events`.
    async fn pull_image(&self, image: &str, events: &dyn EventSink) -> CoreResult<()>;

    /// Create the primary container for `cubelit`. `extra_binds` are appended
    /// to the recipe's default bind list — used by FiveM to attach `/txData`.
    /// Returns the new container ID.
    async fn create_container(
        &self,
        cubelit: &Cubelit,
        extra_binds: &[String],
    ) -> CoreResult<String>;

    /// Start a previously-created container. Idempotent at the Docker level.
    async fn start_container(&self, container_id: &str) -> CoreResult<()>;

    async fn stop_container(&self, container_id: &str) -> CoreResult<()>;

    async fn restart_container(&self, container_id: &str) -> CoreResult<()>;

    async fn remove_container(&self, container_id: &str) -> CoreResult<()>;

    /// Quick "is the container actually running right now?" probe.
    /// Returns `false` for stopped containers AND for any inspection error
    /// (treats "I can't tell" as "not running" — caller expects a boolean).
    async fn is_running(&self, container_id: &str) -> bool;

    /// Tail `lines` lines of stdout+stderr from the container.
    async fn container_logs(&self, container_id: &str, lines: u64) -> CoreResult<Vec<String>>;

    /// Single-shot CPU+memory snapshot.
    async fn container_stats(&self, container_id: &str) -> CoreResult<ContainerStats>;
}
