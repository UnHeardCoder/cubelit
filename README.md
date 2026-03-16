<div align="center">

# Cubelit

**Self-host game servers with ease.**

A modern desktop app that lets you create, manage, and monitor game servers through Docker — no command line needed.

[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## About

Cubelit turns Docker containers into one-click game servers. Pick a game, tweak the settings, and hit create — Cubelit handles the image pull, container configuration, port mapping, and volume management behind the scenes.

Built with a Rust backend for performance and reliability, and a Svelte 5 frontend for a snappy UI.

### Key Features

- **One-click server creation** — Choose a game, configure settings through a visual wizard, and your server is live
- **10 supported games** out of the box, with a recipe system designed for easy expansion
- **Full lifecycle management** — Start, stop, restart, and delete servers from the dashboard
- **Real-time progress** — Watch image pulls and server creation in real time via streamed events
- **Automatic state sync** — On startup, Cubelit inspects Docker to sync server statuses with reality
- **Port management** — Built-in port availability checking with auto-suggestions
- **Persistent data** — Each server gets its own volume directory that survives restarts and upgrades
- **Dark theme** — Clean, purpose-built dark UI

## Supported Games

| Game | Docker Image | Mods |
|:-----|:-------------|:----:|
| Minecraft Java | `itzg/minecraft-server` | Yes |
| Minecraft Bedrock | `itzg/minecraft-bedrock-server` | -- |
| FiveM | `spritsail/fivem` | Yes |
| Rust | `didstopia/rust-server` | Yes |
| Terraria | `ryshe/terraria` | Yes |
| Valheim | `lloesche/valheim-server` | Yes |
| ARK: Survival Evolved | `hermsi/ark-server` | Yes |
| Counter-Strike 2 | `joedwards32/cs2` | -- |
| Project Zomboid | `dandoby/project-zomboid` | Yes |
| Palworld | `thijsvanloef/palworld-server-docker` | -- |

Each game is defined as a JSON recipe in `src-tauri/recipes/`. Adding a new game is as simple as creating a new recipe file — no code changes required.

## Tech Stack

| Layer | Technology |
|:------|:-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) |
| Frontend | [SvelteKit 5](https://svelte.dev) + TypeScript |
| Styling | [Tailwind CSS v4](https://tailwindcss.com) |
| Backend | Rust |
| Docker client | [Bollard](https://github.com/fussybeaver/bollard) |
| Database | SQLite via [sqlx](https://github.com/launchbadge/sqlx) |
| Package manager | [Bun](https://bun.sh) |

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (v1.0+)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (must be running)
- Platform-specific Tauri dependencies:
  - **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), WebView2 (included in Windows 10/11)
  - **macOS:** Xcode Command Line Tools
  - **Linux:** See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/cubelit.git
cd cubelit
```

### 2. Install dependencies

```bash
bun install
```

### 3. Run in development mode

```bash
bun run tauri dev
```

This starts both the Vite dev server (frontend hot reload) and the Rust backend. The app window will open automatically.

### 4. Build for production

```bash
bun run tauri build
```

Produces platform-specific installers in `src-tauri/target/release/bundle/`.

## Development

### Useful commands

```bash
# Type-check the frontend
bun run check

# Type-check the Rust backend (fast, no full build)
cd src-tauri && cargo check

# Lint Rust code
cd src-tauri && cargo clippy

# Run the frontend Vite server standalone (without Tauri)
bun run dev
```

### Project Structure

```
src/                          # Frontend (SvelteKit)
  routes/
    +layout.svelte            # App shell — sidebar, Docker gate
    +page.svelte              # Dashboard
    create/+page.svelte       # Server creation wizard
    server/[id]/+page.svelte  # Server detail & management
  lib/
    api/                      # Tauri IPC wrappers
    stores/                   # Svelte 5 rune-based state
    components/               # Reusable UI components
    types/                    # TypeScript interfaces

src-tauri/                    # Backend (Rust)
  src/
    lib.rs                    # App entry — setup, command registration
    state.rs                  # AppState (Docker + SQLite)
    error.rs                  # Error types
    recipes.rs                # Recipe JSON loader
    ports.rs                  # Port availability utils
    db/                       # SQLite schema, models, queries
    docker/                   # Docker operations (health, containers, images, logs, stats)
    commands/                 # Tauri IPC command handlers
  recipes/                    # Game server JSON templates
  tauri.conf.json             # Tauri configuration
```

### Architecture

The frontend communicates with the Rust backend exclusively through Tauri's IPC system:

- **Commands** (`invoke()`) — Request/response for CRUD operations and actions
- **Events** (`emit()` / `listen()`) — Streaming updates for long-running operations like image pulls and server creation

All Docker operations, database access, recipe parsing, and port management happen in Rust. The frontend is a pure presentation layer with thin API wrappers and reactive stores.

### Adding a New Game

**Simple games** (standard Docker image, no special setup): only a JSON recipe is needed.

1. Create a JSON file in `src-tauri/recipes/` (e.g. `my-game.json`)
2. Follow the recipe schema:

```jsonc
{
  "id": "my-game",
  "name": "My Game",
  "description": "Short description of the game server.",
  "icon": "my-game",
  "available": true,
  "docker_image": "dockerhub/image-name",
  "default_tag": "latest",
  "ports": [
    { "container_port": 7777, "default_host_port": 7777, "protocol": "tcp", "label": "Game Port" }
  ],
  "environment": [
    { "key": "SERVER_NAME", "default_value": "My Server", "label": "Server Name", "type": "string", "options": [] },
    { "key": "MAX_PLAYERS", "default_value": "16", "label": "Max Players", "type": "number", "options": [] }
    // Supported types: "string", "number", "boolean", "select", "ram"
  ],
  "volumes": [
    { "container_path": "/data", "label": "Server Data" }
  ],
  "config_files": [],
  "mods": null,
  "estimated_disk_mb": 2000,
  "tags": ["survival"]
}
```

3. Restart the app — your game appears in the creation wizard with the generic setup form.

**Complex games** (sidecars, special env handling, custom UI): also require Svelte components (`*Setup` for the create wizard, `*Dashboard` for the server detail page) and potentially backend changes in `src-tauri/src/docker/containers.rs`. See the FiveM implementation as a reference.

## Roadmap

- [x] **Server management tabs** — Overview, mods/resources, settings, logs (Minecraft + FiveM)
- [x] **Log viewing** — Container log viewer with auto-polling (FiveM); backend streaming available
- [x] **Mod/resource management** — Upload and delete mods (Minecraft) and resources (FiveM)
- [ ] **Settings editing** — Edit and apply environment variable changes without recreating the container
- [ ] **Live stats** — CPU and memory usage panel (backend implemented, no UI yet)
- [ ] **Backup system** — Zip and restore server volumes
- [ ] **Server updates** — One-click Docker image pull + container recreate
- [ ] **System tray** — Background operation with tray icon
- [ ] **More games** — 8 recipes are stubbed (ARK, CS2, Minecraft Bedrock, Palworld, Project Zomboid, Rust, Terraria, Valheim)

## License

[MIT](LICENSE)
