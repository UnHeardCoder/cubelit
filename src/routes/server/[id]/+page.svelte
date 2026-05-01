<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { listen } from '@tauri-apps/api/event';
  import { syncServerStatus, renameServer } from '$lib/api/servers';
  import { getGameDefinition } from '$lib/games/registry';
  import { GAME_ART, GAME_HUE } from '$lib/games/art';
  import { getServersStore } from '$lib/stores/servers.svelte';
  import Button from '$lib/components/Button.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import GameIcon from '$lib/components/GameIcon.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import type { Cubelit } from '$lib/types/server';

  const servers = getServersStore();

  let server = $state<Cubelit | null>(null);
  let loading = $state(true);
  let actionLoading = $state(false);
  let showRestartModal = $state(false);
  let editing = $state(false);
  let isSavingName = $state(false);
  let editName = $state('');
  let editNameInput = $state<HTMLInputElement | null>(null);

  const DashboardComponent = $derived(server ? getGameDefinition(server.recipe_id).dashboardComponent : null);
  const art = $derived(server ? (GAME_ART[server.recipe_id] ?? {}) : {});
  const hasHero = $derived(!!art.hero);
  const hue = $derived(server ? (GAME_HUE[server.recipe_id] ?? 30) : 30);

  // Unlisten function — stored so we can cancel when ID changes
  let statusUnlisten: (() => void) | null = null;

  // Re-load whenever the server ID in the URL changes
  $effect(() => {
    const id = page.params.id;
    if (!id) { goto('/'); return; }

    // Cancel previous status listener
    if (statusUnlisten) { statusUnlisten(); statusUnlisten = null; }

    loading = true;
    server = null;

    syncServerStatus(id)
      .then(async (s) => {
        server = s;
        // Subscribe to live status changes for this server
        statusUnlisten = await listen<string>('server-status-changed', async (event) => {
          if (server && event.payload === server.id) {
            server = await syncServerStatus(server.id);
          }
        });
      })
      .catch(() => goto('/'))
      .finally(() => { loading = false; });
  });

  onDestroy(() => {
    if (statusUnlisten) statusUnlisten();
  });

  async function startEditing() {
    if (!server) return;
    editName = server.name;
    editing = true;
    await tick();
    editNameInput?.focus();
    editNameInput?.select();
  }

  async function saveName() {
    if (!server || isSavingName) return;
    const trimmed = editName.trim();
    if (!trimmed || trimmed === server.name) { editing = false; return; }
    isSavingName = true;
    try {
      server = await renameServer(server.id, trimmed);
      await servers.load();
    } catch { /* revert */ } finally {
      isSavingName = false;
      editing = false;
    }
  }

  function cancelEdit() { editing = false; }

  async function handleStart() {
    if (!server) return;
    actionLoading = true;
    try { await servers.start(server.id); server = await syncServerStatus(server.id); }
    finally { actionLoading = false; }
  }

  async function handleStop() {
    if (!server) return;
    actionLoading = true;
    try { await servers.stop(server.id); server = await syncServerStatus(server.id); }
    finally { actionLoading = false; }
  }

  async function handleRestart() {
    if (!server) return;
    actionLoading = true;
    showRestartModal = false;
    try { await servers.restart(server.id); server = await syncServerStatus(server.id); }
    finally { actionLoading = false; }
  }
</script>

<!-- Restart Modal -->
<Modal bind:open={showRestartModal} onclose={() => { showRestartModal = false; }} title="Restart Server">
  <p class="text-sm text-cubelit-text-dim mb-4">
    Restarting <span class="text-cubelit-text font-medium">{server?.name}</span> will disconnect all active players. Continue?
  </p>
  <div class="flex gap-2 justify-end">
    <Button variant="ghost" onclick={() => { showRestartModal = false; }}>Cancel</Button>
    <Button variant="secondary" onclick={handleRestart} loading={actionLoading}>Restart</Button>
  </div>
</Modal>

