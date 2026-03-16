<script lang="ts">
  import { goto } from "$app/navigation";
  import { getServersStore } from "$lib/stores/servers.svelte";
  import CubelitCard from "$lib/components/CubelitCard.svelte";

  const servers = getServersStore();
</script>

<div class="p-8">
  <div class="flex items-center justify-between mb-8">
    <div>
      <h1 class="text-2xl font-bold text-cubelit-text">Dashboard</h1>
      <p class="text-cubelit-muted mt-1">
        {servers.servers.length} server{servers.servers.length !== 1 ? "s" : ""}
      </p>
    </div>
  </div>

  {#if servers.servers.length === 0}
    <!-- Empty state -->
    <div class="flex flex-col items-center justify-center py-24">
      <div class="w-20 h-20 rounded-2xl bg-cubelit-surface border border-cubelit-border flex items-center justify-center mb-6">
        <svg class="w-10 h-10 text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 7.5l-2.25-1.313M21 7.5v2.25m0-2.25l-2.25 1.313M3 7.5l2.25-1.313M3 7.5l2.25 1.313M3 7.5v2.25m9 3l2.25-1.313M12 12.75l-2.25-1.313M12 12.75V15m0 6.75l2.25-1.313M12 21.75V19.5m0 2.25l-2.25-1.313m0-16.875L12 2.25l2.25 1.313M21 14.25v2.25l-2.25 1.313m-13.5 0L3 16.5v-2.25" />
        </svg>
      </div>
      <h2 class="text-xl font-semibold text-cubelit-text mb-2">No servers yet</h2>
      <p class="text-cubelit-muted mb-6 max-w-md text-center">
        Create your first game server to get started. Choose from Minecraft, FiveM, and more.
      </p>
      <a
        href="/create"
        class="inline-flex items-center gap-2 px-6 py-3 bg-cubelit-accent text-white rounded-xl hover:bg-cubelit-accent-hover transition-colors font-medium"
      >
        <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
        </svg>
        Create Your First Server
      </a>
    </div>
  {:else}
    <!-- Server grid -->
    <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-5">
      {#each servers.servers as server (server.id)}
        <CubelitCard
          {server}
          onstart={(id) => servers.start(id)}
          onstop={(id) => servers.stop(id)}
          onclick={(id) => goto(`/server/${id}`)}
        />
      {/each}
    </div>
  {/if}
</div>
