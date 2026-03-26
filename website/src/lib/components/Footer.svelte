<script lang="ts">
  const year = new Date().getFullYear()
</script>

<footer class="footer">
  <div class="footer-main">
    <div class="footer-grid">
      <!-- Logo + tagline -->
      <div class="footer-brand">
        <a href="#hero" class="logo-link">
          <svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg" width="32" height="32">
            <polygon points="20,2 36,11 36,29 20,38 4,29 4,11" fill="rgba(249,115,22,0.12)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
            <polygon points="20,2 36,11 20,20 4,11" fill="rgba(249,115,22,0.25)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
            <polygon points="4,11 20,20 20,38 4,29" fill="rgba(249,115,22,0.38)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
            <polygon points="36,11 20,20 20,38 36,29" fill="rgba(249,115,22,0.2)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round"/>
          </svg>
          <span class="wordmark">CubeLit</span>
        </a>
        <p class="tagline">Self-host game servers with ease.</p>
      </div>

      <!-- Nav links -->
      <nav class="footer-nav">
        <div class="nav-group">
          <div class="nav-heading">Product</div>
          <a href="#features" class="nav-link">Features</a>
          <a href="#games" class="nav-link">Supported Games</a>
          <a href="#how-it-works" class="nav-link">How It Works</a>
          <a href="#download" class="nav-link">Download</a>
        </div>
        <div class="nav-group">
          <div class="nav-heading">Tech</div>
          <a href="https://tauri.app" target="_blank" rel="noopener" class="nav-link">Tauri v2</a>
          <a href="https://svelte.dev" target="_blank" rel="noopener" class="nav-link">Svelte 5</a>
          <a href="https://www.rust-lang.org" target="_blank" rel="noopener" class="nav-link">Rust</a>
          <a href="https://www.docker.com" target="_blank" rel="noopener" class="nav-link">Docker</a>
        </div>
      </nav>

      <!-- Stack credit -->
      <div class="footer-credit">
        <div class="credit-heading">Built with</div>
        <div class="credit-pills">
          <span class="credit-pill">Tauri v2</span>
          <span class="credit-pill">Svelte 5</span>
          <span class="credit-pill">Rust</span>
          <span class="credit-pill">TypeScript</span>
          <span class="credit-pill">Docker</span>
        </div>
      </div>
    </div>
  </div>

  <div class="footer-bottom">
    <div class="bottom-inner">
      <span class="copyright">&copy; {year} CubeLit. Released under the MIT License.</span>
      <a href="https://github.com/UnHeardCoder/cubelit" target="_blank" rel="noopener" class="github-link">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0 1 12 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/>
        </svg>
        GitHub
      </a>
    </div>
  </div>
</footer>

<style>
  .footer {
    background: #0a0a0c;
    border-top: 1px solid rgba(255,255,255,0.06);
  }

  .footer-main {
    padding: 64px 24px 48px;
    max-width: 1100px;
    margin: 0 auto;
  }

  .footer-grid {
    display: grid;
    grid-template-columns: 2fr 2fr 1.5fr;
    gap: 48px;
  }

  .footer-brand {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .logo-link {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
  }

  .wordmark {
    font-size: 18px;
    font-weight: 700;
    color: white;
    letter-spacing: -0.3px;
  }

  .tagline {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.5;
  }

  .footer-nav {
    display: flex;
    gap: 40px;
  }

  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .nav-heading {
    font-size: 11px;
    font-weight: 700;
    color: rgba(255,255,255,0.3);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    margin-bottom: 4px;
  }

  .nav-link {
    color: var(--text-muted);
    text-decoration: none;
    font-size: 13px;
    transition: color 0.2s ease;
  }

  .nav-link:hover { color: white; }

  .footer-credit {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .credit-heading {
    font-size: 11px;
    font-weight: 700;
    color: rgba(255,255,255,0.3);
    text-transform: uppercase;
    letter-spacing: 0.8px;
  }

  .credit-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .credit-pill {
    background: rgba(249,115,22,0.08);
    border: 1px solid rgba(249,115,22,0.15);
    color: rgba(249,115,22,0.7);
    font-size: 11px;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: 4px;
  }

  .footer-bottom {
    border-top: 1px solid rgba(255,255,255,0.05);
    padding: 20px 24px;
  }

  .bottom-inner {
    max-width: 1100px;
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .copyright {
    font-size: 12px;
    color: rgba(255,255,255,0.2);
  }

  .github-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: rgba(255,255,255,0.3);
    text-decoration: none;
    font-size: 12px;
    transition: color 0.2s ease;
  }

  .github-link:hover { color: white; }

  @media (max-width: 800px) {
    .footer-grid {
      grid-template-columns: 1fr;
      gap: 32px;
    }
    .footer-nav { justify-content: flex-start; }
  }

  @media (max-width: 480px) {
    .bottom-inner {
      flex-direction: column;
      gap: 8px;
      text-align: center;
    }
  }
</style>
