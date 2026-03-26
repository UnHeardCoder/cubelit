# Changelog

All notable changes to Cubelit are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

---

## [0.1.5] — 2026-03-25

### Added
- CI/CD pipeline: website builds on tag push, pushes Docker image to GHCR, deploys to VPS via SSH
- Version check CI — release workflow fails fast if tag doesn't match all three version files
- `deploy-website.yml` workflow: bun build → GHCR push → SSH deploy to production host

### Changed
- `website/deploy/Dockerfile`: now COPYs `dist/` into the image instead of relying on a volume mount
- `website/deploy/compose.yml`: switched from local build to `ghcr.io/unheardcoder/cubelit-website:latest`
- README clone URL fixed (`yourusername` → `UnHeardCoder`)
- CLAUDE.md: added CI/CD section, CHANGELOG requirement, and commit message rules

---

## [0.1.4] — 2026-03-25

### Fixed
- Installer filenames now correctly labelled 0.1.4 (version was not bumped before the v0.1.3 tag)
- `.gitignore` line 16 was a literal shell command, not a pattern — replaced with `*:Zone.Identifier`
- `cubelit.key.pub` added to `.gitignore` permanently

### Changed
- Bump Cargo patch deps: tauri 2.10.3, tauri-build 2.5.6, tokio 1.50.0, tempfile 3.27.0
- Validation error messages now specify which env var failed and why

### Added
- Unit tests for `validate_env_vars` (5 tests)

---

## [0.1.3] — 2026-03-25

### Added
- File-based logging to `cubelit.log` in app data directory (tracing + tracing-subscriber)
- PR CI workflow (`.github/workflows/ci.yml`) — cargo check/clippy/test + bun check/test on every PR
- macOS Apple Silicon (aarch64) builds in release workflow
- `SECURITY.md` with supported versions, private vulnerability reporting, and threat model
- Troubleshooting section in README
- Game support clarification in README (only Minecraft Java + FiveM fully supported in v0.1)
- `validate_env_vars` helper — env var length and NUL byte validation on server create/update

### Changed
- Fixed 3 pre-existing clippy warnings
- Authors field in Cargo.toml: `"you"` → `"Cubelit Contributors"`
- Env filter order corrected to `"warn,cubelit=info"`

---

## [0.1.2] — 2026-03-23

### Added
- Git workflow guidelines in CLAUDE.md
- Test suite: 12 Rust unit tests (ports, recipes, error), 9 sqlx DB integration tests, 13 Svelte/TS API tests
- `cargo sqlx prepare` offline query cache — builds no longer require a live database
- `migrations/001_init.sql` — declarative schema replaces inline `CREATE TABLE` in code
- Dependabot config for automated dependency updates (Cargo + npm, weekly)

### Changed
- sqlx queries migrated from runtime strings to compile-time `query!` / `query_as!` macros
- Minecraft Java recipe `default_tag` pinned from `"latest"` to `"java21"` (itzg/minecraft-server)
- `package.json` description filled in

### Removed
- `cubelit.key.pub` — stale public key removed from repo

---

## [0.1.1] — 2026-03-22

### Fixed
- Updater artifact signing: switched to `TAURI_SIGNING_PRIVATE_KEY` env var
- Windows signing key: strip carriage returns; write key to temp file to preserve newlines
- Windows onboarding flow and Docker Desktop store link
- Null pointer type for `ShellExecuteW` HWND parameter on Windows

---

## [0.1.0] — 2026-03-10

### Added
- Initial release of Cubelit
- Docker-backed game server management (Minecraft Java, FiveM)
- Create server wizard with recipe-driven configuration (3-step flow)
- Server detail pages with start/stop/restart, logs, file manager, stats
- Minecraft RCON command console and server backup
- FiveM support with MariaDB sidecar, txAdmin web panel
- Custom logo and sidebar with live server list
- Docker onboarding gate (blocks UI until Docker is available)
- Windows: WSL2 detection and setup helper
- Plugins tab, drag-and-drop file upload, folder opener, public IP display
- Auto-updater via `tauri-plugin-updater`
