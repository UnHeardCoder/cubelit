<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getDockerStore } from '$lib/stores/docker.svelte';
  import { getServersStore } from '$lib/stores/servers.svelte';
  import { getThemeStore } from '$lib/stores/theme.svelte';
  import DockerOnboarding from '$lib/components/DockerOnboarding.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import Cube from '$lib/components/Cube.svelte';
  import GameIcon from '$lib/components/GameIcon.svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';

  let { children } = $props();

  const docker = getDockerStore();
  const servers = getServersStore();
  const themeStore = getThemeStore();

  let initialized = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let updateAvailable = $state<{ version: string; download: () => Promise<void> } | null>(null);
  let updateLoading = $state(false);

  // Sidebar collapse state — persisted to localStorage
  let sidebarMode = $state<'expanded' | 'icon'>('expanded');

  function loadSidebarMode() {
    try {
      const raw = localStorage.getItem('cubelit-sidebar');
      if (raw === 'icon' || raw === 'expanded') sidebarMode = raw;
    } catch { /* ignore */ }
  }

  function saveSidebarMode(m: 'expanded' | 'icon') {
    sidebarMode = m;
    try { localStorage.setItem('cubelit-sidebar', m); } catch { /* ignore */ }
  }

  const iconMode = $derived(sidebarMode === 'icon');
  const isCreatePage = $derived(page.url.pathname === '/create');
  const isDashboard = $derived(page.url.pathname === '/');

  function startRefreshLoop() {
    if (refreshInterval !== null) return;
    refreshInterval = setInterval(() => servers.load(), 30_000);
  }

  onMount(async () => {
    themeStore.init();
    loadSidebarMode();
    await docker.check();
    if (docker.status.available) {
      await servers.load();
      startRefreshLoop();
    }
    initialized = true;

    try {
      const update = await check();
      if (update?.available) {
        updateAvailable = {
          version: update.version,
          download: async () => {
            updateLoading = true;
            await update.downloadAndInstall();
            await relaunch();
          },
        };
      }
    } catch (e) {
      console.error('Failed to check for app updates:', e);
    }
  });

  onDestroy(() => {
    if (refreshInterval !== null) clearInterval(refreshInterval);
  });

  function handleDockerCheck() {
    docker.check().then(async () => {
      if (docker.status.available) {
        await servers.load();
        startRefreshLoop();
      }
    });
  }

  function statusClass(status: string): string {
    return status === 'running' ? 'running' : status === 'starting' ? 'starting' : status === 'error' ? 'error' : 'stopped';
  }

  function isServerActive(id: string): boolean {
    return page.url.pathname === `/server/${id}`;
  }
</script>

