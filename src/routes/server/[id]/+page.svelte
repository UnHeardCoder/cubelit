<script lang="ts">
  import { page } from "$app/state";
  import { onMount, onDestroy, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { listen } from "@tauri-apps/api/event";
  import { syncServerStatus, renameServer } from "$lib/api/servers";
  import { getGameDefinition } from "$lib/games/registry";
  import { getServersStore } from "$lib/stores/servers.svelte";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import Spinner from "$lib/components/Spinner.svelte";
  import type { Cubelit } from "$lib/types/server";

  const servers = getServersStore();

  let server = $state<Cubelit | null>(null);
  let loading = $state(true);
  let actionLoading = $state(false);

  let showDeleteModal = $state(false);
  let deleteWithData = $state(false);
  let showRestartModal = $state(false);

  let editing = $state(false);
  let editName = $state("");
  let editNameInput = $state<HTMLInputElement | null>(null);
  let DashboardComponent = $derived(server ? getGameDefinition(server.recipe_id).dashboardComponent : null);

  let statusUnlisten: (() => void) | null = null;

  async function startEditing() {
    if (!server) return;
    editName = server.name;
    editing = true;
    await tick();
    editNameInput?.focus();
    editNameInput?.select();
  }

  async function saveName() {
    if (!server) return;
    const trimmed = editName.trim();
    if (!trimmed || trimmed === server.name) {
      editing = false;
      return;
    }
    try {
      server = await renameServer(server.id, trimmed);
      servers.load();
    } catch {
      // revert on error
    }
    editing = false;
  }

  function cancelEdit() {
    editing = false;
  }

  onMount(async () => {
    try {
      const id = page.params.id;
      if (!id) { await goto("/"); return; }
      server = await syncServerStatus(id);

      // Listen for readiness events emitted by the Rust log watcher
      statusUnlisten = await listen<string>("server-status-changed", async (event) => {
        if (server && event.payload === server.id) {
          server = await syncServerStatus(server.id);
        }
      });
    } catch {
      await goto("/");
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    if (statusUnlisten) statusUnlisten();
  });

  async function handleStart() {
    if (!server) return;
    actionLoading = true;
    try {
      await servers.start(server.id);
      server = await syncServerStatus(server.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleStop() {
    if (!server) return;
    actionLoading = true;
    try {
      await servers.stop(server.id);
      server = await syncServerStatus(server.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleRestart() {
    if (!server) return;
    actionLoading = true;
    try {
      await servers.restart(server.id);
      server = await syncServerStatus(server.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (!server) return;
    actionLoading = true;
    showDeleteModal = false;
    try {
      await servers.remove(server.id, deleteWithData);
      await goto("/");
    } finally {
      actionLoading = false;
      deleteWithData = false;
    }
  }
</script>

<!-- Restart Confirmation Modal -->
<Modal bind:open={showRestartModal} onclose={() => showRestartModal = false} title="Restart Server">
  <div class="space-y-4">
    <p class="text-sm text-cubelit-muted">
      Restarting <span class="text-cubelit-text font-medium">{server?.name}</span> will
      <span class="text-cubelit-warning font-medium">disconnect all active players</span>.
      Continue?
    </p>
    <div class="flex gap-3 justify-end pt-2">
      <Button variant="ghost" onclick={() => showRestartModal = false}>Cancel</Button>
      <Button variant="secondary" onclick={() => { showRestartModal = false; handleRestart(); }} loading={actionLoading}>Restart</Button>
    </div>
  </div>
</Modal>

<!-- Delete Confirmation Modal -->
<Modal bind:open={showDeleteModal} onclose={() => { showDeleteModal = false; deleteWithData = false; }} title="Delete Server">
  <div class="space-y-4">
    <p class="text-sm text-cubelit-muted">
      Delete <span class="text-cubelit-text font-medium">{server?.name}</span>? This will stop and remove the container.
    </p>

    <label class="flex items-start gap-3 cursor-pointer">
      <input
        type="checkbox"
        class="mt-0.5 accent-cubelit-error"
        bind:checked={deleteWithData}
      />
      <div>
        <p class="text-sm text-cubelit-text">Also delete server files from disk</p>
        {#if deleteWithData}
          <p class="text-xs text-cubelit-error mt-0.5">This will permanently delete all world data, mods, and configs. This cannot be undone.</p>
        {:else}
          <p class="text-xs text-cubelit-muted mt-0.5">Your server files will remain at <span class="font-mono">{server?.volume_path}</span></p>
        {/if}
      </div>
    </label>

      <div class="flex gap-3 justify-end pt-2">
      <Button variant="ghost" onclick={() => { showDeleteModal = false; deleteWithData = false; }}>Cancel</Button>
      <Button variant="danger" onclick={handleDelete}>Delete Server</Button>
    </div>
  </div>
</Modal>

{#if loading}
  <div class="flex items-center justify-center p-16">
    <Spinner size="lg" class="text-cubelit-accent" />
  </div>
{:else if server}
  <div class="p-8">
    <!-- Header -->
    <div class="flex items-center gap-4 mb-8">
      <a href="/" class="text-cubelit-muted hover:text-cubelit-text transition-colors" aria-label="Back to dashboard">
        <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
        </svg>
      </a>
      <div class="flex-1 min-w-0">
        {#if editing}
          <input
            bind:this={editNameInput}
            class="text-2xl font-bold text-cubelit-text bg-transparent border-b-2 border-cubelit-accent outline-none w-full"
            bind:value={editName}
            onkeydown={(e: KeyboardEvent) => {
              if (e.key === "Enter") saveName();
              if (e.key === "Escape") cancelEdit();
            }}
            onblur={saveName}
          />
        {:else}
          <div class="flex items-center gap-2">
            <h1 class="text-2xl font-bold text-cubelit-text">{server.name}</h1>
            <button
              type="button"
              class="text-sm text-cubelit-muted hover:text-cubelit-accent transition-colors"
              onclick={startEditing}
              aria-label="Rename server"
              title="Rename server"
            >
              Rename
            </button>
          </div>
        {/if}
        <p class="text-cubelit-muted">{server.game}</p>
      </div>
      <div class="flex gap-2">
        {#if server.status === "running"}
          <Button variant="secondary" onclick={() => showRestartModal = true} loading={actionLoading}>Restart</Button>
          <Button variant="danger" onclick={handleStop} loading={actionLoading}>Stop</Button>
        {:else if server.status === "starting"}
          <Button variant="danger" onclick={handleStop} loading={actionLoading}>Stop</Button>
        {:else}
          <Button variant="primary" onclick={handleStart} loading={actionLoading}>Start</Button>
        {/if}
      </div>
    </div>

    <!-- Game-specific dashboard -->
    {#if DashboardComponent}
      <DashboardComponent {server} />
    {/if}

    <!-- Danger zone -->
    <div class="mt-8 bg-cubelit-error/5 border border-cubelit-error/20 rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-error mb-3">Danger Zone</h3>
      <Button variant="danger" onclick={() => showDeleteModal = true} loading={actionLoading}>
        Delete Server
      </Button>
    </div>
  </div>
{/if}
