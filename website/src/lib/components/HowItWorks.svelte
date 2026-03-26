<script lang="ts">
  const steps = [
    {
      number: '01',
      title: 'Install',
      description: 'Download the CubeLit installer for Windows. Runs as a native desktop app — no browser required.',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`,
    },
    {
      number: '02',
      title: 'Pick a Game',
      description: 'Choose from the built-in recipe library. Each recipe comes pre-configured with sane defaults and Docker image.',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`,
    },
    {
      number: '03',
      title: 'Go Live',
      description: 'CubeLit handles Docker networking, port binding, and volume mounts. Your server is running in seconds.',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>`,
    },
  ]
</script>

<section class="hiw-section" id="how-it-works">
  <div class="container">
    <div class="section-header">
      <h2 class="section-title">Up and Running in Minutes</h2>
      <p class="section-sub">From download to live server — three steps, no expertise needed.</p>
    </div>

    <div class="timeline">
      {#each steps as step, i}
        <div class="step">
          <div class="step-number">{step.number}</div>
          <div class="step-icon">
            {@html step.icon}
          </div>
          <h3 class="step-title">{step.title}</h3>
          <p class="step-desc">{step.description}</p>
        </div>

        {#if i < steps.length - 1}
          <div class="connector">
            <div class="connector-line"></div>
            <svg class="connector-arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#f97316" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </div>
        {/if}
      {/each}
    </div>
  </div>
</section>

<style>
  .hiw-section {
    padding: 96px 24px;
    background: var(--bg-surface);
    position: relative;
  }

  .hiw-section::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(249,115,22,0.3), transparent);
  }

  .container {
    max-width: 1000px;
    margin: 0 auto;
  }

  .section-header {
    text-align: center;
    margin-bottom: 64px;
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
    margin: 0;
    line-height: 1.6;
  }

  .timeline {
    display: flex;
    align-items: flex-start;
    gap: 0;
  }

  .step {
    flex: 1;
    text-align: center;
    padding: 0 20px;
  }

  .step-number {
    width: 48px;
    height: 48px;
    background: var(--accent);
    color: white;
    font-size: 16px;
    font-weight: 800;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 16px;
    box-shadow: 0 4px 16px rgba(249,115,22,0.4);
  }

  .step-icon {
    color: var(--accent);
    display: flex;
    justify-content: center;
    margin-bottom: 16px;
  }

  .step-title {
    font-size: 18px;
    font-weight: 700;
    color: white;
    margin: 0 0 10px;
  }

  .step-desc {
    font-size: 14px;
    color: var(--text-muted);
    line-height: 1.6;
    margin: 0;
  }

  .connector {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 22px;
    flex-shrink: 0;
  }

  .connector-line {
    width: 60px;
    height: 2px;
    border-top: 2px dashed rgba(249,115,22,0.4);
  }

  .connector-arrow {
    margin-top: 4px;
  }

  @media (max-width: 700px) {
    .timeline {
      flex-direction: column;
      align-items: center;
      gap: 0;
    }

    .step {
      width: 100%;
      max-width: 360px;
      padding: 0 16px;
      margin-bottom: 8px;
    }

    .connector {
      flex-direction: row;
      padding: 0;
      margin: 8px 0;
      transform: rotate(90deg);
    }

    .connector-arrow { margin-top: 0; margin-left: 4px; }
  }
</style>
