# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Dev Commands

```bash
# Full app (frontend + Rust backend) — the primary dev workflow
bun run tauri dev

# Frontend only
bun run dev              # Vite dev server on :1420
bun run build            # Production build to ./build
bun run check            # SvelteKit sync + svelte-check (TypeScript)
bun run check:watch      # Same, in watch mode

# Rust only (from repo root — Cargo workspace)
cargo check --workspace --all-targets   # Fast type-check across both crates
cargo build --workspace                  # Full debug build
cargo clippy --workspace --all-targets   # Lints across both crates
```

Always run with `SQLX_OFFLINE=true` — the `.sqlx/` offline query cache lives in `crates/core/.sqlx/` (committed) and is required for builds without a live DB:

```bash
SQLX_OFFLINE=true cargo check --workspace --all-targets
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
SQLX_OFFLINE=true cargo test --workspace
bun run check
bun run test
```

To run a single Rust test by name:
```bash
SQLX_OFFLINE=true cargo test --workspace <test_name>
```

## Architecture

Cargo workspace with two Rust members:

- **`crates/core/`** (`cubelit-core`) — Pure business logic: error type, sqlx schema/queries/migrations, Docker helpers (health, stats), recipe loader, port utilities. No Tauri, no IPC, no UI. The `.sqlx/` offline query cache lives here.
- **`src-tauri/`** (`cubelit`) — Tauri v2 desktop binary. Owns the IPC command modules, the `AppState`, the bollard-based container/image/log streaming that emits Tauri events, and the desktop wiring. Re-exports `cubelit_core::*` through its `db/` and `docker/` modules so existing `crate::error` / `crate::db::queries` call sites keep compiling without churn.

Tauri v2 desktop app: Rust backend manages Docker containers and SQLite persistence, SvelteKit 5 frontend communicates via Tauri IPC (`invoke()`/`emit()`).

### Data flow

```
Frontend (Svelte 5)          Tauri IPC           Rust Backend
┌─────────────────┐         invoke()         ┌──────────────────┐
│  API layer       │ ──────────────────────► │  commands/*       │
│  (lib/api/*.ts)  │                         │                   │
│                  │ ◄────── Result<T> ────── │  ┌─ db/queries   │
│  Stores          │                         │  ├─ docker/*     │
│  (lib/stores/)   │ ◄────── emit() ──────── │  └─ recipes.rs   │
│                  │   (progress events)     │                   │
└─────────────────┘                          └──────────────────┘
```

### Rust backend (`src-tauri/src/`)

