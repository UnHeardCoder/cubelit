//! `ServerLifecycle` — the trait that owns the DB, the recipe directory, and
//! every cross-cutting decision about a Cubelit-managed server.
//!
//! Where `ServerRunner` is the abstract "container plumbing" seam,
//! `ServerLifecycle` is the abstract "what does Cubelit do with a server"
//! seam: create, start, stop, restart, delete, sync, list, rename, fetch
//! logs/stats, send game commands, take backups.
//!
//! The trait shape mirrors the Tauri command surface so the desktop
//! `commands/*` modules can collapse into thin shims that just `state.host
//! .<method>(args).await`. Future consumers (CLI, HTTP agent) do the same
//! against the same trait, in their own transports.
//!
//! Sink threading: methods that spawn background watcher tasks take an
//! `Arc<dyn EventSink>` so the watcher can hold its own clone. Methods
//! that only emit synchronously could take `&dyn EventSink`, but for
//! API consistency they all take the Arc form. Callers construct the
//! Arc once at the transport boundary (e.g. `TauriEventSink::shared`)
//! and reuse it for every lifecycle call.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::db::models::Cubelit;
use crate::docker::stats::ContainerStats;
use crate::error::CoreResult;
use crate::events::EventSink;

use super::types::CreateServerConfig;

#[async_trait]
pub trait ServerLifecycle: Send + Sync {
    // ─── Mutating ops ─────────────────────────────────────────────────────

    /// Provision a new server: load recipe, persist DB row, pull image,
    /// (optionally) set up sidecars, create + start container, spawn
    /// readiness watcher if applicable. Emits `ServerCreateProgress`
    /// events at each step.
    async fn create_server(
        &self,
        config: CreateServerConfig,
        events: Arc<dyn EventSink>,
    ) -> CoreResult<Cubelit>;

    /// Start an already-provisioned server. Emits `ServerStatusChanged`
    /// once the container is running (or once the readiness pattern
    /// matches, for games that need it).
    async fn start_server(&self, id: &str, events: Arc<dyn EventSink>) -> CoreResult<()>;

    async fn stop_server(&self, id: &str) -> CoreResult<()>;

    async fn restart_server(&self, id: &str, events: Arc<dyn EventSink>) -> CoreResult<()>;

    /// Tear down container(s), optionally remove the on-disk volume.
    /// `delete_data = false` keeps the user's world/save files on disk.
    async fn delete_server(&self, id: &str, delete_data: bool) -> CoreResult<()>;

    /// Recreate the container with a new env-var set, preserving running
    /// state if the server was previously running.
    async fn update_server_settings(
        &self,
        id: &str,
        environment: HashMap<String, String>,
        events: Arc<dyn EventSink>,
    ) -> CoreResult<Cubelit>;

    async fn rename_server(&self, id: &str, name: &str) -> CoreResult<Cubelit>;

    // ─── Read ops ─────────────────────────────────────────────────────────

    /// Reconcile one server's DB status with Docker reality. Returns the
    /// server row after any correction.
    async fn sync_single(&self, id: &str) -> CoreResult<Cubelit>;

    /// Reconcile every server. Used at app startup so the UI doesn't show
    /// "running" servers whose containers actually died overnight.
    async fn sync_all(&self) -> CoreResult<Vec<Cubelit>>;

    async fn list_servers(&self) -> CoreResult<Vec<Cubelit>>;

    async fn get_server(&self, id: &str) -> CoreResult<Cubelit>;

    /// Tail `lines` lines of stdout+stderr (default 100) from the server's
    /// primary container. Returns each log frame as a separate string.
    async fn server_logs(&self, id: &str, lines: Option<u64>) -> CoreResult<Vec<String>>;

    async fn server_stats(&self, id: &str) -> CoreResult<ContainerStats>;

    // ─── Game-specific ops ────────────────────────────────────────────────

    /// Send a Minecraft RCON command and return the response. Errors with
    /// `Validation` if the server isn't running, RCON port isn't mapped,
    /// or auth fails.
    async fn send_minecraft_command(&self, id: &str, command: &str) -> CoreResult<String>;

    /// Snapshot the server's data directory into a timestamped backup
    /// folder. Returns the absolute path of the new backup.
    async fn backup_server(&self, id: &str) -> CoreResult<String>;
}
