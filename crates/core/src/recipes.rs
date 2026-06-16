use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub docker_image: String,
    pub default_tag: String,
    pub ports: Vec<RecipePort>,
    pub environment: Vec<RecipeEnvVar>,
    pub volumes: Vec<RecipeVolume>,
    #[serde(default)]
    pub config_files: Vec<RecipeConfigFile>,
    #[serde(default)]
    pub mods: Option<RecipeMods>,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub estimated_disk_mb: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional Docker CMD override, passed verbatim to the container entrypoint.
    /// Use when the image requires runtime flags that cannot be expressed as env vars
    /// (e.g. Terraria's `-autocreate 2 -worldname MyWorld`).
    #[serde(default)]
    pub server_cmd: Option<Vec<String>>,
    /// Log-based readiness detection. When present, the server status stays
    /// `"starting"` until `log_pattern` appears in the container logs, at
    /// which point it is promoted to `"running"`. When absent, the container
    /// being alive 2 s after start is sufficient.
    #[serde(default)]
    pub readiness: Option<RecipeReadiness>,
}

/// Controls log-pattern-based readiness detection for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeReadiness {
    /// Substring to search for in container log output. The first line
    /// containing this string promotes the server from `starting` → `running`.
    pub log_pattern: String,
    /// Maximum seconds to wait for the pattern before timing out.
    /// Defaults to 600 (10 minutes). Heavy SteamCMD games (ARK, CS2) may
    /// need 900+.
    #[serde(default = "default_readiness_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_readiness_timeout_secs() -> u64 {
    600
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePort {
    pub container_port: u16,
    pub default_host_port: u16,
    pub protocol: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEnvVar {
    pub key: String,
    pub default_value: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVolume {
    pub container_path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeConfigFile {
    pub path: String,
    pub format: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMods {
    pub supported: bool,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub file_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub available: bool,
    pub tags: Vec<String>,
}

pub fn load_recipes(recipes_dir: &Path) -> CoreResult<Vec<Recipe>> {
    let mut recipes = Vec::new();

    if !recipes_dir.exists() {
        return Ok(recipes);
    }

    for entry in std::fs::read_dir(recipes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<Recipe>(&content) {
                Ok(recipe) => recipes.push(recipe),
                Err(e) => {
                    tracing::warn!(
                        recipe_path = %path.display(),
                        error = %e,
                        "Failed to parse recipe — skipping"
                    );
                }
            }
        }
    }

    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(recipes)
}

pub fn get_recipe(recipes_dir: &Path, id: &str) -> CoreResult<Recipe> {
    let recipes = load_recipes(recipes_dir)?;
    recipes
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| CoreError::NotFound(format!("Recipe '{}' not found", id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_recipe(dir: &std::path::Path, filename: &str, json: &str) {
        let path = dir.join(filename);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    const MINIMAL_RECIPE: &str = r#"{
        "id": "test-game",
        "name": "Test Game",
        "description": "A test game server.",
        "icon": "test-game",
        "available": true,
        "docker_image": "example/test",
        "default_tag": "1.0.0",
        "ports": [{"container_port": 7777, "default_host_port": 7777, "protocol": "udp", "label": "Game Port"}],
        "environment": [{"key": "MAX_PLAYERS", "default_value": "16", "label": "Max Players", "type": "number", "options": []}],
        "volumes": [{"container_path": "/data", "label": "Data"}],
        "estimated_disk_mb": 500,
        "tags": ["test"]
    }"#;

    #[test]
    fn load_recipes_parses_valid_json() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let recipes = load_recipes(dir.path()).unwrap();
        assert_eq!(recipes.len(), 1);
        let r = &recipes[0];
        assert_eq!(r.id, "test-game");
        assert_eq!(r.docker_image, "example/test");
        assert_eq!(r.default_tag, "1.0.0");
        assert!(r.available);
        assert_eq!(r.ports[0].container_port, 7777);
    }

    #[test]
    fn load_recipes_skips_invalid_json() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "broken.json", "{ not valid json }");
        let recipes = load_recipes(dir.path()).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn load_recipes_ignores_non_json_files() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "readme.txt", "not a recipe");
        let recipes = load_recipes(dir.path()).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn load_recipes_returns_empty_for_missing_dir() {
        let recipes = load_recipes(std::path::Path::new("/nonexistent/path/xyz")).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn get_recipe_finds_by_id() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let recipe = get_recipe(dir.path(), "test-game").unwrap();
        assert_eq!(recipe.id, "test-game");
    }

    #[test]
    fn get_recipe_returns_not_found_for_missing_id() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let err = get_recipe(dir.path(), "does-not-exist").unwrap_err();
        assert!(matches!(err, CoreError::NotFound(_)));
    }

    /// Validates all bundled recipes in src-tauri/recipes/ at compile time.
    ///
    /// Some images publish no semver tags, so pinned tags may be timestamp,
    /// channel, or commit-style tags. The rolling "latest" tag is still not
    /// allowed for any available bundled recipe.
    #[test]
    fn bundled_recipes_pass_validation() {
        let recipes_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/recipes");

        if !recipes_dir.exists() {
            return; // not running from a full monorepo checkout
        }

        let recipes = load_recipes(&recipes_dir).expect("failed to load bundled recipes");
        assert!(
            !recipes.is_empty(),
            "no recipes found in {}",
            recipes_dir.display()
        );

        const VALID_FIELD_TYPES: &[&str] = &["string", "number", "boolean", "select", "ram"];
        const VALID_PROTOCOLS: &[&str] = &["tcp", "udp"];

        let mut seen_ids = std::collections::HashSet::new();
        for r in &recipes {
            let ctx = format!("recipe '{}'", r.id);

            // IDs must be unique
            assert!(seen_ids.insert(r.id.clone()), "{ctx}: duplicate recipe id");

            // Required string fields must not be empty
            assert!(!r.id.is_empty(), "{ctx}: id is empty");
            assert!(!r.name.is_empty(), "{ctx}: name is empty");
            assert!(!r.docker_image.is_empty(), "{ctx}: docker_image is empty");
            assert!(!r.default_tag.is_empty(), "{ctx}: default_tag is empty");

            // Available recipes must pin a named tag (not the rolling "latest")
            if r.available {
                assert_ne!(
                    r.default_tag, "latest",
                    "{ctx}: available recipe uses unpinned 'latest' tag \
                     (image: {}). Pin to a named version, channel, timestamp, \
                     or commit tag.",
                    r.docker_image
                );
            }

            // Port validation
            let mut port_keys: std::collections::HashSet<(u16, &str)> =
                std::collections::HashSet::new();
            for p in &r.ports {
                assert!(
                    p.container_port > 0,
                    "{ctx}: port '{}' has zero container_port",
                    p.label
                );
                assert!(
                    p.default_host_port > 0,
                    "{ctx}: port '{}' has zero default_host_port",
                    p.label
                );
                assert!(
                    VALID_PROTOCOLS.contains(&p.protocol.as_str()),
                    "{ctx}: port '{}' has invalid protocol '{}' (must be tcp or udp)",
                    p.label,
                    p.protocol
                );
                // Duplicate (container_port, protocol) within one recipe → Docker binding conflict
                assert!(
                    port_keys.insert((p.container_port, p.protocol.as_str())),
                    "{ctx}: duplicate (container_port={}, protocol={}) — \
                     Docker cannot bind the same port/protocol twice",
                    p.container_port,
                    p.protocol
                );
            }

            // Environment variable validation
            for env in &r.environment {
                assert!(!env.key.is_empty(), "{ctx}: env var with empty key");
                assert!(
                    VALID_FIELD_TYPES.contains(&env.field_type.as_str()),
                    "{ctx}: env '{}' has invalid type '{}' (valid: {})",
                    env.key,
                    env.field_type,
                    VALID_FIELD_TYPES.join(", ")
                );
                if env.field_type == "select" {
                    assert!(
                        !env.options.is_empty(),
                        "{ctx}: env '{}' is type 'select' but has no options",
                        env.key
                    );
                }
            }

            // Volume container paths must be absolute
            for v in &r.volumes {
                assert!(
                    v.container_path.starts_with('/'),
                    "{ctx}: volume '{}' has non-absolute container_path '{}'",
                    v.label,
                    v.container_path
                );
            }
        }
    }

    #[test]
    fn ark_recipe_exposes_workshop_mods_on_persistent_volume() {
        let recipes_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/recipes");

        if !recipes_dir.exists() {
            return;
        }

        let ark = get_recipe(&recipes_dir, "ark").expect("failed to load ARK recipe");

        assert_eq!(ark.volumes[0].container_path, "/app");
        assert!(
            ark.environment.iter().any(|env| env.key == "GAME_MOD_IDS"),
            "ARK recipe must expose hermsi/ark-server GAME_MOD_IDS for Workshop mods"
        );
        assert_eq!(
            ark.mods.and_then(|mods| mods.path),
            Some("server/ShooterGame/Content/Mods".to_string())
        );
    }

    #[test]
    fn ark_survival_ascended_recipe_matches_container_contract() {
        let recipes_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/recipes");

        if !recipes_dir.exists() {
            return;
        }

        let asa = get_recipe(&recipes_dir, "ark-survival-ascended")
            .expect("failed to load ARK: Survival Ascended recipe");

        assert_eq!(asa.docker_image, "johnnyknighten/ark-sa-server");
        assert_eq!(asa.default_tag, "2.2.2");
        assert!(asa.estimated_disk_mb >= 100_000);
        assert!(asa.environment.iter().any(|env| env.key == "MOD_LIST"));
        assert!(asa.environment.iter().any(|env| env.key == "MAP"));
        assert!(asa.environment.iter().any(|env| env.key == "MANUAL_CONFIG"));
        assert_eq!(
            asa.volumes
                .iter()
                .map(|v| v.container_path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "/ark-server/server",
                "/ark-server/logs",
                "/ark-server/backups"
            ]
        );
        assert_eq!(
            asa.ports
                .iter()
                .map(|p| (p.container_port, p.protocol.as_str()))
                .collect::<Vec<_>>(),
            vec![(7777, "udp"), (7778, "udp"), (27015, "udp"), (27020, "tcp")]
        );
        assert_eq!(
            asa.readiness.map(|r| r.log_pattern),
            Some("Server has completed startup and is now advertising for join".to_string())
        );
    }
}
