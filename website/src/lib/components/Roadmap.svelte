<script lang="ts">
  import { onMount } from 'svelte'

  type Status = 'shipped' | 'building' | 'next' | 'planned' | 'later' | 'vision'
  type Persona = 'all' | 'gamer' | 'dev' | 'biz'

  interface ArchNode {
    label: string
    sub?: string
    style?: 'done' | 'highlight' | 'new' | 'default'
    col?: boolean // starts a column group
  }

  interface Phase {
    id: string
    num: string
    status: Status
    title: string
    version?: string
    progress: number
    plainEnglish: string
    personas: { gamer?: string; developer?: string; investor?: string }
    archTitle?: string
    arch?: (ArchNode | '→' | '←' | 'col-end')[]
    features: { icon: string; title: string; desc: string }[]
    faqs?: { q: string; a: string }[]
    showLiveBadge?: boolean
    pricingTable?: boolean
    bizCards?: boolean
  }

  const phases: Phase[] = [
    {
      id: 'p1',
      num: 'PHASE 01',
      status: 'shipped',
      title: 'Where we are right now',
      version: 'v0.1.7 — v0.1.8',
      progress: 100,
      showLiveBadge: true,
      plainEnglish: `Cubelit is a <strong>working desktop app</strong> for Windows, Mac, and Linux. You download it, open it, click a button, and you have a Minecraft server running on your own computer — no technical knowledge needed. Under the hood, it uses Docker to run the server safely in an isolated container. We've also done something important in the latest update: we split the brains of the app into a separate "core" that can be used by other tools we build later (like a CLI or mobile app).`,
      personas: {
        gamer: `Think of Cubelit like a control panel for your game server. Instead of typing confusing commands into a terminal, you get a clean app with buttons. Your server runs on <em>your</em> computer — you own it, you control it, there's no monthly fee.`,
        developer: `The v0.1.8 refactor extracted all business logic into <code>cubelit-core</code> (lib crate). Tauri commands are now 5–15 line shims. <code>ServerRunner</code> and <code>ServerLifecycle</code> traits are defined. <code>EventSink</code> abstraction ships with <code>TauriEventSink</code>. Cargo workspace is live. Stack: Rust + Bollard + sqlx WAL + Tokio + Svelte 5 runes + Tauri v2.`,
        investor: `We have a working product with real users and clean architecture. The v0.1.8 refactor established the foundation for a multi-client platform. Production readiness score: 88/100. Open source on GitHub. This is the point where distribution begins in earnest.`,
      },
      archTitle: 'How it works today',
      arch: [
        { label: 'You', sub: 'click buttons', style: 'done' },
        '→',
        { label: 'Cubelit App', sub: 'Tauri + Svelte 5', style: 'done' },
        '→',
        { label: 'cubelit-core', sub: 'Rust lib crate', style: 'done' },
        '→',
        { label: 'Docker', sub: 'runs the server', style: 'done' },
        '→',
        { label: 'Game Server', sub: 'Minecraft etc.', style: 'done' },
      ],
      features: [
        { icon: '🎮', title: 'Minecraft Java', desc: 'Install, start, stop, configure — one click' },
        { icon: '🐳', title: 'Docker isolation', desc: "Server runs safely, doesn't mess with your PC" },
        { icon: '🔄', title: 'Auto-updates', desc: 'App updates itself silently in the background' },
        { icon: '🪟', title: 'Windows Home support', desc: 'Works even on Windows Home edition via WSL2' },
        { icon: '📦', title: 'Modular core', desc: 'v0.1.8: backend split into reusable lib crate' },
        { icon: '🔁', title: 'CI/CD pipeline', desc: 'Automated builds and signed releases on GitHub' },
      ],
      faqs: [
        {
          q: "Why is it free?",
          a: "Cubelit is open source — the code is publicly available on GitHub. We believe game server tools should be accessible to everyone. The app will always be free to download and self-host. We'll offer paid hosted servers later for people who don't want to run it themselves.",
        },
        {
          q: "Do I need to know how to code?",
          a: "Not at all. That's the whole point. Cubelit handles all the technical stuff — Docker, networking, configuration — so you just click buttons. If a server crashes, Cubelit tells you what happened in plain English.",
        },
        {
          q: "What's Docker and why does Cubelit use it?",
          a: `Docker is a tool that runs programs in an isolated "container" — like a bubble that doesn't interfere with the rest of your computer. It means your Minecraft server is safely sandboxed, and installing or removing it never leaves a mess on your PC.`,
        },
      ],
    },
    {
      id: 'p2',
      num: 'PHASE 02',
      status: 'shipped',
      title: 'A command-line tool for power users',
      version: 'v0.1.9 — v0.1.10',
      progress: 100,
      plainEnglish: `We built a <strong>CLI (command-line interface)</strong> — a text-based tool that lets you control Cubelit by typing commands instead of clicking buttons. This is especially useful for people running servers on a <strong>remote computer or VPS</strong> that has no screen attached, where you can't open a regular app. The CLI uses the exact same backend code as the desktop app, so it's not a different product — it's just a different way to talk to the same engine.`,
      personas: {
        gamer: `Got a spare PC in your basement running headless? Want to manage your server over SSH from your laptop? The CLI lets you do exactly that. Type <code>cubelit server start minecraft</code> and it just works.`,
        developer: `CLI calls <code>cubelit-core</code> directly as a library — no HTTP layer. <code>CliEventSink</code> implements <code>EventSink</code> and prints progress to stderr. Using <code>clap</code> for argument parsing. Cross-compiled binaries shipped as GitHub release artifacts via CI. Fuzzy identifier resolver: exact UUID → name → ≥4-char UUID prefix.`,
        investor: `The CLI is the proof that the core architecture is truly decoupled. It validates the entire multi-client strategy and opens up the self-hosting market to sysadmins and VPS users — a segment that doesn't use GUIs.`,
      },
      archTitle: 'What changed in v0.1.9–v0.1.10',
      arch: [
        { label: 'Desktop App', sub: 'Tauri + Svelte 5', style: 'done', col: true },
        { label: 'CLI Tool', sub: 'new · type commands', style: 'highlight' },
        'col-end',
        '→',
        { label: 'cubelit-core', sub: 'shared by both', style: 'done' },
        '→',
        { label: 'Docker', sub: 'unchanged', style: 'done' },
        '→',
        { label: 'Game Server', style: 'done' },
      ],
      features: [
        { icon: '📟', title: 'cubelit server list', desc: 'See all servers and their status' },
        { icon: '▶️', title: 'cubelit server start', desc: 'Start a server by name' },
        { icon: '⏹️', title: 'cubelit server stop', desc: 'Gracefully stop a running server' },
        { icon: '📜', title: 'cubelit logs', desc: 'Stream live logs to your terminal' },
        { icon: '⬇️', title: 'cubelit server install', desc: 'Download and set up a new game server' },
        { icon: '🔍', title: 'Fuzzy ID resolver', desc: 'Use full UUID, name, or unique prefix' },
      ],
    },
    {
      id: 'p3',
      num: 'PHASE 03',
      status: 'next',
      title: 'A much better looking app',
      version: 'v0.2.0',
      progress: 5,
      plainEnglish: `The current desktop app works great but was built to <strong>prove the concept</strong>, not win design awards. In v0.2 we're doing a full visual redesign — a clean, polished interface that feels as good as commercial apps. The workflow: design it in Claude's design tool, then use Claude Code to wire the new UI to the stable backend. Because we built the core properly, <strong>the backend doesn't change at all</strong> — we're just swapping the face of the app.`,
      personas: {
        gamer: `The app will go from "developer tool that works" to "app you'd actually show your friends." Better onboarding, clearer status displays, easier server config — everything just looks and feels more polished.`,
        developer: `Workflow: Claude artifacts for UI mockup → Claude Code for wiring to Tauri IPC. The backend API contract (invoke commands, event names, payload shapes) is frozen from v0.1.8 so the AI wiring step is deterministic. Svelte 5 stays — no framework migration needed.`,
        investor: `This is the first public-facing milestone — the version we post on r/selfhosted, write a DEV.to article about, and show in the GitHub README GIF. First impression matters for open source adoption. V0.2 is the product we want people to discover.`,
      },
      features: [
        { icon: '🎨', title: 'Full visual redesign', desc: 'Designed first, then wired to backend' },
        { icon: '🎮', title: '3+ games in registry', desc: 'Minecraft, Valheim, Terraria minimum' },
        { icon: '🎬', title: 'README demo GIF', desc: "Show don't tell for GitHub visitors" },
        { icon: '📢', title: 'Community launch', desc: 'r/selfhosted post, DEV.to article' },
      ],
    },
    {
      id: 'p4',
      num: 'PHASE 04',
      status: 'planned',
      title: 'Control your server from anywhere',
      version: 'v0.3.0',
      progress: 2,
      plainEnglish: `Right now, Cubelit only controls servers running on the <strong>same computer</strong> the app is installed on. In v0.3 we're adding a "remote agent" — a small background program you can run on any PC (even a headless server with no screen) that <strong>exposes an API</strong> over the internet. Your desktop app or phone can then connect to it and control that remote machine as if it was local. Think of it like having remote desktop access, but just for game servers.`,
      personas: {
        gamer: `Run the server on a powerful spare PC in the other room, control it from your laptop anywhere in the house — or anywhere in the world. Start and stop your Valheim server from your phone while you're at work.`,
        developer: `<code>cubelit-agent</code>: Axum HTTP server wrapping <code>cubelit-core</code>. REST for lifecycle commands, WebSocket for log streaming and real-time events. Bearer token auth. NAT traversal via a small WebSocket relay VPS. LAN discovery via mDNS first. <code>HttpEventSink</code> implements <code>EventSink</code> — no core changes needed.`,
        investor: `The agent is the unlock for mobile, the cloud tier, and remote management. Once the HTTP API exists, every future client (mobile, web dashboard, third-party integrations) becomes a front-end concern only. This is the biggest architectural unlock in the roadmap.`,
      },
      archTitle: 'Remote control architecture',
      arch: [
        { label: 'Desktop App', style: 'done', col: true },
        { label: 'Mobile App', sub: 'new', style: 'highlight' },
        { label: 'CLI Tool', style: 'done' },
        'col-end',
        '→',
        { label: 'Relay Server', sub: 'new · routes traffic', style: 'highlight' },
        '→',
        { label: 'cubelit-agent', sub: 'new · runs on your PC', style: 'highlight' },
        '→',
        { label: 'cubelit-core', style: 'done' },
        '→',
        { label: 'Game Server', style: 'done' },
      ],
      features: [
        { icon: '🌐', title: 'HTTP REST API', desc: 'Standardised endpoints every client speaks' },
        { icon: '📡', title: 'WebSocket log streaming', desc: 'Live logs from a remote machine to your screen' },
        { icon: '🔐', title: 'Token authentication', desc: 'Generate a secret key, paste it in the app' },
        { icon: '🏠', title: 'LAN auto-discovery', desc: 'Same network? No config needed at all' },
        { icon: '⚙️', title: 'Native runner', desc: 'Run servers without Docker on bare VPS' },
        { icon: '💥', title: 'Crash auto-restart', desc: 'Agent watches and restarts fallen servers' },
      ],
    },
    {
      id: 'p5',
      num: 'PHASE 05',
      status: 'later',
      title: 'Manage servers from your phone',
      version: 'v0.4.0',
      progress: 1,
      plainEnglish: `Once the remote agent exists, adding a mobile client is mostly a front-end job. We'll start with a <strong>PWA (Progressive Web App)</strong> — a website that works like an app on your phone, no app store needed. You log in with Google, see all your machines, and control any server from anywhere. A native Android APK comes later once the experience is validated.`,
      personas: {
        gamer: `Open your phone, see "Gaming PC — Minecraft: Running — 4 players online." Tap to restart it. That's the whole experience. No app store, no account creation beyond Google login you already have.`,
        developer: `PWA first — thin client hitting the agent HTTP API. Clerk for auth (Google OAuth, JWT). Device registry in a small cloud backend (Axum + SQLite or Supabase). The mobile app has zero Rust — just a web app that talks to the agent API. Tauri v2 mobile APK as a later upgrade path.`,
        investor: `Mobile is the distribution play. Someone shows a friend "I can start my Minecraft server from my phone" — that friend downloads Cubelit. It's also the prerequisite for push notifications (player join alerts, crash alerts), which dramatically increase engagement and retention.`,
      },
      archTitle: 'How your devices connect',
      arch: [
        { label: '🖥️ Gaming PC', sub: 'runs the agent', style: 'done', col: true },
        { label: '💻 Work Laptop', sub: 'runs the agent', style: 'done' },
        'col-end',
        '→',
        { label: 'Cubelit Cloud', sub: 'device registry', style: 'highlight' },
        '←',
        { label: '📱 Your Phone', sub: 'PWA · Google login', style: 'new', col: true },
        { label: '🖥️ Desktop App', sub: 'remote host mode', style: 'done' },
        'col-end',
      ],
      features: [
        { icon: '📱', title: 'PWA — no app store', desc: 'Works on any phone, install from browser' },
        { icon: '🔑', title: 'Google login', desc: 'One account links all your devices' },
        { icon: '🖥️', title: 'Device registry', desc: '"Ben\'s Gaming Rig" appears automatically' },
        { icon: '🔔', title: 'Push notifications', desc: 'Server crashed? You\'ll know immediately' },
      ],
    },
    {
      id: 'p6',
      num: 'PHASE 06',
      status: 'vision',
      title: 'Rent a server by the hour',
      version: 'v1.x',
      progress: 0,
      plainEnglish: `Cubelit will always be <strong>free to self-host</strong>. But not everyone has a spare computer or wants to deal with setup. For them, we'll offer a one-click option to <strong>rent a server from us</strong>, billed by the hour — so you only pay when you're actually playing. We run these servers on enterprise cloud hardware, so they're fast and reliable. This is called an "open core" model — the software stays open source and free, but we offer a paid convenience tier on top.`,
      personas: {
        gamer: `Click "Rent Server," pick your game, pay $0.06 per hour. Play for 3 hours with your friends, pay $0.18 total. Stop when you're done, pay nothing while you sleep. No monthly commitment. Start and stop whenever you want right from the Cubelit app.`,
        developer: `<code>CloudServerRunner</code> implements <code>ServerRunner</code> — talks to Hetzner API instead of local Docker socket. <code>cubelit-cloud</code> crate: Stripe usage-based metering, session tracking, auto-shutdown on idle. Infrastructure: Hetzner CCX dedicated vCPU (no noisy-neighbour lag). Clerk auth carries through from mobile phase.`,
        investor: `Unit economics: Hetzner CCX23 (4vCPU/16GB) ~$28/mo. Host 2–3 Minecraft servers per VPS. Charge $0.06–$0.15/hr per server. Margin: 3–6x on infrastructure cost. Same model as Plausible, Supabase, Sentry — open source with a hosted tier. The self-hosted community is the GTM channel for the paid product.`,
      },
      pricingTable: true,
      bizCards: true,
      features: [
        { icon: '⚡', title: 'Pay per hour', desc: 'Stop playing, stop paying — no monthly bill' },
        { icon: '🏢', title: 'Enterprise hardware', desc: 'Hetzner dedicated vCPU — no noisy neighbours' },
        { icon: '🔄', title: 'Auto-shutdown', desc: 'Server pauses when no players are connected' },
        { icon: '💳', title: 'Stripe billing', desc: 'Usage-based metering with a dashboard' },
      ],
    },
    {
      id: 'p7',
      num: 'PHASE 07',
      status: 'vision',
      title: 'A platform others build on top of',
      version: 'v2.x',
      progress: 0,
      plainEnglish: `The long-term vision is for Cubelit to be the <strong>infrastructure layer for game servers</strong> — an open platform where developers can add games, build integrations, and create tools on top of a stable API. Think: Discord bots that control your server, marketplace of community-built game templates, team accounts for gaming communities, AI-assisted server setup.`,
      personas: {
        developer: `Public HTTP API with documented endpoints and SDK. Plugin system for game registry contributions. MCP server model for AI integration — user brings their own Claude subscription, one-time purchase, no per-request API costs. Potential for community-contributed runners and game templates.`,
        investor: `Platform businesses compound. Every developer who builds on Cubelit's API brings their own users. Marketplace take-rate on community templates. Team/org accounts at higher price points. API access as a paid tier. The open source moat becomes a competitive advantage — switching costs increase as users invest in the ecosystem.`,
      },
      features: [
        { icon: '🤖', title: 'AI setup assistant', desc: 'MCP server — bring your own Claude key' },
        { icon: '🔌', title: 'Public API', desc: 'Discord bots, web dashboards, third-party tools' },
        { icon: '🏢', title: 'Team accounts', desc: 'Shared servers for gaming communities' },
        { icon: '🛒', title: 'Game template marketplace', desc: 'Community-built server configs and mods' },
      ],
    },
  ]

  let activePersona = $state<Persona>('all')
  let visiblePhases = $state<Set<string>>(new Set())
  let activeDot = $state<string>('p1')
  let openFaqs = $state<Set<string>>(new Set())

  function toggleFaq(key: string) {
    const next = new Set(openFaqs)
    if (next.has(key)) {
      next.delete(key)
    } else {
      next.clear()
      next.add(key)
    }
    openFaqs = next
  }

  function scrollToPhase(id: string) {
    const el = document.getElementById(id)
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }

  function statusLabel(s: Status): string {
    switch (s) {
      case 'shipped':  return '✓ SHIPPED'
      case 'building': return '⚡ BUILDING NOW'
      case 'next':     return 'NEXT UP'
      case 'planned':  return 'PLANNED'
      case 'later':    return 'LATER'
      case 'vision':   return 'VISION'
    }
  }

  const dotLabels: Record<string, string> = {
    p1: 'Today', p2: 'CLI Tool', p3: 'New Desktop',
    p4: 'Remote Control', p5: 'Mobile', p6: 'Cloud Hosting', p7: 'Platform',
  }

  onMount(() => {
    // IntersectionObserver: fade-in-up each phase section
    const sectionObs = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            visiblePhases = new Set([...visiblePhases, e.target.id])
          }
        })
      },
      { threshold: 0.08, rootMargin: '0px 0px -5% 0px' }
    )

    // IntersectionObserver: active dot
    const navObs = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) activeDot = e.target.id
        })
      },
      { threshold: 0.35 }
    )

    phases.forEach((p) => {
      const el = document.getElementById(p.id)
      if (el) {
        sectionObs.observe(el)
        navObs.observe(el)
      }
    })

    return () => {
      sectionObs.disconnect()
      navObs.disconnect()
    }
  })
