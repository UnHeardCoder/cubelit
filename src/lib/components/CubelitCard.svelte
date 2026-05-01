<script lang="ts">
  import { getGameDefinition } from '$lib/games/registry';
  import { GAME_ART, GAME_HUE } from '$lib/games/art';
  import GameIcon from './GameIcon.svelte';
  import StatusPill from './StatusPill.svelte';
  import type { Cubelit } from '$lib/types/server';

  interface Props {
    server: Cubelit;
    onstart: (id: string) => void;
    onstop: (id: string) => void;
    onclick: (id: string) => void;
  }

  let { server, onstart, onstop, onclick }: Props = $props();

  let actionLoading = $state(false);

  const art = $derived(GAME_ART[server.recipe_id] ?? {});
  const hue = $derived(GAME_HUE[server.recipe_id] ?? 30);
  const gameDef = $derived(getGameDefinition(server.recipe_id));

  function getAddress(): string {
    try {
      const ports: Record<string, number> = JSON.parse(server.port_mappings);
      const first = Object.values(ports)[0];
      if (first) return `localhost:${first}`;
    } catch { /* ignore */ }
    return '—';
  }

  function fmtUptime(createdAt: string): string {
    // We don't have a real started_at; show created_at as a rough indicator
    return '—';
  }

  async function handleStart(e: MouseEvent) {
    e.stopPropagation();
    actionLoading = true;
    try { await onstart(server.id); } finally { actionLoading = false; }
  }

  async function handleStop(e: MouseEvent) {
    e.stopPropagation();
    actionLoading = true;
    try { await onstop(server.id); } finally { actionLoading = false; }
  }

  const isRunningOrStarting = $derived(server.status === 'running' || server.status === 'starting');
  const canStart = $derived(server.status === 'stopped' || server.status === 'created');
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="card-lift animate-fade-up bg-cubelit-surface border border-cubelit-border rounded-2xl overflow-hidden cursor-pointer hover:border-cubelit-border-2"
  onclick={() => onclick(server.id)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onclick(server.id); } }}
  role="button"
  tabindex="0"
>
  <!-- Hero band -->
  <div class="relative h-24 overflow-hidden">
    {#if art.hero}
      <div
        class="absolute inset-0"
        style="background-image: linear-gradient(180deg, rgba(0,0,0,0.2) 0%, rgba(0,0,0,0.75) 100%), url({art.hero}); background-size: cover; background-position: center;"
      ></div>
    {:else}
      <div
        class="absolute inset-0"
        style="background: linear-gradient(135deg, oklch(0.42 0.13 {hue}) 0%, var(--c-surface) 100%);"
      ></div>
    {/if}
    <!-- Header overlay -->
    <div class="absolute inset-0 p-4 flex items-start justify-between gap-3">
      <div class="flex items-center gap-3">
        <GameIcon recipeId={server.recipe_id} gameName={server.game} size={40} radius={10} />
        <div>
          <div class="text-[15px] font-semibold text-white leading-tight" style="text-shadow: 0 1px 3px rgba(0,0,0,0.5);">
            {server.name}
          </div>
          <div class="text-xs text-white/80" style="text-shadow: 0 1px 2px rgba(0,0,0,0.5);">
            {server.game}{gameDef.cardStyle?.subtitle ? ` · ${gameDef.cardStyle.subtitle}` : ''}
          </div>
        </div>
      </div>
      <div onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
        <StatusPill status={server.status} glass />
      </div>
    </div>
  </div>

  <!-- Data rows -->
  <div class="px-4 py-3 flex flex-col gap-1.5">
    <div class="flex justify-between text-xs">
      <span class="text-cubelit-muted">Address</span>
      <span class="font-mono text-cubelit-text">{getAddress()}</span>
    </div>
    <div class="flex justify-between text-xs">
      <span class="text-cubelit-muted">Status</span>
      <span class="text-cubelit-text capitalize">{server.status}</span>
    </div>
  </div>

  <!-- Footer -->
  <div class="px-4 py-2.5 border-t border-cubelit-border flex items-center justify-between">
    <div class="flex items-center gap-3 text-[11px] text-cubelit-text-dim">
      <div class="flex items-center gap-1">
        <!-- CPU icon -->
        <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <rect x="5" y="5" width="14" height="14" rx="1"/>
          <rect x="9" y="9" width="6" height="6"/>
          <path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3"/>
        </svg>
        <span>—</span>
      </div>
      <div class="flex items-center gap-1">
        <!-- Memory icon -->
        <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <rect x="3" y="8" width="18" height="9" rx="1"/>
          <path d="M7 8V5M11 8V5M15 8V5M19 8V5"/>
        </svg>
        <span>—</span>
      </div>
    </div>

    <div onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      {#if isRunningOrStarting}
        <button
          type="button"
          class="px-2.5 py-1 text-xs font-medium rounded-md text-cubelit-error border border-cubelit-error/40 bg-cubelit-error/10 hover:bg-cubelit-error/20 transition-colors disabled:opacity-50"
          onclick={handleStop}
          disabled={actionLoading}
        >
          Stop
        </button>
      {:else if canStart}
        <button
          type="button"
          class="px-2.5 py-1 text-xs font-medium rounded-md text-cubelit-accent border border-cubelit-accent/40 bg-cubelit-accent/10 hover:bg-cubelit-accent/20 transition-colors disabled:opacity-50"
          onclick={handleStart}
          disabled={actionLoading}
        >
          Start
        </button>
      {/if}
    </div>
  </div>
</div>
