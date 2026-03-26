<script lang="ts">
  import CubeBackground from './CubeBackground.svelte'
  import { WINDOWS_URL, LINUX_URL, getOS } from '../downloads'

  const os = getOS()
</script>

<section class="hero" id="hero">
  <CubeBackground />

  <div class="hero-content">
    <!-- Badges -->
    <div class="pills">
      <span class="pill">Tauri v2</span>
      <span class="pill">Svelte 5</span>
      <span class="pill">Rust</span>
      <span class="pill pill-license">MIT License</span>
    </div>

    <!-- Headline -->
    <h1 class="headline">
      Self-Host Game Servers.<br />
      <span class="accent-text">No Command Line Needed.</span>
    </h1>

    <p class="subheading">
      Create, manage, and monitor game servers through Docker —<br class="hide-mobile" />
      one click, no terminal.
    </p>

    <!-- CTA Buttons -->
    <div class="cta-buttons">
      {#if os === 'windows'}
        <a href={WINDOWS_URL} target="_blank" rel="noopener" class="btn-primary">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Download for Windows
        </a>
      {:else if os === 'linux'}
        <a href={LINUX_URL} target="_blank" rel="noopener" class="btn-primary">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Download for Linux
        </a>
      {:else if os === 'mac'}
        <span class="btn-unavailable">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
          </svg>
          Not available for macOS this release
        </span>
      {:else}
        <a href={WINDOWS_URL} target="_blank" rel="noopener" class="btn-primary">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Download for Windows
        </a>
        <a href={LINUX_URL} target="_blank" rel="noopener" class="btn-primary btn-linux">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Download for Linux
        </a>
      {/if}
      <a href="https://github.com/UnHeardCoder/cubelit" target="_blank" rel="noopener" class="btn-outline">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0 1 12 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/>
        </svg>
        View on GitHub
      </a>
    </div>

    <!-- App Window Mockup -->
    <div class="app-window">
      <!-- Title bar -->
      <div class="title-bar">
        <div class="traffic-lights">
          <span class="dot red"></span>
          <span class="dot yellow"></span>
          <span class="dot green"></span>
        </div>
        <span class="window-title">CubeLit</span>
        <div style="width: 56px;"></div>
      </div>

      <!-- Window body -->
      <div class="window-body">
        <!-- Sidebar -->
        <div class="sidebar">
          <div class="sidebar-logo">
            <svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg" width="28" height="28">
              <polygon points="20,2 36,11 36,29 20,38 4,29 4,11" fill="rgba(249,115,22,0.15)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
              <polygon points="20,2 36,11 20,20 4,11" fill="rgba(249,115,22,0.3)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
              <polygon points="4,11 20,20 20,38 4,29" fill="rgba(249,115,22,0.45)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
              <polygon points="36,11 20,20 20,38 36,29" fill="rgba(249,115,22,0.25)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
            </svg>
          </div>
          <div class="sidebar-icon active">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="7" height="7"/>
              <rect x="14" y="3" width="7" height="7"/>
              <rect x="3" y="14" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/>
            </svg>
          </div>
        </div>

        <!-- Main content -->
        <div class="main-area">
          <div class="dashboard-header">
            <h2 class="dashboard-title">Dashboard</h2>
            <p class="dashboard-sub">2 servers</p>
          </div>

          <div class="server-cards">
            <!-- FiveM card -->
            <div class="server-card fivem">
              <div class="ribbon">Online</div>
              <div class="card-header">
                <div>
                  <div class="game-name">FiveM</div>
                  <div class="game-sub">GTA V Multiplayer</div>
                </div>
              </div>
              <div class="status-row">
                <span class="status-dot"></span>
                <span class="status-text">Online</span>
              </div>
              <div class="server-name-text">My FiveM Server</div>
              <div class="card-footer">
                <span class="address">localhost:30120</span>
                <button class="stop-btn">Stop</button>
              </div>
            </div>

            <!-- Minecraft card -->
            <div class="server-card minecraft">
              <div class="ribbon">Online</div>
              <div class="card-header">
                <div>
                  <div class="game-name">Minecraft Java</div>
                  <div class="game-sub">Java Edition</div>
                </div>
              </div>
              <div class="status-row">
                <span class="status-dot"></span>
                <span class="status-text">Online</span>
              </div>
              <div class="server-name-text">My Minecraft Java Server</div>
              <div class="card-footer">
                <span class="address">localhost:25565</span>
                <button class="stop-btn">Stop</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .hero {
    position: relative;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 80px 24px 60px;
    overflow: hidden;
    background: var(--bg-base);
  }

  .hero-content {
    position: relative;
    z-index: 1;
    max-width: 900px;
    width: 100%;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
    animation: fade-in-up 0.7s ease both;
  }

  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: center;
    margin-bottom: 28px;
  }

  .pill {
    background: rgba(249, 115, 22, 0.12);
    border: 1px solid rgba(249, 115, 22, 0.3);
    color: #f97316;
    font-size: 12px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 999px;
    letter-spacing: 0.3px;
  }

  .pill-license {
    background: rgba(255,255,255,0.06);
    border-color: rgba(255,255,255,0.15);
    color: var(--text-muted);
  }

  .headline {
    font-size: clamp(2rem, 5vw, 3.5rem);
    font-weight: 800;
    line-height: 1.1;
    letter-spacing: -1.5px;
    color: white;
    margin: 0 0 20px;
  }

  .accent-text {
    color: var(--accent);
  }

  .subheading {
    font-size: clamp(1rem, 2vw, 1.2rem);
    color: var(--text-muted);
    line-height: 1.6;
    margin: 0 0 36px;
  }

  .hide-mobile {
    display: inline;
  }

  .cta-buttons {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
    justify-content: center;
    margin-bottom: 56px;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: var(--accent);
    color: white;
    text-decoration: none;
    font-size: 15px;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 10px;
    transition: background 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
    box-shadow: 0 4px 20px rgba(249, 115, 22, 0.3);
  }

  .btn-primary:hover {
    background: var(--accent-dark);
    transform: translateY(-2px);
    box-shadow: 0 8px 28px rgba(249, 115, 22, 0.4);
  }

  .btn-outline {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    color: white;
    text-decoration: none;
    font-size: 15px;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 10px;
    border: 1px solid rgba(255,255,255,0.2);
    transition: background 0.2s ease, border-color 0.2s ease, transform 0.15s ease;
  }

  .btn-outline:hover {
    background: rgba(255,255,255,0.06);
    border-color: rgba(255,255,255,0.4);
    transform: translateY(-2px);
  }

  .btn-linux {
    background: rgba(249,115,22,0.7);
    box-shadow: 0 4px 20px rgba(249,115,22,0.2);
  }

  .btn-linux:hover {
    background: var(--accent-dark);
    box-shadow: 0 8px 28px rgba(249,115,22,0.35);
  }

  .btn-unavailable {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.35);
    font-size: 15px;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 10px;
    border: 1px solid rgba(255,255,255,0.1);
    cursor: default;
  }

  /* App Window */
  .app-window {
    width: 100%;
    max-width: 780px;
    background: #1c1c24;
    border-radius: 12px;
    border: 1px solid rgba(255,255,255,0.1);
    box-shadow: 0 32px 80px rgba(0,0,0,0.6), 0 0 0 1px rgba(249,115,22,0.1);
    overflow: hidden;
    text-align: left;
  }

  .title-bar {
    height: 40px;
    background: #141418;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    border-bottom: 1px solid rgba(255,255,255,0.06);
  }

  .traffic-lights {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .dot.red { background: #ff5f57; }
  .dot.yellow { background: #ffbd2e; }
  .dot.green { background: #28c840; }

  .window-title {
    font-size: 13px;
    font-weight: 500;
    color: rgba(255,255,255,0.5);
  }

  .window-body {
    display: flex;
    height: 340px;
  }

  .sidebar {
    width: 56px;
    background: #111116;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 0;
    gap: 4px;
    border-right: 1px solid rgba(255,255,255,0.06);
    flex-shrink: 0;
  }

  .sidebar-logo {
    padding: 8px;
    margin-bottom: 8px;
    background: rgba(249,115,22,0.9);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-icon {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    color: rgba(255,255,255,0.3);
    cursor: pointer;
    transition: background 0.2s, color 0.2s;
  }

  .sidebar-icon.active {
    background: rgba(249,115,22,0.15);
    color: #f97316;
  }

  .main-area {
    flex: 1;
    padding: 24px;
    overflow: auto;
    background: #1a1a22;
  }

  .dashboard-header {
    margin-bottom: 20px;
  }

  .dashboard-title {
    font-size: 22px;
    font-weight: 700;
    color: white;
    margin: 0 0 4px;
  }

  .dashboard-sub {
    font-size: 13px;
    color: rgba(255,255,255,0.4);
    margin: 0;
  }

  .server-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .server-card {
    position: relative;
    border-radius: 10px;
    padding: 16px;
    border: 1px solid rgba(255,255,255,0.07);
    overflow: hidden;
  }

  .server-card.fivem {
    background: linear-gradient(135deg, #2a1810 0%, #1e1209 100%);
    border-color: rgba(139,69,19,0.3);
  }

  .server-card.minecraft {
    background: linear-gradient(135deg, #0f2218 0%, #0a1a10 100%);
    border-color: rgba(34,85,34,0.3);
  }

  .ribbon {
    position: absolute;
    top: 12px;
    right: -18px;
    background: var(--accent);
    color: white;
    font-size: 10px;
    font-weight: 700;
    padding: 3px 22px;
    transform: rotate(35deg);
    letter-spacing: 0.5px;
    text-transform: uppercase;
    box-shadow: 0 2px 6px rgba(0,0,0,0.3);
  }

  .card-header {
    margin-bottom: 10px;
  }

  .game-name {
    font-size: 16px;
    font-weight: 700;
    color: white;
    margin-bottom: 2px;
  }

  .game-sub {
    font-size: 11px;
    color: rgba(255,255,255,0.4);
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 2px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #22c55e;
    flex-shrink: 0;
    box-shadow: 0 0 6px rgba(34,197,94,0.5);
  }

  .status-text {
    font-size: 12px;
    color: #22c55e;
    font-weight: 500;
  }

  .server-name-text {
    font-size: 11px;
    color: rgba(255,255,255,0.35);
    margin-bottom: 12px;
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-top: 1px solid rgba(255,255,255,0.06);
    padding-top: 10px;
  }

  .address {
    font-size: 11px;
    font-family: ui-monospace, monospace;
    color: rgba(255,255,255,0.35);
  }

  .stop-btn {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239,68,68,0.25);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .stop-btn:hover {
    background: rgba(239, 68, 68, 0.25);
  }

  @media (max-width: 640px) {
    .server-cards { grid-template-columns: 1fr; }
    .window-body { height: auto; }
    .main-area { padding: 16px; }
    .hide-mobile { display: none; }
  }
</style>
