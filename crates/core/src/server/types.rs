//! Transport-agnostic input types for server-lifecycle operations.
//!
//! `CreateServerConfig` is the single struct callers (Tauri, future CLI,
//! future HTTP agent) hand to `ServerLifecycle::create_server`. It mirrors
//! the JSON shape the desktop frontend has emitted since v0.1.7 — the
//! `Deserialize` impl is part of the IPC contract and must not drift.

use serde::Deserialize;
use std::collections::HashMap;

/// Inputs required to provision a new managed server.
///
/// `name` and `recipe_id` are mandatory; everything else falls back to the
/// recipe's defaults. The `Option` fields are how the desktop frontend
/// transmits "user did not override this" — preserve the shape verbatim
/// or older clients will fail to deserialize.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerConfig {
    pub name: String,
    pub recipe_id: String,
    pub port_overrides: Option<HashMap<String, u16>>,
    pub env_overrides: Option<HashMap<String, String>>,
    pub volume_path: Option<String>,
    /// Override the recipe's `default_tag` (e.g. "java17", "java8", "latest")
    pub tag_override: Option<String>,
    /// Override the recipe's `readiness.timeout_secs` for the watcher spawned
    /// at create time. The smoke harness uses this so long first boots (ASA,
    /// CS2 steamcmd downloads) aren't prematurely resolved by the watcher's
    /// fallback status write. Absent from the desktop wizard payload.
    #[serde(default)]
    pub readiness_timeout_override_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The desktop wizard's v0.1.7 payload (no override field) must keep
    /// deserializing — the field is additive.
    #[test]
    fn create_server_config_deserializes_without_readiness_override() {
        let json = r#"{
            "name": "s",
            "recipe_id": "minecraft-java",
            "port_overrides": null,
            "env_overrides": null,
            "volume_path": null,
            "tag_override": null
        }"#;
        let cfg: CreateServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.readiness_timeout_override_secs, None);
    }
}
