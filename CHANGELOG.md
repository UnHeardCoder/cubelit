# Changelog

All notable changes to Cubelit are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

---

## [0.2.0] — 2026-06-16

### Added
- Full desktop UI overhaul with a Console-inspired visual direction (Pterodactyl-meets-Linear aesthetic)
- Dark ↔ light theme toggle, persisted to localStorage
- Collapsible sidebar (240px expanded / 64px icon-only) with inline server list and live status dots
- SteamGridDB hero art integration for Minecraft Java and FiveM, plus per-game art/gradient treatments for the full bundled lineup
- Dashboard stat strip for total servers, running, stopped, and unique games
- Server card redesign with hero art banners, GameIcon, and StatusPill
- Server detail redesign with hero banners plus Overview / Console / Files / Settings tabs
- Sparkline CPU / memory charts on server overview panels
- Connection address rows with copy-to-clipboard actions
- Terminal-style server creation progress display
- Docker onboarding redesign with numbered steps, progress strip, and WSL2 → Docker guidance
- Dedicated settings page in the app shell
- New shared UI components: Cube, StatusPill, GaugeCard, Sparkline, ConnRow, GameArt, GameIcon, and SkeletonCard
- Inter + JetBrains Mono typography via @fontsource
- Smoke-test harness in `cubelit-core` for recipe lifecycle validation (create → ready/start → cleanup)
- `cubelit smoke-test` CLI subcommand with tabular output and optional JSON report export
- Ignored Docker-backed CLI integration test scaffold for end-to-end smoke coverage
- ARK: Survival Ascended recipe, game registry entry, UI art treatment, and dashboard support
- Root MIT `LICENSE` file

### Changed
- Design tokens shifted to a deeper, more polished Console-style palette
- Sidebar upgraded from a fixed 64px icon rail to a collapsible navigation + server list layout
- GenericDashboard rebuilt around the new Console-direction layout system
- StatusRibbon replaced by StatusPill across the UI
- Create flow, server detail pages, and onboarding screens were restyled to match the new visual system
- Game art, animated backgrounds, and theme handling were expanded across the dashboard experience
- All 9 previously coming-soon bundled games are now surfaced as available (Bedrock, ARK: Survival Evolved, ARK: Survival Ascended, CS2, Palworld, Project Zomboid, Rust, Terraria, Valheim)
- All bundled recipe `default_tag` values are now pinned to specific image versions instead of `latest`
- Release metadata, docs, and website supported-game messaging were updated for the v0.2.0 release state

### Fixed
- Windows onboarding no longer treats "WSL optional features enabled, default WSL version set to 2, but no standalone distro installed" as a blocker for Docker Desktop. Docker Desktop is expected to create and manage its own WSL backend, and Cubelit now offers an "Open Docker Desktop" action when Docker Desktop is installed but the engine is not running.
- FiveM sidecar MariaDB connection strings now use the correct `mariadb://` DSN format; sidecar container naming and network joins were also corrected
- Terraria now preserves the recipe `server_cmd` during settings updates / container recreation instead of silently dropping it
- Terraria recipe handling was tightened around `server_cmd` metadata, autocreate behavior, and art mapping support
- Readiness lifecycle semantics were corrected: a server moves to `Running` only after readiness actually succeeds, and startup sync no longer clobbers servers still inside the readiness window
- All recipe-declared volumes are now mounted instead of only the first volume entry
- Partial resources (containers, DB rows, and created volume state) are cleaned up when `create_server` fails mid-flight
- Stale cache references and UI accessibility warnings were cleaned up during the overhaul pass
- Loading states, action buttons, and modals were hardened across the dashboard and create flow to reduce duplicate or misleading actions
- Server detail pages once again expose Delete Server actions for game-specific dashboards, including Minecraft Java and FiveM
- Minecraft console actions now remain usable while the UI reports `starting`, avoiding a false-disabled console during early boot

---

## [0.1.10] — 2026-04-28

### Added
- Sentry crash reporting initialized in both the desktop app and CLI.
  Pass `SENTRY_DSN` at build time (or set the env var at runtime) to enable.
  Panics and `CoreError` failures on key lifecycle commands are automatically
  captured. No-op if the DSN is empty — existing behaviour preserved.
- Intel macOS (x86_64) added to the desktop and CLI release matrices
  (`macos-13` GitHub Actions runner, `--target x86_64-apple-darwin`).
  Artifact: `cubelit-cli-macos-intel`.

