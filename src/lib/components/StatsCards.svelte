<script lang="ts">
  import { getServerStats } from "$lib/api/docker";
  import type { ContainerStats } from "$lib/types/docker";

  interface Props {
    serverId: string;
    serverStatus: string;
  }

  let { serverId, serverStatus }: Props = $props();

  let stats = $state<ContainerStats | null>(null);

  function formatMemory(mb: number): string {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${Math.round(mb)} MB`;
  }

  async function fetchStats() {
    try {
      stats = await getServerStats(serverId);
    } catch {
      stats = null;
    }
  }

  $effect(() => {
    if (serverStatus === "running" || serverStatus === "starting") {
      fetchStats();
      const interval = setInterval(fetchStats, 5000);
      return () => clearInterval(interval);
    } else {
      stats = null;
    }
  });

  const cpuPercent = $derived(stats ? Math.min(stats.cpu_percent, 100) : 0);
  const memPercent = $derived(
    stats && stats.memory_limit_mb > 0
      ? Math.min((stats.memory_usage_mb / stats.memory_limit_mb) * 100, 100)
      : 0
  );
</script>

<div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
  <h3 class="text-sm font-medium text-cubelit-muted mb-3">CPU</h3>
  {#if stats}
    <p class="text-lg font-semibold text-cubelit-text">{stats.cpu_percent.toFixed(1)}%</p>
    <div class="mt-2 h-1.5 bg-cubelit-bg rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-all duration-500 {cpuPercent > 80 ? 'bg-cubelit-error' : cpuPercent > 50 ? 'bg-cubelit-warning' : 'bg-cubelit-accent'}"
        style="width: {cpuPercent}%"
      ></div>
    </div>
  {:else}
    <p class="text-sm text-cubelit-muted">{serverStatus === "running" || serverStatus === "starting" ? "Loading..." : "—"}</p>
  {/if}
</div>

<div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
  <h3 class="text-sm font-medium text-cubelit-muted mb-3">Memory</h3>
  {#if stats}
    <p class="text-lg font-semibold text-cubelit-text">
      {formatMemory(stats.memory_usage_mb)}
      <span class="text-sm font-normal text-cubelit-muted">/ {formatMemory(stats.memory_limit_mb)}</span>
    </p>
    <div class="mt-2 h-1.5 bg-cubelit-bg rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-all duration-500 {memPercent > 80 ? 'bg-cubelit-error' : memPercent > 50 ? 'bg-cubelit-warning' : 'bg-cubelit-accent'}"
        style="width: {memPercent}%"
      ></div>
    </div>
  {:else}
    <p class="text-sm text-cubelit-muted">{serverStatus === "running" || serverStatus === "starting" ? "Loading..." : "—"}</p>
  {/if}
</div>
