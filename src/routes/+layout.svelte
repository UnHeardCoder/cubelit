<script lang="ts">
  import "../app.css";
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/state";
  import { getDockerStore } from "$lib/stores/docker.svelte";
  import { getServersStore } from "$lib/stores/servers.svelte";
  import DockerOnboarding from "$lib/components/DockerOnboarding.svelte";
  import Spinner from "$lib/components/Spinner.svelte";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  let { children } = $props();

  const docker = getDockerStore();
  const servers = getServersStore();

  let initialized = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let updateAvailable = $state<{ version: string; download: () => Promise<void> } | null>(null);
  let updateLoading = $state(false);

  function startRefreshLoop() {
    if (refreshInterval !== null) return;
    refreshInterval = setInterval(() => servers.load(), 30_000);
  }

  onMount(async () => {
    await docker.check();
    if (docker.status.available) {
      await servers.load();
      startRefreshLoop();
    }
    initialized = true;

    // Silently check for app updates in the background
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
    } catch (error) {
      console.error("Failed to check for app updates:", error);
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

  function isActive(path: string): boolean {
    return page.url.pathname === path;
  }
</script>

{#if !initialized}
  <div class="min-h-screen bg-cubelit-bg flex items-center justify-center">
    <div class="text-center">
      <Spinner size="lg" class="text-cubelit-accent mx-auto mb-4" />
      <p class="text-cubelit-muted">Loading Cubelit...</p>
    </div>
  </div>
{:else if !docker.status.available}
  <DockerOnboarding
    status={docker.onboarding}
    checking={docker.checking}
    oncheck={handleDockerCheck}
  />
{:else}
  <div class="min-h-screen bg-cubelit-bg text-cubelit-text flex">
    <!-- Narrow Icon Sidebar -->
    <aside class="w-16 bg-cubelit-surface border-r border-cubelit-border flex flex-col items-center shrink-0 py-3 gap-2">
      <!-- Logo -->
      <a href="/" class="w-10 h-10 rounded-xl flex items-center justify-center mb-2 overflow-hidden">
        <img src="/logo.jpg" alt="Cubelit" class="w-10 h-10 object-contain" />
      </a>

      <!-- Dashboard -->
      <a
        href="/"
        class="w-10 h-10 rounded-xl flex items-center justify-center transition-colors {isActive('/') ? 'bg-cubelit-accent text-white' : 'text-cubelit-muted hover:bg-cubelit-accent/10 hover:text-cubelit-accent'}"
        title="Dashboard"
      >
        <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
        </svg>
      </a>

      <div class="flex-1"></div>

      <!-- Add Server -->
      <a
        href="/create"
        class="w-10 h-10 rounded-xl flex items-center justify-center transition-colors {isActive('/create') ? 'bg-cubelit-accent text-white' : 'text-cubelit-muted hover:bg-cubelit-accent/10 hover:text-cubelit-accent'}"
        title="Create Server"
      >
        <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
        </svg>
      </a>

    </aside>

    <!-- Main content -->
    <main class="flex-1 overflow-y-auto">
      {@render children()}
    </main>
  </div>

  <!-- Update available banner -->
  {#if updateAvailable}
    <div class="fixed bottom-4 right-4 z-50 flex items-center gap-3 bg-cubelit-surface border border-cubelit-accent/40 rounded-xl px-4 py-3 shadow-lg">
      <div>
        <p class="text-sm font-medium text-cubelit-text">Update available</p>
        <p class="text-xs text-cubelit-muted">v{updateAvailable.version} is ready to install</p>
      </div>
      <button
        class="text-xs font-medium text-cubelit-accent hover:text-cubelit-accent-hover transition-colors disabled:opacity-50"
        onclick={updateAvailable.download}
        disabled={updateLoading}
      >
        {updateLoading ? "Installing…" : "Update"}
      </button>
      <button
        class="text-cubelit-muted hover:text-cubelit-text transition-colors"
        onclick={() => updateAvailable = null}
        aria-label="Dismiss update notice"
      >
        <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
{/if}
