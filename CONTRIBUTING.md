# Contributing to Cubelit

Thanks for your interest in contributing!

## Dev Environment Setup

**Prerequisites:**

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable | https://rustup.rs |
| Bun | ≥1.0 | https://bun.sh |
| Docker Desktop | latest | https://docker.com |
| Tauri CLI v2 | bundled via `bun tauri` | — |

**On Windows:** Docker Desktop requires WSL2. Cubelit has a built-in WSL2 setup helper that runs on first launch.

**Clone and run:**
```bash
git clone <repo>
cd Cubelit
bun install
bun run tauri dev
```

## Branch Workflow

See the Git Workflow section in [CLAUDE.md](./CLAUDE.md). Never commit directly to `master`. Open a PR for every change.

## Running Tests

```bash
# Rust (from src-tauri/)
SQLX_OFFLINE=true cargo test

# TypeScript / Svelte
bun run test

# Type-check the frontend
bun run check
```

## Adding a New Game Recipe

The fastest way to add a game is a JSON-only recipe in `src-tauri/recipes/`. No code change required for standard games.

**Checklist:**
1. Copy an existing recipe as a starting point (e.g., `ark.json`).
2. Set a unique `"id"` (lowercase, hyphenated).
3. Find the right Docker image on Docker Hub. **Always use a pinned tag** — never `"latest"`. See the Docker tag pinning note in CLAUDE.md.
4. Map all required ports with correct `"protocol"` (`"tcp"` or `"udp"`).
5. List all configurable environment variables. Match the image's documented env vars exactly.
6. Set `"available": true` only after you've verified the container actually starts with the recipe config.
7. Set a reasonable `"estimated_disk_mb"` (include the image size + data).

**When a game needs code changes** (custom UI, sidecar containers, special networking):
- See the *Complex games* section in CLAUDE.md for the dispatch pattern.
- FiveM (`containers.rs`) is the reference implementation for sidecars.

**Recipe schema reference** is in CLAUDE.md under *Recipe system*.

## Updating sqlx Queries

After adding or changing a query in `src-tauri/src/db/queries.rs`:

```bash
cd src-tauri
DATABASE_URL=sqlite:///tmp/cubelit-dev.db sqlx migrate run
DATABASE_URL=sqlite:///tmp/cubelit-dev.db cargo sqlx prepare
```

Commit the updated `.sqlx/` directory alongside your query changes.

## Architecture Overview

See [CLAUDE.md](./CLAUDE.md) for a full breakdown of the Rust backend, SvelteKit frontend, IPC layer, and recipe system.

## Opening a Pull Request

- Keep PRs focused: one feature or fix per branch.
- Run `bun run check` and `SQLX_OFFLINE=true cargo check` before opening.
- Add or update tests for changed behaviour.
- Update `CHANGELOG.md` under `[Unreleased]`.
