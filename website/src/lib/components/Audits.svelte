<script lang="ts">
  import { onMount } from 'svelte'

  interface AuditEntry {
    version: string
    date: string
    score: number
    label?: string
  }

  let audits = $state<AuditEntry[]>([])
  let selectedVersion = $state('')
  let content = $state('')
  let manifestError = $state(false)
  let contentLoading = $state(false)
  let contentError = $state(false)
  let currentFetchController: AbortController | null = null

  function scoreColor(score: number): string {
    if (score >= 80) return '#22c55e'
    if (score >= 65) return '#f97316'
    return '#ef4444'
  }

  function scoreBg(score: number): string {
    if (score >= 80) return 'rgba(34,197,94,0.12)'
    if (score >= 65) return 'rgba(249,115,22,0.12)'
    return 'rgba(239,68,68,0.12)'
  }

  async function selectVersion(v: string) {
    selectedVersion = v
    const url = new URL(window.location.href)
    url.pathname = '/audits'
    url.searchParams.set('v', v)
    history.replaceState({}, '', url.toString())
    contentLoading = true
    contentError = false
    content = ''
    currentFetchController?.abort()
    const controller = new AbortController()
    currentFetchController = controller
    try {
      const res = await fetch(`/audits/v${v}.html`, { signal: controller.signal })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const scrollbarCss = `<style>
        ::-webkit-scrollbar { width: 6px; height: 6px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: rgba(100,100,100,0.4); border-radius: 999px; }
        ::-webkit-scrollbar-thumb:hover { background: rgba(100,100,100,0.65); }
        * { scrollbar-width: thin; scrollbar-color: rgba(100,100,100,0.4) transparent; }
      </style>`
      const raw = await res.text()
      const injected = raw.replace('</head>', scrollbarCss + '</head>') || raw + scrollbarCss
      if (currentFetchController === controller) content = injected
    } catch (err) {
      if (!(err instanceof DOMException && err.name === 'AbortError')) {
        contentError = true
      }
    } finally {
      if (currentFetchController === controller) contentLoading = false
    }
  }

  onMount(async () => {
    try {
      const res = await fetch('/audits/manifest.json')
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      audits = data.audits as AuditEntry[]
    } catch {
      manifestError = true
      return
    }

    if (audits.length === 0) {
      manifestError = true
      return
    }

    const params = new URLSearchParams(window.location.search)
    const qv = params.get('v')
    const initial = audits.find(a => a.version === qv) ? qv! : audits[0]?.version
    if (initial) await selectVersion(initial)
  })
</script>

<div class="audits-page">
  <div class="audits-header">
    <div class="count-badge">Security &amp; Quality</div>
    <h1 class="page-title">Audits</h1>
    <p class="page-sub">Transparency reports for every release — scored across security, code quality, and production readiness.</p>
  </div>

  {#if manifestError}
    <div class="error-state">Failed to load audit manifest. Please try again later.</div>
  {:else if audits.length === 0}
    <div class="loading-state">Loading...</div>
  {:else}
    <div class="audits-layout">
      <!-- Version sidebar -->
      <aside class="version-list">
        {#each audits as audit}
          <button
            class="version-card"
            class:selected={audit.version === selectedVersion}
            onclick={() => selectVersion(audit.version)}
          >
            <div class="version-top">
              <span class="version-num">v{audit.version}</span>
              {#if audit.label}
                <span class="version-label">{audit.label}</span>
              {/if}
            </div>
            <div class="version-bottom">
              <span class="version-date">{audit.date}</span>
              <span
                class="score-badge"
                style="color: {scoreColor(audit.score)}; background: {scoreBg(audit.score)}"
              >{audit.score}</span>
            </div>
          </button>
        {/each}
      </aside>

      <!-- Content area -->
      <div class="content-area">
        {#if contentLoading}
          <div class="content-placeholder">Loading audit...</div>
        {:else if contentError}
          <div class="content-placeholder error">Failed to load audit content.</div>
        {:else if content}
          <iframe class="audit-frame" srcdoc={content} title="Audit report for v{selectedVersion}" sandbox="allow-same-origin"></iframe>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .audits-page {
    min-height: calc(100vh - 64px);
    padding: 96px 24px 64px;
    background: var(--bg-base);
    max-width: 1200px;
    margin: 0 auto;
  }

  .audits-header {
    text-align: center;
    margin-bottom: 56px;
  }

  .count-badge {
    display: inline-block;
    background: rgba(249,115,22,0.12);
    border: 1px solid rgba(249,115,22,0.3);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
    padding: 4px 14px;
    border-radius: 999px;
    margin-bottom: 14px;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .page-title {
    font-size: clamp(1.8rem, 4vw, 3rem);
    font-weight: 800;
    color: white;
    margin: 0 0 12px;
    letter-spacing: -0.5px;
  }

  .page-sub {
    font-size: 1.05rem;
    color: var(--text-muted);
    margin: 0 auto;
    max-width: 560px;
    line-height: 1.6;
  }

  .audits-layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: 24px;
    align-items: start;
  }

  .version-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .version-card {
    background: var(--bg-surface);
    border: 1px solid rgba(255,255,255,0.06);
    border-left: 3px solid rgba(255,255,255,0.08);
    border-radius: 10px;
    padding: 14px 16px;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.2s ease, background 0.2s ease, transform 0.15s ease;
    width: 100%;
  }

  .version-card:hover {
    border-left-color: rgba(249,115,22,0.4);
    background: var(--bg-elevated);
    transform: translateX(2px);
  }

  .version-card.selected {
    border-left-color: var(--accent);
    background: var(--bg-elevated);
  }

  .version-top {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .version-num {
    font-size: 14px;
    font-weight: 700;
    color: white;
  }

  .version-label {
    font-size: 10px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(249,115,22,0.15);
    color: var(--accent);
    border: 1px solid rgba(249,115,22,0.3);
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }

  .version-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .version-date {
    font-size: 12px;
    color: var(--text-muted);
  }

  .score-badge {
    font-size: 12px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
  }

  .content-area {
    min-height: 400px;
  }

  .audit-frame {
    width: 100%;
    min-height: 800px;
    border: none;
    border-radius: 12px;
    background: white;
    display: block;
  }

  .content-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    color: var(--text-muted);
    font-size: 14px;
    background: var(--bg-surface);
    border-radius: 12px;
  }

  .content-placeholder.error {
    color: #ef4444;
  }

  .error-state,
  .loading-state {
    text-align: center;
    padding: 64px;
    color: var(--text-muted);
  }

  .error-state {
    color: #ef4444;
  }

  /* Custom scrollbar — version sidebar (desktop sticky scroll) */
  .version-list {
    scrollbar-width: thin;
    scrollbar-color: rgba(249,115,22,0.3) transparent;
  }
  .version-list::-webkit-scrollbar {
    width: 4px;
    height: 4px;
  }
  .version-list::-webkit-scrollbar-track {
    background: transparent;
  }
  .version-list::-webkit-scrollbar-thumb {
    background: rgba(249,115,22,0.3);
    border-radius: 999px;
  }
  .version-list::-webkit-scrollbar-thumb:hover {
    background: rgba(249,115,22,0.55);
  }

  @media (max-width: 768px) {
    .audits-layout {
      grid-template-columns: 1fr;
    }

    .version-list {
      flex-direction: row;
      overflow-x: auto;
      gap: 8px;
      padding-bottom: 6px;
    }

    .version-card {
      min-width: 140px;
      flex-shrink: 0;
    }
  }
</style>
