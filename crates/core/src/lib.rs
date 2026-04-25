//! Cubelit core: DB, Docker, recipes, and server lifecycle, free of any UI
//! transport (Tauri, HTTP, CLI). Consumers wire their own sink to receive
//! progress / status events emitted by long-running operations.
//!
//! v0.1.8 introduces this crate as an empty skeleton; subsequent commits
//! relocate the existing `src-tauri/src/{error,ports,recipes,db,docker,…}`
//! modules into here behind explicit traits.
