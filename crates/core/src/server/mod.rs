//! Server lifecycle: provision, start, stop, restart, delete, sync, rename,
//! logs, stats, RCON, backups. The transport layer (Tauri / CLI / HTTP agent)
//! drives this module via the `ServerLifecycle` trait.
//!
//! For v0.1.8 there's exactly one impl — `LocalServerHost` — which targets
//! the local Docker socket and the bundled SQLite DB. Future transports add
//! their own impls without touching the trait surface.

pub mod lifecycle;
pub mod local;
pub mod minecraft;
pub mod runner;
pub mod types;
pub mod watchers;

pub use lifecycle::ServerLifecycle;
pub use local::{sync_all_servers, sync_single_server, LocalServerHost};
pub use runner::ServerRunner;
pub use types::CreateServerConfig;
pub use watchers::{
    readiness_pattern, spawn_crash_watcher, spawn_readiness_watcher, validate_env_vars,
    verify_container_status,
};
