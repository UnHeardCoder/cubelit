# Changelog

All notable changes to Cubelit are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

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
