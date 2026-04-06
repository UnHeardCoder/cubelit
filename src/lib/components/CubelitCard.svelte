<script lang="ts">
  import { getGameDefinition } from "$lib/games/registry";
  import type { Cubelit } from "$lib/types/server";
  import StatusRibbon from "./StatusRibbon.svelte";

  interface Props {
    server: Cubelit;
    onstart: (id: string) => void;
    onstop: (id: string) => void;
    onclick: (id: string) => void;
  }

  let { server, onstart, onstop, onclick }: Props = $props();

  let actionLoading = $state(false);

  const statusDot: Record<string, string> = {
    starting: "bg-cubelit-warning",
    running: "bg-cubelit-success",
    stopped: "bg-cubelit-error",
    created: "bg-cubelit-warning",
    error: "bg-cubelit-error",
  };

  const statusLabels: Record<string, string> = {
    starting: "Starting",
    running: "Online",
    stopped: "Offline",
    created: "Created",
    error: "Error",
  };

  function getStyle() {
    return getGameDefinition(server.recipe_id).cardStyle ?? {
      titleClass: "font-bold",
      subtitle: server.game,
      gradient: "from-cubelit-border/40 to-cubelit-surface",
    };
  }

  function getAddress(): string {
    try {
      const ports: Record<string, number> = JSON.parse(server.port_mappings);
      const firstPort = Object.values(ports)[0];
      if (firstPort) return `localhost:${firstPort}`;
    } catch { /* ignore */ }
    return "";
  }

  async function handleStart(e: MouseEvent) {
    e.stopPropagation();
    actionLoading = true;
    try {
      await onstart(server.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleStop(e: MouseEvent) {
    e.stopPropagation();
    actionLoading = true;
    try {
      await onstop(server.id);
    } finally {
      actionLoading = false;
    }
  }
</script>

<div
  role="button"
  tabindex="0"
  class="relative rounded-2xl border border-cubelit-border bg-gradient-to-b {getStyle().gradient} cursor-pointer hover:border-cubelit-accent/50 transition-all overflow-hidden group"
  onclick={() => onclick(server.id)}
  onkeydown={(e: KeyboardEvent) => {
    if (e.target !== e.currentTarget) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onclick(server.id);
    }
  }}
>
  <StatusRibbon status={server.status} />

  <div class="p-6 pb-0">
    <h3 class="text-2xl text-cubelit-text {getStyle().titleClass}">{server.game}</h3>
    <p class="text-sm text-cubelit-muted mt-0.5">{getStyle().subtitle}</p>

    <div class="flex items-center gap-2 mt-4">
      <span class="w-2.5 h-2.5 rounded-full {statusDot[server.status] ?? 'bg-cubelit-muted'} {server.status === 'running' || server.status === 'starting' ? 'animate-pulse' : ''}"></span>
      <span class="text-sm text-cubelit-text">{statusLabels[server.status] ?? server.status}</span>
    </div>

    <p class="text-sm text-cubelit-muted mt-1 truncate">{server.name}</p>
  </div>

  <div class="border-t border-cubelit-border/50 mt-4 px-6 py-3 flex items-center justify-between">
    <span class="text-xs text-cubelit-muted font-mono">{getAddress()}</span>
    <div class="flex gap-1.5">
      {#if server.status === "running" || server.status === "starting"}
        <button
          type="button"
          class="px-3 py-1 text-xs rounded-lg bg-cubelit-error/10 text-cubelit-error border border-cubelit-error/30 hover:bg-cubelit-error/20 transition-colors disabled:opacity-50"
          onclick={handleStop}
          disabled={actionLoading}
        >
          Stop
        </button>
      {:else if server.status === "stopped" || server.status === "created"}
        <button
          type="button"
          class="px-3 py-1 text-xs rounded-lg bg-cubelit-accent/10 text-cubelit-accent border border-cubelit-accent/30 hover:bg-cubelit-accent/20 transition-colors disabled:opacity-50"
          onclick={handleStart}
          disabled={actionLoading}
        >
          Start
        </button>
      {/if}
    </div>
  </div>
</div>