{#if loading}
  <div class="flex items-center justify-center p-16">
    <Spinner size="lg" class="text-cubelit-accent" />
  </div>
{:else if server}
  <div class="p-8 max-w-[1200px] mx-auto">
    <!-- Back button -->
    <button
      type="button"
      onclick={() => goto('/')}
      class="inline-flex items-center gap-1.5 text-cubelit-text-dim hover:text-cubelit-text transition-colors text-sm mb-4"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 6l-9 6 9 6"/>
      </svg>
      Dashboard
    </button>

    <!-- Hero banner (games with art) -->
    {#if hasHero}
      <div
        class="relative rounded-2xl overflow-hidden mb-5 border border-cubelit-border"
        style="height: 180px; background-image: linear-gradient(180deg, rgba(0,0,0,0.15) 0%, rgba(0,0,0,0.78) 100%), url({art.hero}); background-size: cover; background-position: center;"
      >
        <!-- Bottom overlay with info + controls -->
        <div class="absolute bottom-0 left-0 right-0 p-4 flex items-end justify-between gap-4">
          <div class="flex items-center gap-3">
            <GameIcon recipeId={server.recipe_id} gameName={server.game} size={52} radius={12} />
            <div>
              <div class="flex items-center gap-2.5 mb-1">
                {#if editing}
                  <input
                    bind:this={editNameInput}
                    class="text-2xl font-bold text-white bg-transparent border-b border-white/50 outline-none"
                    bind:value={editName}
                    onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') cancelEdit(); }}
                    onblur={saveName}
                    style="text-shadow: 0 2px 6px rgba(0,0,0,0.6);"
                  />
                {:else}
                  <button
                    type="button"
                    class="text-2xl font-bold text-white cursor-pointer hover:opacity-80 transition-opacity bg-transparent border-0 p-0 text-left"
                    style="text-shadow: 0 2px 6px rgba(0,0,0,0.6); font-family: inherit;"
                    onclick={startEditing}
                    title="Click to rename"
                  >{server.name}</button>
                {/if}
                <StatusPill status={server.status} glass />
              </div>
              <div class="text-xs font-mono text-white/80" style="text-shadow: 0 1px 3px rgba(0,0,0,0.6);">
                {server.game} · {server.docker_image}
              </div>
            </div>
          </div>
          <!-- Action buttons -->
          <div class="flex gap-2">
            {#if server.status === 'running'}
              <button type="button" onclick={() => { showRestartModal = true; }} disabled={actionLoading}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-white/10 text-white border border-white/20 hover:bg-white/20 backdrop-blur-sm transition-colors">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M3 12a9 9 0 1 0 3-6.7M3 3v6h6"/></svg>
                Restart
              </button>
              <button type="button" onclick={handleStop} disabled={actionLoading}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-cubelit-error border border-cubelit-error/40 bg-cubelit-error/10 hover:bg-cubelit-error/20 backdrop-blur-sm transition-colors">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
                Stop
              </button>
            {:else if server.status === 'starting'}
              <button type="button" onclick={handleStop} disabled={actionLoading}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-cubelit-error border border-cubelit-error/40 bg-cubelit-error/10 hover:bg-cubelit-error/20 backdrop-blur-sm transition-colors">
                Stop
              </button>
            {:else}
              <button type="button" onclick={handleStart} disabled={actionLoading}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24"><path d="M6 4l14 8-14 8V4z"/></svg>
                Start
              </button>
            {/if}
          </div>
        </div>
      </div>

    {:else}
      <!-- Flat header (no hero art) -->
      <div class="flex items-start justify-between gap-4 mb-5 flex-wrap">
        <div class="flex items-center gap-3.5">
          <GameIcon recipeId={server.recipe_id} gameName={server.game} size={52} radius={12} />
          <div>
            <div class="flex items-center gap-2.5 mb-1">
              {#if editing}
                <input
                  bind:this={editNameInput}
                  class="text-xl font-semibold text-cubelit-text bg-transparent border-b-2 border-cubelit-accent outline-none"
                  bind:value={editName}
                  onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') cancelEdit(); }}
                  onblur={saveName}
                />
              {:else}
                <button
                  type="button"
                  class="text-xl font-semibold text-cubelit-text cursor-pointer hover:opacity-80 transition-opacity tracking-tight bg-transparent border-0 p-0 text-left"
                  style="font-family: inherit;"
                  onclick={startEditing}
                  title="Click to rename"
                >{server.name}</button>
              {/if}
              <StatusPill status={server.status} />
            </div>
            <div class="text-xs font-mono text-cubelit-text-dim">{server.game} · {server.docker_image}</div>
          </div>
        </div>
        <div class="flex gap-2">
          {#if server.status === 'running'}
            <Button variant="secondary" onclick={() => { showRestartModal = true; }} loading={actionLoading}>
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75"><path stroke-linecap="round" stroke-linejoin="round" d="M3 12a9 9 0 1 0 3-6.7M3 3v6h6"/></svg>
              Restart
            </Button>
            <Button variant="danger" onclick={handleStop} loading={actionLoading}>
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
              Stop
            </Button>
          {:else if server.status === 'starting'}
            <Button variant="danger" onclick={handleStop} loading={actionLoading}>Stop</Button>
          {:else}
            <Button onclick={handleStart} loading={actionLoading}>
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 4l14 8-14 8V4z"/></svg>
              Start
            </Button>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Dashboard component (handles tabs) -->
    {#if DashboardComponent}
      <DashboardComponent {server} />
    {/if}
  </div>
{/if}
