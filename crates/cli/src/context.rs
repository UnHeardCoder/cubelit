use std::path::{Path, PathBuf};

use include_dir::{include_dir, Dir};

use cubelit_core::error::{CoreError, CoreResult};
use cubelit_core::server::LocalServerHost;

static EMBEDDED_RECIPES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../src-tauri/recipes");

/// Match desktop data dir: Linux `~/.config/cubelit`, Windows/macOS under `dirs::data_dir()`.
/// Respects the `CUBELIT_DATA_DIR` environment variable override.
pub fn resolve_data_dir() -> CoreResult<PathBuf> {
    if let Some(s) = std::env::var_os("CUBELIT_DATA_DIR") {
        return Ok(PathBuf::from(s));
    }
    #[cfg(target_os = "linux")]
    let base = dirs::config_dir().ok_or_else(|| {
        CoreError::Validation("Linux: XDG config directory ($XDG_CONFIG_HOME) is not available".into())
    })?;
    #[cfg(not(target_os = "linux"))]
    let base = dirs::data_dir().ok_or_else(|| {
        CoreError::Validation("Platform data directory is not available".into())
    })?;
    Ok(base.join("cubelit"))
}

fn needs_recipe_seed(dir: &Path) -> bool {
    if !dir.exists() {
        return true;
    }
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return true;
    };
    let json_count = read_dir
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .count();
    json_count == 0
}

pub fn seed_embedded_recipes_if_needed(dir: &Path) -> CoreResult<()> {
    if !needs_recipe_seed(dir) {
        return Ok(());
    }
    std::fs::create_dir_all(dir)?;
    for file in EMBEDDED_RECIPES.files() {
        let rel = file.path();
        let dest = dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, file.contents())?;
    }
    Ok(())
}

/// Resolve the recipes directory.
/// Priority: `--recipes-dir` flag → `CUBELIT_RECIPES_DIR` env → `{data_dir}/recipes` (seeded from embedded bundle).
/// The `data_dir` parameter is the already-resolved data directory so that `CUBELIT_DATA_DIR`
/// overrides affect the default recipes path as well.
pub fn resolve_recipes_dir(cli_flag: Option<&PathBuf>, data_dir: &Path) -> CoreResult<PathBuf> {
    if let Some(p) = cli_flag {
        return Ok(p.clone());
    }
    if let Ok(s) = std::env::var("CUBELIT_RECIPES_DIR") {
        return Ok(PathBuf::from(s));
    }
    let recipes = data_dir.join("recipes");
    seed_embedded_recipes_if_needed(&recipes)?;
    Ok(recipes)
}

pub struct Context {
    pub host: LocalServerHost,
}

impl Context {
    pub async fn new(recipes_flag: Option<PathBuf>) -> CoreResult<Self> {
        let data_dir = resolve_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        let recipes_dir = resolve_recipes_dir(recipes_flag.as_ref(), &data_dir)?;
        let host = LocalServerHost::new(data_dir, recipes_dir).await?;
        Ok(Self { host })
    }
}