- **`lib.rs`** — App setup: initializes `AppState`, registers 26 IPC commands (plus 3 Windows-only), runs `sync_all_servers()` on startup to match DB status with Docker reality, and spawns a background `crash_watcher` task that periodically reconciles container state so unexpected exits flip the DB status without user action.
- **`state.rs`** — `AppState` holds `Docker` (bollard), `SqlitePool` (sqlx), `data_dir`, `recipes_dir`. Created once in `.setup()` block, injected via `app_handle.manage()`.
- **`commands/docker_commands.rs`** — Main orchestrator file. `create_server` is the workhorse: loads recipe → creates DB record → pulls image → creates container → starts it. Emits `"server-create-progress"` events at each step. Also hosts: `start_server`, `stop_server`, `restart_server`, `delete_server`, `check_docker_status`, `sync_server_status`, `sync_all_statuses`, `update_server_settings` (recreates the container with new env/ports), `get_server_stats`, `get_server_logs`, plus the `spawn_crash_watcher` background task and the internal `sync_single_server`/`sync_all_servers` helpers.
- **`commands/server_commands.rs`** — `list_cubelits`, `get_cubelit`, `rename_server` (DB-only metadata operations; no Docker calls).
- **`commands/recipe_commands.rs`** — `list_recipes`, `get_recipe_detail`.
- **`commands/system_commands.rs`** — Grab bag for non-Docker host operations: `check_port`, `suggest_port`, `get_onboarding_status` (drives the first-launch DockerOnboarding gate), `get_public_ip` (shown on the server overview so users can share an address), `open_folder` (jumps to server files in the OS file manager), and the Windows-only `check_wsl_status` / `enable_wsl2` / `set_wsl_default_version` (gated by `#[cfg(target_os = "windows")]`).
- **`commands/file_commands.rs`** — `list_server_files`, `copy_file_to_server`, `delete_server_file`. (Note: `get_server_logs` lives in `docker_commands.rs`, not here, despite the name.)
- **`commands/minecraft_commands.rs`** — `send_minecraft_command` (RCON over TCP), `backup_server` (recursive copy to timestamped dir).
- **`db/`** — Re-exports `cubelit_core::db::{models, queries, run_migrations}`. The actual schema/queries/migrations live in `crates/core/src/db/` and `crates/core/migrations/`. Single `cubelits` table, compile-time checked queries (`query!` / `query_as!`). WAL mode. DB at `{app_data_dir}/cubelit.db`. Migrations run via `sqlx::migrate!("./migrations")` from inside `cubelit-core`. The `.sqlx/` offline cache (committed) lives at `crates/core/.sqlx/` — set `SQLX_OFFLINE=true` for offline builds. To update the cache after changing queries: `DATABASE_URL=sqlite:///tmp/cubelit-dev.db sqlx migrate run --source crates/core/migrations && DATABASE_URL=sqlite:///tmp/cubelit-dev.db cargo sqlx prepare --workspace` (from the repo root).
- **`docker/containers.rs`** — Maps `Cubelit` fields to bollard `Config`. Containers named `cubelit-{id}`, labeled with `cubelit.id`/`cubelit.managed=true`, restart policy `unless-stopped`.
- **`docker/images.rs`** — Pulls images with streaming progress via Tauri events.
- **`docker/logs.rs`** — Streams container logs via Tauri events.
- **`docker/{health, stats}`** — Re-exports of `cubelit_core::docker::health` (DockerStatus + ping) and `cubelit_core::docker::stats` (resource-stats payload types). Streaming uses Tauri events from src-tauri.

