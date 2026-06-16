# Cubelit Website

Marketing and documentation website for Cubelit. The site is a Vite + Svelte SPA served separately from the Tauri desktop app.

## Local Development

```bash
bun install
bun run dev
bun run check
bun run build
```

## Environment Variables

- `VITE_DOWNLOAD_WINDOWS_URL` - direct Windows installer URL.
- `VITE_DOWNLOAD_LINUX_URL` - direct Linux AppImage URL.

See `.env.example` for release URL templates.

## Deployment

The website is built by `.github/workflows/deploy-website.yml` on release tags. The workflow builds the static site, publishes a GHCR image, and deploys the image to the VPS with Docker Compose.

## Release Audits

Each release audit has two parts:

1. Add the HTML report at `website/public/audits/vX.Y.Z.html`.
2. Add the release entry to `website/public/audits/manifest.json`.

The newest entry should appear first in the manifest. Use the `Latest` label only for the current release.
