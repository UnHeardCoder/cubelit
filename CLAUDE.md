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

# Rust only (from src-tauri/)
cargo check              # Fast type-check without building
cargo build              # Full debug build
cargo clippy             # Lints
```

No test suite exists yet. Verify changes with `cargo check` (Rust) and `bun run check` (TypeScript).

## Architecture

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

- **`lib.rs`** — App setup: initializes `AppState`, registers all 18 IPC commands, runs `sync_all_servers()` on startup to match DB status with Docker reality.
- **`state.rs`** — `AppState` holds `Docker` (bollard), `SqlitePool` (sqlx), `data_dir`, `recipes_dir`. Created once in `.setup()` block, injected via `app_handle.manage()`.
- **`commands/docker_commands.rs`** — `create_server` is the main orchestrator: loads recipe → creates DB record → pulls image → creates container → starts it. Emits `"server-create-progress"` events at each step. Also: `start_server`, `stop_server`, `restart_server`, `delete_server`, `check_docker_status`, `sync_server_status`, `sync_all_statuses`.
- **`commands/server_commands.rs`** — `list_cubelits`, `get_cubelit` (read-only DB queries).
- **`commands/recipe_commands.rs`** — `list_recipes`, `get_recipe_detail`.
- **`commands/system_commands.rs`** — `check_port`, `suggest_port`.
- **`commands/file_commands.rs`** — `list_server_files`, `copy_file_to_server`, `delete_server_file`, `get_server_logs`.
- **`commands/minecraft_commands.rs`** — `send_minecraft_command` (RCON over TCP), `backup_server` (recursive copy to timestamped dir).
- **`db/`** — SQLite with runtime queries (no compile-time checking). `models.rs` defines the `Cubelit` struct; `queries.rs` contains all DB operations. Single `cubelits` table. WAL mode. DB at `{app_data_dir}/cubelit.db`.
- **`docker/containers.rs`** — Maps `Cubelit` fields to bollard `Config`. Containers named `cubelit-{id}`, labeled with `cubelit.id`/`cubelit.managed=true`, restart policy `unless-stopped`.
- **`docker/images.rs`** — Pulls images with streaming progress via Tauri events.
- **`docker/health.rs`** — `check_docker_status()` helper: pings Docker and returns version info as `DockerStatus` struct (used by `check_docker_status` IPC command).
- **`docker/logs.rs`** — Streams container logs via Tauri events.
- **`docker/stats.rs`** — Streams container resource stats (CPU, memory) via Tauri events.
- **`recipes.rs`** — Loads JSON recipe files from `recipes_dir`, deserializes into `Recipe` structs.
- **`error.rs`** — `AppError` enum with variants: `Docker`, `Database`, `Io`, `NotFound`, `Validation`. Serializes as a plain string for IPC transport.
- **`ports.rs`** — `is_port_available()` / `suggest_port()` utilities (tries up to 100 offsets from the default); used by `system_commands.rs`.

### Frontend (`src/`)

- **Svelte 5 runes** — Stores use `$state` + exported getter functions (not Svelte 4 writable stores).
- **API layer** (`lib/api/`) — Thin wrappers around `invoke()`. Each file maps to a Rust command module.
- **Stores** (`lib/stores/`) — `servers.svelte.ts`, `docker.svelte.ts`, `recipes.svelte.ts`. Stores are created via `getXxxStore()` factory functions.
- **Routes**: `/` (dashboard), `/create` (3-step wizard), `/server/[id]` (detail page).
- **Layout** (`+layout.svelte`) — Sidebar with server list + Docker onboarding gate. Checks Docker on mount; if unavailable, blocks UI with `DockerOnboarding` component.
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
- **Create wizard** (`/create`): dispatches to `MinecraftSetup` or `FivemSetup` by `recipe_id`; all others fall back to `ServerConfigForm`.
- **Server detail** (`/server/[id]`): dispatches to `MinecraftDashboard` or `FivemDashboard` by `recipe_id`; all others fall back to `GenericDashboard`.
- **Backend** (`containers.rs`): FiveM requires a MariaDB sidecar, Docker network, and filtered env vars (`FRAMEWORK`, `LICENSE_KEY` removed). Standard games use none of this.

Bundled via `tauri.conf.json` → `bundle.resources: ["recipes/*"]`. The `recipes/*` glob **requires at least one file to exist** or the build fails.

## Key Patterns

- **Tauri v2 path API**: Use `app.path().app_data_dir()`, NOT `app.path_resolver()`.
- **Tauri v2 state injection**: `app_handle.manage(state)` inside the `.setup()` block.
- **IPC commands** return `Result<T, AppError>` where `AppError` implements `Serialize` (serializes as string).
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
