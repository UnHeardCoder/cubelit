<script lang="ts">
  import { goto } from '$app/navigation';
  import { getServersStore } from '$lib/stores/servers.svelte';
  import CubelitCard from '$lib/components/CubelitCard.svelte';
  import SkeletonCard from '$lib/components/SkeletonCard.svelte';
  import Cube from '$lib/components/Cube.svelte';

  const servers = getServersStore();

  // Aggregate stats
  const total = $derived(servers.servers.length);
  const running = $derived(servers.servers.filter(s => s.status === 'running').length);

  async function handleSync() {
    await servers.load();
  }
</script>

<div class="p-8 max-w-[1200px] mx-auto">
  <!-- Page header -->
  <div class="flex items-end justify-between mb-6 gap-4 flex-wrap">
    <div>
      <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-1.5">Overview</div>
      <h1 class="text-[28px] font-semibold tracking-tight text-cubelit-text leading-none">Your servers</h1>
      <p class="text-sm text-cubelit-text-dim mt-1.5">Self-hosted Docker game servers, running on this machine.</p>
    </div>
    <div class="flex gap-2">
      <button
        type="button"
        onclick={handleSync}
        class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-sm font-medium text-cubelit-text-dim border border-cubelit-border bg-cubelit-surface hover:border-cubelit-border-2 hover:bg-cubelit-surface-2 transition-colors"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <path stroke-linecap="round" stroke-linejoin="round" d="M3 12a9 9 0 1 0 3-6.7M3 3v6h6"/>
        </svg>
        Sync
      </button>
      <a
        href="/create"
        class="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-sm font-medium text-white bg-cubelit-accent hover:brightness-110 transition-colors shadow-sm"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 5v14M5 12h14"/>
        </svg>
        New server
      </a>
    </div>
  </div>

  {#if total > 0}
    <!-- Stat strip -->
    <div class="grid grid-cols-4 gap-3 mb-6">
      {#each [
        { label: 'Servers',  value: total,                        sub: `${running} online` },
        { label: 'Running',  value: running,                      sub: 'right now' },
        { label: 'Stopped',  value: total - running,              sub: 'offline' },
        { label: 'Games',    value: new Set(servers.servers.map(s => s.recipe_id)).size, sub: 'unique' },
      ] as stat, i}
        <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-4 animate-fade-up stagger-{i + 1}">
          <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest">{stat.label}</div>
          <div class="text-2xl font-semibold tracking-tight text-cubelit-text mt-1">{stat.value}</div>
          <div class="text-xs text-cubelit-text-dim mt-0.5">{stat.sub}</div>
        </div>
      {/each}
    </div>
  {/if}

  {#if total === 0}
    <!-- Empty state -->
    <div class="flex flex-col items-center justify-center py-24">
      <div class="mb-6 opacity-40 float-cube">
        <Cube size={56} />
      </div>
      <h2 class="text-xl font-semibold text-cubelit-text mb-2">No servers yet</h2>
      <p class="text-sm text-cubelit-text-dim mb-8 max-w-sm text-center">
        Create your first game server to get started. Choose from Minecraft, FiveM, and more.
      </p>
      <a
        href="/create"
        class="inline-flex items-center gap-2 px-5 py-2.5 bg-cubelit-accent text-white rounded-lg font-medium hover:brightness-110 transition-colors shadow-sm"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15"/>
        </svg>
        Create Your First Server
      </a>
    </div>
  {:else}
    <!-- Server grid -->
    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 14px;">
      {#if servers.loading && servers.servers.length === 0}
        {#each Array(4) as _, i}
          <SkeletonCard />
        {/each}
      {:else}
      {#each servers.servers as server (server.id)}
        <CubelitCard
          {server}
          onstart={(id) => servers.start(id)}
          onstop={(id) => servers.stop(id)}
          onclick={(id) => goto(`/server/${id}`)}
        />
      {/each}
      {/if}
      <!-- New server dashed card -->
      <button
        type="button"
        onclick={() => goto('/create')}
        class="min-h-[180px] flex flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-cubelit-border-2 text-cubelit-text-dim hover:text-cubelit-text hover:border-cubelit-muted transition-colors text-sm"
      >
        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 5v14M5 12h14"/>
        </svg>
        New server
      </button>
    </div>
  {/if}
</div>
