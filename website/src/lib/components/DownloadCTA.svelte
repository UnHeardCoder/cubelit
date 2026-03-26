<script lang="ts">
  import { WINDOWS_URL, LINUX_URL, getOS } from '../downloads'

  const os = getOS()
</script>

<section class="cta-section" id="download">
  <div class="glow-bg" aria-hidden="true"></div>

  <div class="container">
    <div class="cta-inner">
      <div class="cta-icon" aria-hidden="true">
        <svg viewBox="0 0 56 56" fill="none" xmlns="http://www.w3.org/2000/svg" width="56" height="56">
          <polygon points="28,3 51,16 51,40 28,53 5,40 5,16" fill="rgba(249,115,22,0.12)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
          <polygon points="28,3 51,16 28,29 5,16" fill="rgba(249,115,22,0.22)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
          <polygon points="5,16 28,29 28,53 5,40" fill="rgba(249,115,22,0.32)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
          <polygon points="51,16 28,29 28,53 51,40" fill="rgba(249,115,22,0.18)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
          <line x1="28" y1="3" x2="28" y2="29" stroke="#f97316" stroke-width="1" stroke-opacity="0.5"/>
        </svg>
      </div>

      <h2 class="cta-title">Ready to run your own server?</h2>
      <p class="cta-sub">
        Download CubeLit and go from zero to live game server in under five minutes.
      </p>

      <div class="cta-buttons">
        {#if os === 'windows'}
          <a href={WINDOWS_URL} target="_blank" rel="noopener" class="btn-primary">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            Download for Windows
          </a>
        {:else if os === 'linux'}
          <a href={LINUX_URL} target="_blank" rel="noopener" class="btn-primary">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            Download for Linux
          </a>
        {:else if os === 'mac'}
          <span class="btn-unavailable">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
            </svg>
            Not available for macOS this release
          </span>
        {:else}
          <a href={WINDOWS_URL} target="_blank" rel="noopener" class="btn-primary">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            Download for Windows
          </a>
          <a href={LINUX_URL} target="_blank" rel="noopener" class="btn-primary btn-linux">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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

      <p class="fine-print">
        Requires Docker Desktop &nbsp;·&nbsp; Windows 10/11 &nbsp;·&nbsp; Free &amp; Open Source &nbsp;·&nbsp; MIT License
      </p>
    </div>
  </div>
</section>

<style>
  .cta-section {
    padding: 100px 24px;
    background: var(--bg-surface);
    position: relative;
    overflow: hidden;
  }

  .cta-section::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(249,115,22,0.4), transparent);
  }

  .glow-bg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 700px;
    height: 500px;
    background: radial-gradient(ellipse at center, rgba(249,115,22,0.15) 0%, transparent 65%);
    pointer-events: none;
    animation: glow-pulse 5s ease-in-out infinite;
  }

  .container {
    max-width: 700px;
    margin: 0 auto;
    position: relative;
    z-index: 1;
  }

  .cta-inner {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
  }

  .cta-icon {
    margin-bottom: 24px;
    animation: float 6s ease-in-out infinite;
  }

  .cta-title {
    font-size: clamp(1.8rem, 4vw, 2.8rem);
    font-weight: 800;
    color: white;
    margin: 0 0 16px;
    letter-spacing: -0.8px;
  }

  .cta-sub {
    font-size: 1.1rem;
    color: var(--text-muted);
    margin: 0 0 36px;
    line-height: 1.6;
    max-width: 480px;
  }

  .cta-buttons {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
    justify-content: center;
    margin-bottom: 24px;
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
    padding: 13px 28px;
    border-radius: 10px;
    transition: background 0.2s, transform 0.15s, box-shadow 0.2s;
    box-shadow: 0 4px 20px rgba(249,115,22,0.35);
  }

  .btn-primary:hover {
    background: var(--accent-dark);
    transform: translateY(-2px);
    box-shadow: 0 8px 30px rgba(249,115,22,0.5);
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
    padding: 13px 28px;
    border-radius: 10px;
    border: 1px solid rgba(255,255,255,0.2);
    transition: background 0.2s, border-color 0.2s, transform 0.15s;
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
    box-shadow: 0 8px 30px rgba(249,115,22,0.4);
  }

  .btn-unavailable {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.35);
    font-size: 15px;
    font-weight: 600;
    padding: 13px 28px;
    border-radius: 10px;
    border: 1px solid rgba(255,255,255,0.1);
    cursor: default;
  }

  .fine-print {
    font-size: 12px;
    color: rgba(255,255,255,0.25);
    margin: 0;
    letter-spacing: 0.2px;
  }
</style>
