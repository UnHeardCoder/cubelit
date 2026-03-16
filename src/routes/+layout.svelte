<script lang="ts">
  import "../app.css";
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/state";
  import { getDockerStore } from "$lib/stores/docker.svelte";
  import { getServersStore } from "$lib/stores/servers.svelte";
  import DockerOnboarding from "$lib/components/DockerOnboarding.svelte";
  import Spinner from "$lib/components/Spinner.svelte";

  let { children } = $props();

  const docker = getDockerStore();
  const servers = getServersStore();

  let initialized = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await docker.check();
    if (docker.status.available) {
      await servers.load();
      refreshInterval = setInterval(() => servers.load(), 30_000);
    }
    initialized = true;
  });

  onDestroy(() => {
    if (refreshInterval !== null) clearInterval(refreshInterval);
  });

  function handleDockerCheck() {
    docker.check().then(() => {
      if (docker.status.available) {
        servers.load();
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
    error={docker.status.error}
    checking={docker.checking}
    oncheck={handleDockerCheck}
  />
{:else}
  <div class="min-h-screen bg-cubelit-bg text-cubelit-text flex">
    <!-- Narrow Icon Sidebar -->
    <aside class="w-16 bg-cubelit-surface border-r border-cubelit-border flex flex-col items-center shrink-0 py-3 gap-2">
      <!-- Logo -->
      <a href="/" class="w-10 h-10 bg-cubelit-accent rounded-xl flex items-center justify-center text-white font-bold text-sm mb-2">
        C
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
{/if}