### Fixed
- `resolve_data_dir()` in the CLI now returns `CoreResult` instead of panicking
  when the platform config/data directory is unavailable.
- Default recipes directory is now derived from the same resolved data dir so
  `CUBELIT_DATA_DIR` affects both the SQLite DB path and the recipe seed path.
- CLI tracing writes to `cubelit.log` in the platform data dir (matching the
  desktop), falling back to stderr when the file is unavailable.
- `cubelit agent start` now exits with code 4 (Validation error) instead of 0,
  so scripts can detect the unimplemented command.
- `--port` flag removed from `agent start` (no-op flag that implied
  configurability).
- Log lines in `CliEventSink` now strip only `\r`/`\n` rather than all trailing
  whitespace, preserving meaningful trailing spaces.
- `crates/cli/Cargo.toml` and `.github/workflows/release.yml` converted from
  CRLF to LF; `.gitattributes` added to enforce LF repo-wide.
- Release `check-version` CI no longer false-fails due to `\r` in extracted
  version strings from CRLF-encoded files.
- `bollard` bumped from 0.18 to 0.20 across all three crates (`crates/core`,
  `crates/cli`, `src-tauri`). Updated call sites in `stats.rs`, `images.rs`,
  `logs.rs`, `containers.rs`, `local.rs`, `watchers.rs`, and
  `crates/cli/src/commands/logs.rs` to match the new API (options types moved
  to `bollard::query_parameters`, `Config` → `ContainerCreateBody`, network
  types to `bollard::models`).
- `ServerLogLine` enum variant now carries `#[allow(dead_code)]` to suppress
  clippy warnings as the workspace grows.
