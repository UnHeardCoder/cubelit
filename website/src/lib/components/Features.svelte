<script lang="ts">
  import { onMount } from 'svelte'

  interface Feature {
    icon: string
    title: string
    description: string
  }

  const features: Feature[] = [
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14"/><path d="M5 12h14"/></svg>`,
      title: 'One-Click Creation',
      description: 'Launch a game server instantly from a curated recipe library. No config files, no manual port forwarding.',
    },
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
      title: 'Live Resource Stats',
      description: 'Real-time CPU, memory, and network usage graphs per server — straight from the Docker engine.',
    },
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>`,
      title: 'Full Lifecycle Control',
      description: 'Start, stop, restart, and delete containers with one click. View live logs in a built-in terminal pane.',
    },
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>`,
      title: 'Mod & Plugin Manager',
      description: 'Install, enable, and disable mods directly from the UI. CubeLit mounts them into the correct container paths.',
    },
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>`,
      title: 'Backup System',
      description: 'Schedule automatic backups of world data and config files. Restore any snapshot in one click.',
    },
    {
      icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
      title: 'Connection Info',
      description: 'Local IP, port, and shareable LAN link displayed automatically. No manual network scanning.',
    },
  ]

  let cardEls: HTMLElement[] = []

  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('visible')
          }
        })
      },
      { threshold: 0.1 }
    )
    cardEls.forEach((el) => { if (el) observer.observe(el) })
    return () => observer.disconnect()
  })
</script>

<section class="features-section" id="features">
  <div class="container">
    <div class="section-header">
      <h2 class="section-title">Everything You Need to Run Game Servers</h2>
      <p class="section-sub">Built for players, not sysadmins. CubeLit handles the complexity so you can focus on the game.</p>
    </div>

    <div class="features-grid">
      {#each features as feature, i}
        <div
          class="feature-card"
          style="transition-delay: {i * 80}ms"
          bind:this={cardEls[i]}
        >
          <div class="card-icon">
            {@html feature.icon}
          </div>
          <h3 class="card-title">{feature.title}</h3>
          <p class="card-desc">{feature.description}</p>
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  .features-section {
    padding: 96px 24px;
    background: var(--bg-surface);
    position: relative;
  }

  .features-section::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(249,115,22,0.3), transparent);
  }

  .container {
    max-width: 1100px;
    margin: 0 auto;
  }

  .section-header {
    text-align: center;
    margin-bottom: 60px;
  }

  .section-title {
    font-size: clamp(1.6rem, 3vw, 2.4rem);
    font-weight: 800;
    color: white;
    margin: 0 0 14px;
    letter-spacing: -0.5px;
  }

  .section-sub {
    font-size: 1.05rem;
    color: var(--text-muted);
    max-width: 540px;
    margin: 0 auto;
    line-height: 1.6;
  }

  .features-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  .feature-card {
    background: var(--bg-elevated);
    border: 1px solid rgba(255,255,255,0.06);
    border-left: 3px solid var(--accent);
    border-radius: 12px;
    padding: 28px 24px;
    cursor: default;
    opacity: 0;
    transform: translateY(16px);
    transition: opacity 0.4s ease, transform 0.4s ease;
  }

  .feature-card:global(.visible) {
    opacity: 1;
    transform: translateY(0);
  }

  .card-icon {
    color: var(--accent);
    margin-bottom: 16px;
    display: flex;
  }

  .card-title {
    font-size: 16px;
    font-weight: 700;
    color: white;
    margin: 0 0 10px;
  }

  .card-desc {
    font-size: 14px;
    color: var(--text-muted);
    line-height: 1.6;
    margin: 0;
  }

  @media (max-width: 900px) {
    .features-grid { grid-template-columns: repeat(2, 1fr); }
  }

  @media (max-width: 560px) {
    .features-grid { grid-template-columns: 1fr; }
  }
</style>
