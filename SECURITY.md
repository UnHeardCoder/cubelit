# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in Cubelit, please open a GitHub issue and label it **"security"**. For sensitive disclosures, you may also contact the maintainers directly via the contact details on the GitHub profile.

Please include:
- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested mitigations

We aim to respond within 72 hours and resolve confirmed issues within 14 days.

## Scope

**In scope:**
- Rust backend (IPC command handlers in `src-tauri/src/commands/`)
- Tauri IPC surface (input validation, serialization)
- File system access (volume paths, recipe loading)
- SQLite database queries

**Out of scope:**
- The Docker daemon itself — Cubelit communicates with Docker via the Docker API (bollard). Security of the Docker daemon is the user's responsibility.
- The game server containers — vulnerabilities in Docker images (e.g. `itzg/minecraft-server`) should be reported to those image maintainers.
- Social engineering attacks

## Threat Model

Cubelit is a local desktop application. It does not expose any network services itself — all communication is between the Tauri frontend and the Rust backend via IPC, and between the backend and the local Docker daemon via Unix socket or named pipe.

**Environment variables**: Recipe environment variables are passed to containers via the Docker API, not through a shell. This means values cannot be used for shell injection. Cubelit validates that env var values do not exceed 4096 characters and do not contain null bytes.

**Recipe files**: Recipes are JSON files bundled with the application and loaded from the app resource directory. User-supplied recipe files are not supported.