Modules now living in `crates/core/src/` (consumed via re-exports):
- **`error.rs`** — `CoreError` enum with variants: `Docker`, `Database`, `Migration`, `Io`, `NotFound`, `Validation`. Serializes as a plain string for IPC transport (byte-identical to v0.1.7's `AppError`). Re-exported as `cubelit_lib::error::CoreError` for existing IPC commands.
- **`ports.rs`** — `is_port_available()` / `suggest_port()` utilities (tries up to 100 offsets from the default); used by `system_commands.rs`.
- **`recipes.rs`** — Loads JSON recipe files from `recipes_dir`, deserializes into `Recipe` structs.

### Frontend (`src/`)

- **Svelte 5 runes** — Stores use `$state` + exported getter functions (not Svelte 4 writable stores).
- **API layer** (`lib/api/`) — Thin wrappers around `invoke()`. Each file maps to a Rust command module. Vitest unit tests live alongside the modules (`docker.test.ts`, `servers.test.ts`); run with `bun run test`.
- **Stores** (`lib/stores/`) — `servers.svelte.ts`, `docker.svelte.ts`, `recipes.svelte.ts`. Stores are created via `getXxxStore()` factory functions.
- **Game registry** (`lib/games/registry.ts`) — Single source of truth for the game dispatch pattern. Maps `recipe_id` → `{ setupComponent, dashboardComponent, reviewNotes?, tileMonogram?, cardStyle? }`. The create wizard and the server detail page both call `getGameDefinition(recipeId)` and fall back to `ServerConfigForm` + `GenericDashboard` for unregistered recipes. Adding a complex game means adding an entry here in addition to creating the Svelte components.
- **Component layout** — `lib/components/` holds shared UI (Card, Button, LogViewer, StatsCards, StatusRibbon, DockerOnboarding, etc.). Game-specific Setup/Dashboard components live under `lib/components/games/<game>/`.
- **Routes**: `/` (dashboard), `/create` (3-step wizard), `/server/[id]` (detail page).
- **Layout** (`+layout.svelte`) — Sidebar with server list + Docker onboarding gate. Checks Docker on mount; if unavailable, blocks UI with `DockerOnboarding` (which on Windows surfaces the WSL2 helpers via `get_onboarding_status` + `enable_wsl2`).
- **Create wizard** (`/create`) — Step 1: pick game (recipe), Step 2: configure (dynamic form from recipe fields), Step 3: review + create. Listens for `"server-create-progress"` Tauri events during creation.

### Recipe system

JSON files in `src-tauri/recipes/` define game server templates. The `available` field gates whether a recipe appears as selectable or "Coming Soon" in the UI.

Schema:
```jsonc
{
  "id": "my-game",
  "name": "My Game",
  "description": "...",
  "icon": "my-game",
  "available": true,          // false = "Coming Soon" in UI
  "docker_image": "dockerhub/image",
  "default_tag": "latest",
  "ports": [{ "container_port": 7777, "default_host_port": 7777, "protocol": "tcp", "label": "Game Port" }],
  "environment": [
    { "key": "KEY", "default_value": "val", "label": "Label", "type": "string", "options": [] }
    // types: "string" | "number" | "boolean" | "select" | "ram"
  ],
  "volumes": [{ "container_path": "/data", "label": "Server Data" }],
  "config_files": [],
  "mods": null,               // or { "supported": true, "path": "mods", "file_types": [".jar"] }
  "estimated_disk_mb": 2000,
  "tags": ["survival"]
}
```

**Standard games** (no special backend needs): JSON recipe alone is enough. The create wizard uses the generic `ServerConfigForm` and the server detail uses `GenericDashboard`.

**Complex games** that need code changes beyond the JSON:
- **Frontend**: register the game in `src/lib/games/registry.ts` with custom `setupComponent` / `dashboardComponent` (and optional `reviewNotes`, `tileMonogram`, `cardStyle`). Currently `minecraft-java` → `MinecraftSetup`/`MinecraftDashboard` and `fivem` → `FivemSetup`/`FivemDashboard`; everything else inherits the generic fallback. Routes (`/create`, `/server/[id]`) read from this registry — do not hardcode `recipe_id` checks elsewhere.
- **Backend** (`containers.rs`): FiveM requires a MariaDB sidecar, Docker network, and filtered env vars (`FRAMEWORK`, `LICENSE_KEY` removed). Standard games use none of this.

Bundled via `tauri.conf.json` → `bundle.resources: ["recipes/*"]`. The `recipes/*` glob **requires at least one file to exist** or the build fails.

**Docker tag pinning**: Always use a specific tag in `default_tag`, never `"latest"`. For `itzg/minecraft-server` use Java-version tags (`java21`). For other images, check Docker Hub for the latest stable tag before enabling a recipe. FiveM (`spritsail/fivem`) uses date-based tags — check Docker Hub for the current stable tag before enabling.

## Key Patterns

- **Tauri v2 path API**: Use `app.path().app_data_dir()`, NOT `app.path_resolver()`.
- **Tauri v2 state injection**: `app_handle.manage(state)` inside the `.setup()` block.
- **IPC commands** return `Result<T, CoreError>` where `CoreError` (defined in `cubelit-core`, re-exported as `cubelit_lib::error::CoreError`) implements `Serialize` and serializes as a plain string. The wire format is byte-identical to v0.1.7's `AppError` — guarded by a unit test in `crates/core/src/error.rs`.
- **Long operations** (image pull, server creation) stream progress via `app_handle.emit("event-name", payload)` → frontend listens with `listen()` from `@tauri-apps/api/event`.
- **SvelteKit SPA mode**: SSR is disabled (`+layout.ts` exports `ssr = false`), static adapter with `fallback: "index.html"`.
- **`page.params.id`** can be `undefined` in SvelteKit — always guard before use.
- **Tailwind CSS v4**: Uses `@theme` directive in `app.css` for custom colors. Vite plugin via `@tailwindcss/vite`.
- **Windows `nul` file**: A `nul` file can appear in the project root on Windows; it's in `.gitignore`.

## Theme Colors

Defined in `src/app.css` via `@theme`. Use `cubelit-*` classes:
- `bg`/`surface`/`border` — background layers
- `text`/`muted` — text colors
- `accent`/`accent-hover` — primary action color (orange)
- `success`/`warning`/`error` — status colors

## Versioning

Version must be bumped in **four files simultaneously** before tagging a release, or installer filenames won't match the tag:
- `src-tauri/Cargo.toml` — `version = "x.y.z"`
- `crates/core/Cargo.toml` — `version = "x.y.z"` (kept in lockstep with the desktop crate; no workspace inheritance for now)
- `package.json` — `"version": "x.y.z"`
- `src-tauri/tauri.conf.json` — `"version": "x.y.z"`

## Logging

The app writes structured logs to `cubelit.log` in the platform app data dir:
- Linux: `~/.config/cubelit/cubelit.log`
- Windows: `%APPDATA%\cubelit\cubelit.log`
- macOS: `~/Library/Application Support/cubelit/cubelit.log`

Filter level is `warn,cubelit=info` by default. Tracing calls use `tracing::info!` / `tracing::error!` from the `tracing` crate.

## CI/CD

Four workflows in `.github/workflows/`:

- **`release.yml`** (on `v*` tag push) — runs a version check first, then builds the Tauri app for Windows, Linux, and macOS (ARM), and uploads a draft GitHub Release.
- **`deploy-website.yml`** (on `v*` tag push) — builds the website, pushes a Docker image to GHCR, and deploys to the VPS via SSH.
- **`ci.yml`** (on PR / push to `master` touching app paths) — cargo check, clippy (-D warnings), cargo test, `bun run check`, `bun run test`. All Rust steps use `SQLX_OFFLINE=true`.
- **`ci-website.yml`** (on PR / push to `master` touching `website/**`) — `bun install` + `bun run check` for the marketing site only. The website is a separate SvelteKit-less Vite + Svelte 5 SPA with its own `package.json` and its own `CLAUDE.md`.

### Version check

`release.yml` has a `check-version` job that runs before the build matrix. It strips the `v` prefix from the tag and compares it against `src-tauri/Cargo.toml`, `package.json`, and `src-tauri/tauri.conf.json`. If any file doesn't match, the entire workflow fails immediately.

### Required GitHub Secrets

| Secret | Used by |
|--------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | release.yml — signs updater artifacts |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | release.yml |
| `VPS_SSH_KEY` | deploy-website.yml — SSH key for `unheard@benlundy.com` |
| `GITHUB_TOKEN` | deploy-website.yml — GHCR login (automatic) |

### Deploying the website manually

```bash
cd website && bun run build
docker build -t ghcr.io/unheardcoder/cubelit-website:latest -f deploy/Dockerfile .
docker push ghcr.io/unheardcoder/cubelit-website:latest
ssh unheard@benlundy.com "cd /opt/stacks/cubelit && docker compose pull && docker compose up -d"
```

### CHANGELOG.md is required on every release

Before tagging a new version, `CHANGELOG.md` **must** be updated with an entry for that version. Format:

```markdown
## [x.y.z] — YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

Never tag a release with `[Unreleased]` as the only entry. The version check CI will catch mismatched versions, but CHANGELOG accuracy is a human responsibility.

## Commit Messages

Never include `Co-Authored-By:` lines or any AI attribution in commit messages.

## Git Workflow

Always create a new branch before starting any feature or fix. Never commit directly to `master`.

```bash
git checkout -b feature/<short-description>   # new feature
git checkout -b fix/<short-description>        # bug fix
git checkout -b chore/<short-description>      # maintenance / refactor
```

Once the work is done, open a PR into `master`. Keep PRs focused — one feature or fix per branch.
