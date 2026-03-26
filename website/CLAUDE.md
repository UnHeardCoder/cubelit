# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
bun run dev        # Start dev server with HMR
bun run build      # Production build (output: dist/)
bun run preview    # Preview production build locally
bun run check      # TypeScript type-check (Svelte + Node configs)
```

## Stack

- **Svelte 5** — component framework
- **Vite** — build tool
- **TypeScript** — strict mode enabled, checks both `.ts` and `.js` files
- **Tailwind CSS v4** — via `@tailwindcss/vite` plugin (no `tailwind.config.js`; configured entirely in `app.css`)

## Architecture

Single-page landing site. Entry: `index.html` → `src/main.ts` → `src/App.svelte`.

`App.svelte` composes all section components in order from `src/lib/components/`:
`Navbar` → `Hero` → `Features` → `SupportedGames` → `HowItWorks` → `TechStack` → `DownloadCTA` → `Footer`

Global styles live in `src/app.css`, which imports Tailwind and defines CSS custom properties (dark theme: `#0d0d0f` base, `#f97316` orange accent) plus keyframe animations (`float`, `glow-pulse`, `fade-in-up`, `slide-down`).

`CubeBackground.svelte` is a standalone animated background component used in `Hero`.
