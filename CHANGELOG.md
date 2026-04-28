# Changelog

All notable changes to Cubelit are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

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