- Stale pre-workspace Dependabot PRs (#22, #23, #24) closed; bollard Dependabot
  PR (#27) closed (resolved manually); safe dependency updates previously merged:
  tokio 1.52.1 (#28), tauri-plugin-updater 2.10.1 (#29), uuid 1.23.1 (#30).

### Changed
- CLI macOS release binary is now pinned to `aarch64-apple-darwin` (matching
  the desktop job) for deterministic architecture.
- Integration test scaffold in `crates/cli/tests/integration.rs` now contains
  a real Docker-backed flow body (guarded by `#[ignore]`).

### Tests
- Added `#[tokio::test]` unit tests to `crates/core/src/server/local.rs`
  covering `list_servers` (empty DB), `get_server` (NotFound), and
  `rename_server` (NotFound) — no Docker daemon required.

---

## [0.1.9] — 2026-04-28

### Added
- `cubelit-cli` workspace crate — standalone Rust CLI binary (`cubelit`) that calls `cubelit-core` directly. Same SQLite database and Docker orchestration as the desktop, no GUI. Subcommands: `server list`, `install`, `start`, `stop`, `restart`, `status`, `remove`; `logs` (follow with Ctrl+C); `agent start` stub for v0.3.0.
- `CliEventSink` — `EventSink` implementation that streams create/pull/status events to stderr (progress lines) and log lines to stdout where applicable.
- Identifier resolver: full UUID, unique server name, or unique UUID prefix (4+ characters); ambiguous matches return a validation error.
- Embedded recipe bundle with first-run seeding under the default recipes directory; overrides via `--recipes-dir` and `CUBELIT_RECIPES_DIR`.
- `cubelit.1` manual page under `crates/cli/man/` (install to `share/man/man1` for `man cubelit`).
- `build-cli` GitHub Actions job attaching `cubelit-cli-linux`, `cubelit-cli-macos`, and `cubelit-cli-windows.exe` to the same release as the desktop app.

### Changed
- Workspace root `Cargo.toml` includes `crates/cli`.
- Desktop package default binary is `cubelit-desktop` (`src-tauri/src/bin/cubelit-desktop.rs`) so it does not collide with the CLI binary named `cubelit` in `target/release/`. Installers still use `productName` from `tauri.conf.json`.
- Release workflow `check-version` validates five Rust/npm/tauri version sources (added `crates/cli/Cargo.toml`).
- CLI macOS release binary is now pinned to `aarch64-apple-darwin` (matching the desktop job) for deterministic architecture.
- Integration test scaffold in `crates/cli/tests/integration.rs` now contains a real Docker-backed flow body (guarded by `#[ignore]`).

### Fixed
- `resolve_data_dir()` in the CLI now returns `CoreResult` instead of panicking when the platform config/data directory is unavailable.
- Default recipes directory is now derived from the same resolved data dir so `CUBELIT_DATA_DIR` affects both the SQLite DB path and the recipe seed path.
- CLI tracing writes to `cubelit.log` in the platform data dir (matching the desktop), falling back to stderr when the file is unavailable.
- `cubelit agent start` now exits with code 4 (Validation error) instead of 0, so scripts can detect the unimplemented command.
- `--port` flag removed from `agent start` (no-op flag that implied configurability).
- Log lines in `CliEventSink` now strip only `\r`/`\n` rather than all trailing whitespace, preserving meaningful trailing spaces.
- `crates/cli/Cargo.toml` and `.github/workflows/release.yml` converted from CRLF to LF; `.gitattributes` added to enforce LF repo-wide.
- Release `check-version` CI no longer false-fails due to `\r` in extracted version strings from CRLF-encoded files.

## [0.1.8] — 2026-04-25

### Changed
- Restructured the Rust backend into a Cargo workspace with a new `cubelit-core` crate housing shared business logic (error types, ports, recipes, sqlx queries + migrations + offline cache, Docker orchestration, server lifecycle, RCON / backup helpers). The `src-tauri` crate is now a thin transport layer whose Tauri IPC commands are 5–15 line shims that delegate to `cubelit-core`.
- Introduced an `EventSink` trait abstracting progress-event emission. The desktop ships `TauriEventSink`; future CLI and HTTP/WebSocket agent transports can supply their own implementation without touching core orchestration code.
- Split server orchestration into two traits: `ServerRunner` (narrow Docker runtime ops) and `ServerLifecycle` (full DB-touching lifecycle). `LocalServerHost` implements both for the v0.1.8 single-process desktop and is the seam future remote agents will replace.
- CI `paths` filter expanded to cover the new `crates/**`, `Cargo.toml`, `Cargo.lock`, and workspace `.sqlx/` directory layout.
- Frontend Tauri IPC wire format preserved byte-for-byte: every command signature, event name (`server-create-progress`, `server-status-changed`, `image-pull-progress`, etc.), and payload shape is unchanged from v0.1.7.

### Fixed
- Minecraft Java recipe now pins `default_tag` to `itzg/minecraft-server:java25` (was `java21`). Modern Minecraft releases — including vanilla `LATEST` (1.21.10+ / "26.x") and any modpack built against them — ship a Java 25 bundler (class file version 69.0) and refused to launch on the Java 21 image. The new pin runs older Minecraft versions just as well; users on pre-1.18 worlds can still pick a different image with `tag_override`.

---

## [0.1.7] — 2026-04-06

### Added
- Expanded Docker and WSL onboarding diagnostics with clearer Windows setup guidance and richer failure states
- Shared game registry for setup/dashboard selection and card styling across supported games

### Changed
- Create flow and server detail pages now dispatch game-specific setup and dashboard components through the registry
- UI controls across setup and dashboard screens received accessibility improvements including labeled inputs, explicit button types, and keyboard-friendly interactions
- GitHub release workflow now publishes non-draft releases so updater metadata is available to clients

### Fixed
- Windows onboarding polling now stays in the in-progress state while WSL commands are still running
- Keyboard activation on server cards no longer steals `Enter`/`Space` events from nested Start/Stop buttons
- Updater check failures are logged in the frontend instead of failing silently

---

## [0.1.6] — 2026-03-28

### Added
- Audits page at `/audits` — version switcher with scores, deep-linkable via `?v=X.X.X`, renders HTML audit reports inline
- Structured tracing spans in all Docker command handlers (`restart_server`, `update_server_settings`, `start_server`, `stop_server`, `delete_server`)
- Unit tests for `readiness_pattern` helper

### Changed
- `deploy-website.yml` now also triggers on pushes to `master` when `website/**` files change — website deploys without needing a new release tag
- `ci.yml` scoped to app files only (`src/**`, `src-tauri/**`, etc.)
- New `ci-website.yml` type-checks the website on `website/**` changes
- GitHub release notes now auto-populated from `CHANGELOG.md` instead of a hardcoded template

### Fixed
- `website/tsconfig.node.json`: removed `erasableSyntaxOnly` option (requires TS 5.8+, project uses 5.6.x)

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
