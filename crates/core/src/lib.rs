//! Cubelit core: DB, Docker, recipes, and server lifecycle, free of any UI
//! transport (Tauri, HTTP, CLI). Consumers wire their own sink to receive
//! progress / status events emitted by long-running operations.

pub mod db;
pub mod docker;
pub mod error;
pub mod events;
pub mod ports;
pub mod recipes;

pub use error::{CoreError, CoreResult};
pub use events::{CoreEvent, EventSink, ImagePullProgress, NoopSink, ServerCreateProgress};