{#if !initialized}
  <div class="min-h-screen bg-cubelit-bg flex items-center justify-center">
    <div class="text-center">
      <Spinner size="lg" class="text-cubelit-accent mx-auto mb-4" />
      <p class="text-cubelit-muted text-sm">Loading Cubelit...</p>
    </div>
  </div>
{:else if !docker.status.available}
  <DockerOnboarding
    status={docker.onboarding}
    statusError={docker.status.error}
    checking={docker.checking}
    oncheck={handleDockerCheck}
  />
{:else}
  <div
    class="min-h-screen bg-cubelit-bg text-cubelit-text flex"
    style="display: grid; grid-template-columns: {iconMode ? '64px' : '240px'} 1fr; height: 100vh; transition: grid-template-columns 0.25s ease;"
  >
    <!-- ── Sidebar ── -->
    <aside
      class="h-screen border-r border-cubelit-border bg-cubelit-bg-2 flex flex-col overflow-hidden"
      style="padding: 14px {iconMode ? '8px' : '12px'}; gap: 14px;"
    >
      <!-- Brand row -->
      <div class="flex items-center {iconMode ? 'justify-center' : 'justify-between'} gap-2 px-1">
        <div class="flex items-center gap-2.5">
          <div class="w-9 h-9 flex items-center justify-center shrink-0">
            <Cube size={32} />
          </div>
          {#if !iconMode}
            <div class="leading-tight min-w-0">
              <div class="text-[13px] font-semibold tracking-tight text-cubelit-text">cubelit</div>
              <div class="text-[10px] text-cubelit-muted font-mono">v2.0.0</div>
            </div>
          {/if}
        </div>
        {#if !iconMode}
          <!-- Collapse button -->
          <button
            type="button"
            class="w-7 h-7 rounded-lg flex items-center justify-center text-cubelit-muted hover:text-cubelit-text hover:bg-cubelit-surface transition-colors shrink-0"
            title="Collapse sidebar"
            onclick={() => saveSidebarMode('icon')}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M15 6l-9 6 9 6" />
            </svg>
          </button>
        {/if}
      </div>

      {#if iconMode}
        <!-- Expand button -->
        <button
          type="button"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-cubelit-muted hover:text-cubelit-text hover:bg-cubelit-surface transition-colors self-center"
          title="Expand sidebar"
          onclick={() => saveSidebarMode('expanded')}
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 6l9 6-9 6" />
          </svg>
        </button>
      {/if}

      <!-- Primary nav -->
      <nav class="flex flex-col gap-0.5">
        <!-- Dashboard -->
        <button
          type="button"
          onclick={() => goto('/')}
          title={iconMode ? 'Dashboard' : undefined}
          class="flex items-center gap-2.5 rounded-lg transition-colors text-[13px] font-medium
            {iconMode ? 'justify-center p-2.5' : 'px-2.5 py-2'}
            {isDashboard
              ? 'bg-cubelit-accent/15 text-cubelit-accent'
              : 'text-cubelit-text-dim hover:text-cubelit-text hover:bg-cubelit-surface'}"
        >
          <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 11l9-8 9 8M5 10v10h5v-6h4v6h5V10" />
          </svg>
          {#if !iconMode}<span>Dashboard</span>{/if}
        </button>

        <!-- New server -->
        <button
          type="button"
          onclick={() => goto('/create')}
          title={iconMode ? 'New server' : undefined}
          class="flex items-center gap-2.5 rounded-lg transition-colors text-[13px] font-medium
            {iconMode ? 'justify-center p-2.5' : 'px-2.5 py-2'}
            {isCreatePage
              ? 'bg-cubelit-accent/15 text-cubelit-accent'
              : 'text-cubelit-text-dim hover:text-cubelit-text hover:bg-cubelit-surface'}"
        >
          <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 5v14M5 12h14" />
          </svg>
          {#if !iconMode}<span>New server</span>{/if}
        </button>

        <!-- Settings -->
        <button
          type="button"
          onclick={() => goto('/settings')}
          title={iconMode ? 'Settings' : undefined}
          class="flex items-center gap-2.5 rounded-lg transition-colors text-[13px] font-medium
            {iconMode ? 'justify-center p-2.5' : 'px-2.5 py-2'}
            {page.url.pathname === '/settings'
              ? 'bg-cubelit-accent/15 text-cubelit-accent'
              : 'text-cubelit-text-dim hover:text-cubelit-text hover:bg-cubelit-surface'}"
        >
          <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <circle cx="12" cy="12" r="3"/>
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 0 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 0 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 0 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 0 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>
          </svg>
          {#if !iconMode}<span>Settings</span>{/if}
        </button>
      </nav>

      <!-- Server list -->
      <div class="flex flex-col gap-1.5 flex-1 min-h-0">
        {#if !iconMode}
          <div class="flex items-center justify-between px-1.5 py-1">
            <span class="text-[10px] font-semibold uppercase tracking-widest text-cubelit-muted">Servers</span>
            <span class="text-[10px] font-mono text-cubelit-muted">{servers.servers.length}</span>
          </div>
        {:else}
          <div class="h-px bg-cubelit-border mx-1"></div>
        {/if}

        <div class="overflow-y-auto flex flex-col {iconMode ? 'items-center gap-1.5' : 'gap-0.5'}">
          {#each servers.servers as server, i (server.id)}
            {#if iconMode}
              <!-- Icon mode: game icon with status dot -->
              <button
                type="button"
                onclick={() => goto(`/server/${server.id}`)}
                title="{server.name} · {server.status}"
                class="animate-fade-in stagger-{Math.min(i + 1, 8)} relative p-1 rounded-xl transition-colors
                  {isServerActive(server.id) ? 'ring-2 ring-cubelit-accent' : 'hover:bg-cubelit-surface'}"
              >
                <GameIcon recipeId={server.recipe_id} gameName={server.game} size={32} radius={7} />
                <span
                  class="absolute bottom-0.5 right-0.5 status-dot {statusClass(server.status)}"
                  style="width: 7px; height: 7px; border: 2px solid var(--c-bg-2); border-radius: 50%;"
                ></span>
              </button>
            {:else}
              <!-- Expanded mode: icon + name + status dot -->
              <button
                type="button"
                onclick={() => goto(`/server/${server.id}`)}
                class="animate-slide-in-left stagger-{Math.min(i + 1, 8)} w-full flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors text-left
                  {isServerActive(server.id)
                    ? 'bg-cubelit-surface border border-cubelit-border text-cubelit-text'
                    : 'text-cubelit-text-dim hover:bg-cubelit-surface border border-transparent'}"
              >
                <GameIcon recipeId={server.recipe_id} gameName={server.game} size={20} radius={5} />
                <span class="text-xs flex-1 truncate">{server.name}</span>
                <span class="status-dot {statusClass(server.status)} shrink-0"></span>
              </button>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Docker status footer -->
      <div
        class="flex items-center gap-2 rounded-xl border border-cubelit-border bg-cubelit-surface
          {iconMode ? 'justify-center p-2' : 'px-3 py-2.5'}"
      >
        <span class="status-dot running shrink-0"></span>
        {#if !iconMode}
          <div class="leading-tight min-w-0">
            <div class="text-[12px] text-cubelit-text truncate">
              Docker {docker.status.version ?? 'ready'}
            </div>
            <div class="text-[10px] text-cubelit-muted font-mono">engine</div>
          </div>
        {/if}
      </div>
    </aside>

    <!-- ── Main content ── -->
    <main class="app-main overflow-y-auto bg-cubelit-bg min-w-0">
      {@render children()}
    </main>
  </div>

  <!-- Update available banner -->
  {#if updateAvailable}
    <div class="fixed bottom-4 right-4 z-50 flex items-center gap-3 bg-cubelit-surface border border-cubelit-border-2 rounded-xl px-4 py-3 shadow-2xl">
      <div>
        <p class="text-sm font-medium text-cubelit-text">Update available</p>
        <p class="text-xs text-cubelit-muted">v{updateAvailable.version} is ready to install</p>
      </div>
      <button
        type="button"
        class="text-xs font-medium text-cubelit-accent hover:brightness-110 transition-colors disabled:opacity-50"
        onclick={updateAvailable.download}
        disabled={updateLoading}
      >
        {updateLoading ? 'Installing…' : 'Update'}
      </button>
      <button
        type="button"
        class="text-cubelit-muted hover:text-cubelit-text transition-colors"
        onclick={() => { updateAvailable = null; }}
        aria-label="Dismiss update notice"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>
  {/if}
{/if}