</script>

<!-- Fixed phase-dot nav rail -->
<nav class="phase-rail" aria-label="Phase navigation">
  {#each phases as phase}
    <button
      class="rail-dot"
      class:rail-dot-active={activeDot === phase.id}
      onclick={() => scrollToPhase(phase.id)}
      title={dotLabels[phase.id]}
      aria-label={dotLabels[phase.id]}
    >
      <span class="rail-label">{dotLabels[phase.id]}</span>
    </button>
  {/each}
</nav>

<div class="roadmap-page">
  <!-- Header -->
  <div class="roadmap-header">
    <div class="header-badge">Product Roadmap · 2026</div>
    <h1 class="header-title">What is <span class="accent">Cubelit</span>,<br>and where is it going?</h1>
    <p class="header-sub">Cubelit makes it easy to host your own game servers — no command line needed, no technical knowledge required. This page explains what we've built, what we're building next, and the bigger vision.</p>

    <!-- Persona picker -->
    <p class="persona-hint">I'm reading this as a...</p>
    <div class="persona-picker">
      <button class="persona-btn" class:persona-active={activePersona === 'all'} onclick={() => activePersona = 'all'}>Everyone</button>
      <button class="persona-btn" class:persona-active={activePersona === 'gamer'} onclick={() => activePersona = 'gamer'}>🎮 Gamer</button>
      <button class="persona-btn" class:persona-active={activePersona === 'dev'} onclick={() => activePersona = 'dev'}>💻 Developer</button>
      <button class="persona-btn" class:persona-active={activePersona === 'biz'} onclick={() => activePersona = 'biz'}>📈 Investor / Builder</button>
    </div>
  </div>

  <!-- Phases -->
  <div class="phases-wrap">
    {#each phases as phase, i}
      <section
        id={phase.id}
        class="phase-section"
        class:phase-visible={visiblePhases.has(phase.id)}
      >
        <!-- Eyebrow row -->
        <div class="phase-eyebrow">
          <span class="phase-num">{phase.num}</span>
          <span class="status-chip status-{phase.status}">{statusLabel(phase.status)}</span>
          {#if phase.showLiveBadge}
            <span class="live-badge"><span class="live-dot"></span>v0.1.9 live</span>
          {/if}
        </div>

        <!-- Title row -->
        <div class="phase-title-row">
          <h2 class="phase-h2">{@html phase.title}</h2>
          {#if phase.version}
            <span class="phase-version">{phase.version}</span>
          {/if}
        </div>

        <!-- Progress bar -->
        <div class="progress-bar">
          <div class="progress-fill" style="width: {phase.progress}%"></div>
        </div>

        <!-- Plain English -->
        <div class="plain-english">{@html phase.plainEnglish}</div>

        <!-- Persona callouts -->
        {#if activePersona === 'gamer' && phase.personas.gamer}
          <div class="persona-callout callout-gamer">
            <div class="callout-label">🎮 For gamers</div>
            <p>{@html phase.personas.gamer}</p>
          </div>
        {/if}
        {#if activePersona === 'dev' && phase.personas.developer}
          <div class="persona-callout callout-dev">
            <div class="callout-label">💻 For developers</div>
            <p>{@html phase.personas.developer}</p>
          </div>
        {/if}
        {#if activePersona === 'biz' && phase.personas.investor}
          <div class="persona-callout callout-biz">
            <div class="callout-label">📈 For builders</div>
            <p>{@html phase.personas.investor}</p>
          </div>
        {/if}

        <!-- Architecture diagram -->
        {#if phase.arch && phase.archTitle}
          <div class="diagram-card">
            <div class="diagram-label">{phase.archTitle}</div>
            <div class="arch-flow">
              {#each phase.arch as node}
                {#if node === '→'}
                  <span class="arch-arrow">→</span>
                {:else if node === '←'}
                  <span class="arch-arrow">←</span>
                {:else if node === 'col-end'}
                  <!-- closing col tag handled by col: true on first item -->
                {:else if typeof node === 'object' && node.col}
                  <!-- Start a column; collect siblings until col-end -->
                  {@const colItems = (() => {
                    const idx = phase.arch!.indexOf(node)
                    const items: ArchNode[] = []
                    for (let k = idx; k < phase.arch!.length; k++) {
                      const n = phase.arch![k]
                      if (n === 'col-end') break
                      if (typeof n === 'object') items.push(n as ArchNode)
                    }
                    return items
                  })()}
                  <div class="arch-col">
                    {#each colItems as colNode}
                      <div class="arch-box arch-{colNode.style ?? 'default'}">
                        {colNode.label}
                        {#if colNode.sub}<small>{colNode.sub}</small>{/if}
                      </div>
                    {/each}
                  </div>
                {:else if typeof node === 'object' && !node.col}
                  {@const prev = i > 0 ? phase.arch![phase.arch!.indexOf(node) - 1] : null}
                  {#if prev !== 'col-end' && !(typeof prev === 'object' && (prev as ArchNode).col)}
                    <div class="arch-box arch-{node.style ?? 'default'}">
                      {node.label}
                      {#if node.sub}<small>{node.sub}</small>{/if}
                    </div>
                  {/if}
                {/if}
              {/each}
            </div>
          </div>
        {/if}

        <!-- Pricing table (Phase 6 only) -->
        {#if phase.pricingTable}
          <div class="price-table">
            <div class="price-row price-header">
              <span>Game</span>
              <span>Our cost</span>
              <span>You pay</span>
              <span>Margin</span>
            </div>
            <div class="price-row"><span class="pw">🎮 Minecraft (small, ≤10 players)</span><span class="pm">~$0.008/hr</span><span class="pc">$0.06/hr</span><span class="pg">~7x</span></div>
            <div class="price-row"><span class="pw">🎮 Minecraft (modded, ≤20 players)</span><span class="pm">~$0.016/hr</span><span class="pc">$0.12/hr</span><span class="pg">~7x</span></div>
            <div class="price-row"><span class="pw">⚔️ Valheim (≤10 players)</span><span class="pm">~$0.008/hr</span><span class="pc">$0.06/hr</span><span class="pg">~7x</span></div>
            <div class="price-row"><span class="pw">🌿 Terraria (≤20 players)</span><span class="pm">~$0.005/hr</span><span class="pc">$0.03/hr</span><span class="pg">~6x</span></div>
            <div class="price-row"><span class="pw">🚗 FiveM (multi-core)</span><span class="pm">~$0.016/hr</span><span class="pc">$0.15/hr</span><span class="pg">~9x</span></div>
          </div>
        {/if}

        <!-- Biz cards (Phase 6 only) -->
        {#if phase.bizCards}
          <div class="biz-grid">
            <div class="biz-card">
              <div class="biz-card-title">The open core model</div>
              <div class="biz-row"><span class="biz-k">Desktop app</span><span class="biz-v biz-g">Free forever</span></div>
              <div class="biz-row"><span class="biz-k">CLI tool</span><span class="biz-v biz-g">Free forever</span></div>
              <div class="biz-row"><span class="biz-k">Self-hosting</span><span class="biz-v biz-g">Free forever</span></div>
              <div class="biz-row"><span class="biz-k">Source code</span><span class="biz-v biz-g">Open source (GitHub)</span></div>
              <div class="biz-row"><span class="biz-k">Hosted servers</span><span class="biz-v biz-o">Pay per hour</span></div>
            </div>
            <div class="biz-card">
              <div class="biz-card-title">6-user example scenario</div>
              <div class="biz-row"><span class="biz-k">2× small Minecraft</span><span class="biz-v">1× CCX23 VPS</span></div>
              <div class="biz-row"><span class="biz-k">4× modded Minecraft</span><span class="biz-v">1× CCX33 VPS</span></div>
              <div class="biz-row"><span class="biz-k">Monthly Hetzner cost</span><span class="biz-v biz-o">~$84/mo</span></div>
              <div class="biz-row"><span class="biz-k">Revenue (24/7 use)</span><span class="biz-v biz-g">~$432/mo</span></div>
              <div class="biz-row"><span class="biz-k">Revenue (5hrs/day)</span><span class="biz-v biz-g">~$180/mo</span></div>
            </div>
          </div>
        {/if}

        <!-- Feature grid -->
        <div class="feature-grid">
          {#each phase.features as feat}
            <div class="feature-card">
              <span class="feat-icon">{feat.icon}</span>
              <div class="feat-body">
                <div class="feat-title">{feat.title}</div>
                <div class="feat-desc">{feat.desc}</div>
              </div>
            </div>
          {/each}
        </div>

        <!-- FAQs -->
        {#if phase.faqs}
          <div class="faq-list">
            {#each phase.faqs as faq, fi}
              {@const key = `${phase.id}-${fi}`}
              <details
                class="faq-item"
                open={openFaqs.has(key)}
                ontoggle={(e) => {
                  const det = e.currentTarget as HTMLDetailsElement
                  if (det.open) {
                    const next = new Set<string>()
                    next.add(key)
                    openFaqs = next
                  } else {
                    const next = new Set(openFaqs)
                    next.delete(key)
                    openFaqs = next
                  }
                }}
              >
                <summary class="faq-q">{faq.q}<span class="faq-icon">{openFaqs.has(key) ? '−' : '+'}</span></summary>
                <div class="faq-a">{faq.a}</div>
              </details>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Connector between phases -->
      {#if i < phases.length - 1}
        <div class="phase-connector">
          <div class="connector-line"></div>
          <span class="connector-label">
            {#if phases[i + 1].status === 'shipped'}next shipped phase{:else if phases[i + 1].status === 'next'}coming next{:else if phases[i + 1].status === 'planned'}planned{:else if phases[i + 1].status === 'later'}later{:else}long term{/if}
          </span>
          <div class="connector-line"></div>
        </div>
      {/if}
    {/each}

    <!-- CTA -->
    <div class="cta-card">
      <h2 class="cta-title">Try it today — it's free</h2>
      <p class="cta-sub">Download the desktop app and have a Minecraft server running in under 5 minutes.</p>
      <div class="cta-links">
        <a href="https://github.com/UnHeardCoder/cubelit/releases" class="cta-primary">Download Cubelit</a>
        <a href="https://github.com/UnHeardCoder/cubelit" target="_blank" rel="noopener" class="cta-secondary">View on GitHub</a>
      </div>
    </div>
  </div>
</div>

<style>
  /* ── Page shell ── */
  .roadmap-page {
    min-height: calc(100vh - 64px);
    padding: 96px 24px 80px;
    background: var(--bg-base);
    max-width: 1200px;
    margin: 0 auto;
  }

  /* ── Header ── */
  .roadmap-header {
    text-align: center;
    margin-bottom: 72px;
  }

  .header-badge {
    display: inline-block;
    background: rgba(249, 115, 22, 0.12);
    border: 1px solid rgba(249, 115, 22, 0.3);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
    padding: 4px 14px;
    border-radius: 999px;
    margin-bottom: 14px;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .header-title {
    font-size: clamp(2rem, 5vw, 3.5rem);
    font-weight: 800;
    color: white;
    margin: 0 0 16px;
    letter-spacing: -0.5px;
    line-height: 1.1;
  }

  .accent {
    color: var(--accent);
  }

  .header-sub {
    font-size: 1.05rem;
    color: var(--text-muted);
    max-width: 580px;
    margin: 0 auto 32px;
    line-height: 1.65;
  }

  /* ── Persona picker ── */
  .persona-hint {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 12px;
    font-family: monospace;
  }

  .persona-picker {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .persona-btn {
    padding: 8px 18px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: var(--bg-surface);
    color: var(--text-muted);
    font-family: Inter, sans-serif;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .persona-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .persona-active {
    background: rgba(249, 115, 22, 0.12);
    border-color: var(--accent);
    color: var(--accent);
  }

  /* ── Fixed nav rail ── */
  .phase-rail {
    position: fixed;
    left: 20px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 40;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .rail-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.15);
    background: transparent;
    cursor: pointer;
    transition: all 0.2s;
    position: relative;
    padding: 0;
  }

  .rail-dot:hover,
  .rail-dot-active {
    border-color: var(--accent);
    background: var(--accent);
    box-shadow: 0 0 10px rgba(249, 115, 22, 0.5);
  }

  .rail-label {
    position: absolute;
    left: 18px;
    top: 50%;
    transform: translateY(-50%);
    font-size: 11px;
    font-family: monospace;
    color: var(--text-muted);
    white-space: nowrap;
    opacity: 0;
    transition: opacity 0.2s;
    pointer-events: none;
    background: var(--bg-elevated);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .rail-dot:hover .rail-label {
    opacity: 1;
  }

  @media (max-width: 900px) {
    .phase-rail { display: none; }
  }

  /* ── Phases container ── */
  .phases-wrap {
    max-width: 820px;
    margin: 0 auto;
  }

  /* ── Phase section ── */
  .phase-section {
    margin-bottom: 80px;
    opacity: 0;
    transform: translateY(32px);
    transition: opacity 0.55s ease, transform 0.55s ease;
  }

  .phase-visible {
    opacity: 1;
    transform: translateY(0);
  }

  /* ── Eyebrow ── */
  .phase-eyebrow {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }

  .phase-num {
    font-family: monospace;
    font-size: 11px;
    color: var(--text-muted);
    letter-spacing: 0.1em;
  }

  /* Status chips */
  .status-chip {
    font-family: monospace;
    font-size: 10px;
    padding: 3px 10px;
    border-radius: 999px;
    font-weight: 600;
    letter-spacing: 0.08em;
  }

  .status-shipped  { background: rgba(34,197,94,0.12); color: #22c55e; border: 1px solid rgba(34,197,94,0.25); }
  .status-building { background: rgba(249,115,22,0.12); color: var(--accent); border: 1px solid rgba(249,115,22,0.3); }
  .status-next     { background: rgba(96,165,250,0.12); color: #60a5fa; border: 1px solid rgba(96,165,250,0.25); }
  .status-planned  { background: rgba(167,139,250,0.12); color: #a78bfa; border: 1px solid rgba(167,139,250,0.25); }
  .status-later    { background: rgba(167,139,250,0.08); color: #a78bfa; border: 1px solid rgba(167,139,250,0.2); }
  .status-vision   { background: rgba(251,191,36,0.1); color: #fbbf24; border: 1px solid rgba(251,191,36,0.2); }

  /* Live badge */
  .live-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: monospace;
    font-size: 10px;
    color: #22c55e;
    background: rgba(34,197,94,0.08);
    border: 1px solid rgba(34,197,94,0.2);
    padding: 3px 10px;
    border-radius: 999px;
  }

  .live-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #22c55e;
    animation: pulse 2s ease infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.35; }
  }

  /* ── Title row ── */
  .phase-title-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 20px;
  }

  .phase-h2 {
    font-size: clamp(1.6rem, 3.5vw, 2.5rem);
    font-weight: 800;
    color: white;
    line-height: 1.15;
    letter-spacing: -0.3px;
    margin: 0;
  }

  .phase-version {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-muted);
    padding-top: 6px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* ── Progress bar ── */
  .progress-bar {
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 999px;
    height: 5px;
    overflow: hidden;
    margin-bottom: 24px;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, #22c55e, var(--accent));
    transition: width 1s ease;
  }

  /* ── Plain English box ── */
  .plain-english {
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-left: 3px solid var(--accent);
    border-radius: 0 10px 10px 0;
    padding: 20px 24px;
    margin-bottom: 20px;
    font-size: 15px;
    color: white;
    line-height: 1.7;
  }

  :global(.plain-english strong) { color: var(--accent); }

  /* ── Persona callouts ── */
  .persona-callout {
    background: var(--bg-elevated);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 14px 20px;
    margin-bottom: 20px;
    font-size: 13.5px;
    color: var(--text-muted);
    line-height: 1.6;
  }

  .callout-label {
    font-family: monospace;
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    margin-bottom: 6px;
    font-weight: 600;
  }

  .callout-gamer .callout-label { color: #22c55e; }
  .callout-dev .callout-label   { color: #60a5fa; }
  .callout-biz .callout-label   { color: #fbbf24; }

  :global(.persona-callout code) {
    font-family: monospace;
    font-size: 12px;
    background: rgba(255,255,255,0.06);
    padding: 1px 5px;
    border-radius: 4px;
  }

  :global(.persona-callout p) { margin: 0; }

  /* ── Diagram card ── */
  .diagram-card {
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 20px;
    overflow-x: auto;
  }

  .diagram-label {
    font-family: monospace;
    font-size: 10px;
    color: var(--text-muted);
    letter-spacing: 0.15em;
    text-transform: uppercase;
    margin-bottom: 20px;
  }

  .arch-flow {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
  }

  .arch-col {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .arch-box {
    background: var(--bg-elevated);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 13px;
    font-weight: 600;
    color: white;
    text-align: center;
    min-width: 100px;
  }

  .arch-box :global(small) {
    display: block;
    font-size: 10px;
    font-weight: 400;
    font-family: monospace;
    opacity: 0.65;
    margin-top: 2px;
  }

  .arch-done      { border-color: rgba(34,197,94,0.3); background: rgba(34,197,94,0.07); color: #22c55e; }
  .arch-highlight { border-color: rgba(249,115,22,0.4); background: rgba(249,115,22,0.1); color: var(--accent); }
  .arch-new       { border-color: rgba(96,165,250,0.3); background: rgba(96,165,250,0.08); color: #60a5fa; }
  .arch-default   { border-color: rgba(255,255,255,0.1); }

  .arch-arrow {
    color: var(--text-muted);
    font-size: 18px;
    flex-shrink: 0;
  }

  /* ── Pricing table ── */
  .price-table {
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    overflow: hidden;
    margin-bottom: 20px;
  }

  .price-row {
    display: grid;
    grid-template-columns: 1fr 100px 100px 80px;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    font-size: 13px;
    align-items: center;
  }

  .price-row:last-child { border: none; }

  .price-header {
    background: var(--bg-elevated);
    font-family: monospace;
    font-size: 10px;
    color: var(--text-muted);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .pw { font-weight: 600; color: white; }
  .pm { font-family: monospace; color: var(--text-muted); font-size: 12px; }
  .pc { font-family: monospace; color: var(--accent); font-weight: 600; }
  .pg { font-family: monospace; color: #22c55e; }

  /* ── Biz cards ── */
  .biz-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 20px;
  }

  .biz-card {
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 16px;
  }

  .biz-card-title {
    font-size: 13px;
    font-weight: 700;
    color: white;
    margin-bottom: 12px;
  }

  .biz-row {
    display: flex;
    justify-content: space-between;
    padding: 5px 0;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    font-size: 12.5px;
  }

  .biz-row:last-child { border: none; }
  .biz-k { color: var(--text-muted); }
  .biz-v { font-family: monospace; font-size: 11.5px; color: white; }
  .biz-g { color: #22c55e; }
  .biz-o { color: var(--accent); }

  /* ── Feature grid ── */
  .feature-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-bottom: 20px;
  }

  .feature-card {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 12px 14px;
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    font-size: 13.5px;
    transition: border-color 0.2s;
  }

  .feature-card:hover {
    border-color: rgba(255, 255, 255, 0.12);
  }

  .feat-icon {
    font-size: 16px;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .feat-title {
    font-weight: 600;
    color: white;
    font-size: 13px;
  }

  .feat-desc {
    color: var(--text-muted);
    font-size: 12px;
    margin-top: 2px;
    line-height: 1.4;
  }

  /* ── FAQ ── */
  .faq-list {
    margin-bottom: 20px;
  }

  .faq-item {
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }

  .faq-q {
    padding: 14px 0;
    font-weight: 600;
    font-size: 14px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: white;
    list-style: none;
    user-select: none;
    transition: color 0.2s;
  }

  .faq-q::-webkit-details-marker { display: none; }

  .faq-q:hover { color: var(--accent); }

  .faq-icon {
    color: var(--text-muted);
    font-size: 16px;
    flex-shrink: 0;
  }

  .faq-a {
    font-size: 13.5px;
    color: var(--text-muted);
    line-height: 1.7;
    padding-bottom: 14px;
  }

  /* ── Phase connector ── */
  .phase-connector {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    margin-bottom: 48px;
    color: var(--text-muted);
    font-family: monospace;
    font-size: 11px;
  }

  .connector-line {
    width: 1px;
    height: 36px;
    background: linear-gradient(rgba(255,255,255,0.12), rgba(255,255,255,0.04));
  }

  .connector-label {
    padding: 2px 10px;
    background: var(--bg-elevated);
    border-radius: 4px;
    border: 1px solid rgba(255,255,255,0.06);
  }

  /* ── CTA ── */
  .cta-card {
    text-align: center;
    padding: 56px 32px;
    background: var(--bg-surface);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 16px;
    position: relative;
    overflow: hidden;
    margin-top: 24px;
  }

  .cta-card::before {
    content: '';
    position: absolute;
    top: -60%;
    left: 50%;
    transform: translateX(-50%);
    width: 400px;
    height: 300px;
    background: radial-gradient(ellipse, rgba(249,115,22,0.07) 0%, transparent 70%);
    pointer-events: none;
  }

  .cta-title {
    font-size: 1.8rem;
    font-weight: 800;
    letter-spacing: -0.3px;
    margin: 0 0 10px;
    color: white;
  }

  .cta-sub {
    color: var(--text-muted);
    margin: 0 0 28px;
    font-size: 14px;
  }

  .cta-links {
    display: flex;
    gap: 12px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .cta-primary {
    background: var(--accent);
    color: white;
    text-decoration: none;
    font-size: 14px;
    font-weight: 600;
    padding: 10px 24px;
    border-radius: 8px;
    transition: background 0.2s, transform 0.15s;
  }

  .cta-primary:hover {
    background: var(--accent-dark);
    transform: translateY(-1px);
  }

  .cta-secondary {
    background: var(--bg-elevated);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: white;
    text-decoration: none;
    font-size: 14px;
    font-weight: 600;
    padding: 10px 24px;
    border-radius: 8px;
    transition: border-color 0.2s, color 0.2s;
  }

  .cta-secondary:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  /* ── Responsive ── */
  @media (max-width: 640px) {
    .feature-grid { grid-template-columns: 1fr; }
    .biz-grid     { grid-template-columns: 1fr; }
    .price-row    { grid-template-columns: 1fr 1fr; }
    .pm, .pg,
    .price-header span:nth-child(2),
    .price-header span:nth-child(4) { display: none; }
    .arch-flow    { flex-direction: column; }
    .phase-title-row { flex-direction: column; }
  }
</style>
