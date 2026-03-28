<script lang="ts">
  import { onMount } from 'svelte'

  let { navigate, currentPage }: { navigate: (path: string) => void; currentPage: string } = $props()

  let scrolled = $state(false)
  let mobileOpen = $state(false)

  onMount(() => {
    const onScroll = () => {
      scrolled = window.scrollY > 20
    }
    window.addEventListener('scroll', onScroll, { passive: true })
    return () => window.removeEventListener('scroll', onScroll)
  })

  function toggleMobile() {
    mobileOpen = !mobileOpen
  }

  function closeMobile() {
    mobileOpen = false
  }

  function goTo(path: string) {
    closeMobile()
    navigate(path)
  }
</script>

<nav class="nav" class:scrolled>
  <div class="nav-inner">
    <!-- Logo -->
    <button class="logo-link" onclick={() => goTo('/')}>
      <svg class="logo-icon" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <polygon points="20,2 36,11 36,29 20,38 4,29 4,11" fill="rgba(249,115,22,0.15)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
        <polygon points="20,2 36,11 20,20 4,11" fill="rgba(249,115,22,0.25)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
        <polygon points="4,11 20,20 20,38 4,29" fill="rgba(249,115,22,0.35)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
        <polygon points="36,11 20,20 20,38 36,29" fill="rgba(249,115,22,0.2)" stroke="#f97316" stroke-width="1.5" stroke-linejoin="round" />
        <line x1="20" y1="2" x2="20" y2="20" stroke="#f97316" stroke-width="1" stroke-opacity="0.6" />
      </svg>
      <span class="wordmark">CubeLit</span>
    </button>

    <!-- Desktop nav -->
    <div class="desktop-nav">
      {#if currentPage === 'home'}
        <a href="#features" class="nav-link">Features</a>
        <a href="#games" class="nav-link">Games</a>
        <a href="#how-it-works" class="nav-link">How It Works</a>
      {/if}
      <button class="nav-link" class:nav-link-active={currentPage === 'audits'} onclick={() => goTo('/audits')}>Audits</button>
      <a href="https://github.com/UnHeardCoder/cubelit" target="_blank" rel="noopener" class="nav-link">GitHub</a>
      {#if currentPage === 'home'}
        <a href="#download" class="btn-download">Download</a>
      {:else}
        <button class="btn-download" onclick={() => goTo('/')}>Download</button>
      {/if}
    </div>

    <!-- Mobile hamburger -->
    <button class="hamburger" onclick={toggleMobile} aria-label="Toggle menu" aria-expanded={mobileOpen}>
      <span class="bar" class:open={mobileOpen}></span>
      <span class="bar" class:open={mobileOpen}></span>
      <span class="bar" class:open={mobileOpen}></span>
    </button>
  </div>

  <!-- Mobile drawer -->
  {#if mobileOpen}
    <div class="mobile-drawer">
      {#if currentPage === 'home'}
        <a href="#features" class="mobile-link" onclick={closeMobile}>Features</a>
        <a href="#games" class="mobile-link" onclick={closeMobile}>Games</a>
        <a href="#how-it-works" class="mobile-link" onclick={closeMobile}>How It Works</a>
      {/if}
      <button class="mobile-link" class:mobile-link-active={currentPage === 'audits'} onclick={() => goTo('/audits')}>Audits</button>
      <a href="https://github.com/UnHeardCoder/cubelit" target="_blank" rel="noopener" class="mobile-link" onclick={closeMobile}>GitHub</a>
      {#if currentPage === 'home'}
        <a href="#download" class="mobile-btn" onclick={closeMobile}>Download</a>
      {:else}
        <button class="mobile-btn" onclick={() => goTo('/')}>Download</button>
      {/if}
    </div>
  {/if}
</nav>

<style>
  .nav {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
    transition: background 0.3s ease, border-color 0.3s ease, backdrop-filter 0.3s ease;
    border-bottom: 1px solid transparent;
  }

  .nav.scrolled {
    background: rgba(13, 13, 15, 0.85);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom-color: rgba(249, 115, 22, 0.15);
  }

  .nav-inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 24px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo-link {
    display: flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .logo-icon {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
  }

  .wordmark {
    font-size: 18px;
    font-weight: 700;
    color: white;
    letter-spacing: -0.3px;
  }

  .desktop-nav {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .nav-link {
    color: var(--text-muted);
    text-decoration: none;
    font-size: 14px;
    font-weight: 500;
    padding: 6px 12px;
    border-radius: 6px;
    transition: color 0.2s ease;
    background: none;
    border: none;
    cursor: pointer;
  }

  .nav-link:hover {
    color: white;
  }

  .nav-link-active {
    color: var(--accent);
  }

  .nav-link-active:hover {
    color: var(--accent);
  }

  .btn-download {
    background: var(--accent);
    color: white;
    text-decoration: none;
    font-size: 14px;
    font-weight: 600;
    padding: 8px 18px;
    border-radius: 8px;
    margin-left: 8px;
    transition: background 0.2s ease, transform 0.15s ease;
    border: none;
    cursor: pointer;
  }

  .btn-download:hover {
    background: var(--accent-dark);
    transform: translateY(-1px);
  }

  .hamburger {
    display: none;
    flex-direction: column;
    gap: 5px;
    cursor: pointer;
    background: none;
    border: none;
    padding: 4px;
  }

  .bar {
    display: block;
    width: 22px;
    height: 2px;
    background: white;
    border-radius: 2px;
    transition: transform 0.25s ease, opacity 0.25s ease;
  }

  .bar:nth-child(1).open { transform: translateY(7px) rotate(45deg); }
  .bar:nth-child(2).open { opacity: 0; }
  .bar:nth-child(3).open { transform: translateY(-7px) rotate(-45deg); }

  .mobile-drawer {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 16px 20px;
    background: rgba(13, 13, 15, 0.97);
    border-top: 1px solid rgba(249, 115, 22, 0.1);
    animation: slide-down 0.2s ease;
  }

  .mobile-link {
    color: var(--text-muted);
    text-decoration: none;
    font-size: 16px;
    font-weight: 500;
    padding: 12px 8px;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    transition: color 0.2s ease;
    background: none;
    border: none;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    cursor: pointer;
    text-align: left;
  }

  .mobile-link:hover { color: white; }

  .mobile-link-active { color: var(--accent); }

  .mobile-btn {
    display: block;
    margin-top: 12px;
    background: var(--accent);
    color: white;
    text-decoration: none;
    font-size: 15px;
    font-weight: 600;
    padding: 12px 18px;
    border-radius: 8px;
    text-align: center;
    transition: background 0.2s ease;
    border: none;
    cursor: pointer;
    width: 100%;
  }

  .mobile-btn:hover { background: var(--accent-dark); }

  @media (max-width: 768px) {
    .desktop-nav { display: none; }
    .hamburger { display: flex; }
  }
</style>
