<script lang="ts">
  import type { Cubelit } from "$lib/types/server";
  import { updateServerSettings } from "$lib/api/docker";
  import { getRecipeDetail } from "$lib/api/recipes";
  import { parsePorts, parseEnv, STATUS_COLORS } from "$lib/utils/server";
  import type { Recipe } from "$lib/types/recipe";
  import StatsCards from "$lib/components/StatsCards.svelte";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";

  interface Props {
    server: Cubelit;
  }

  let { server }: Props = $props();

  let activeTab = $state<"overview" | "settings">("overview");

  // ─── Settings state ───────────────────────────────────────────────────────
  let recipe = $state<Recipe | null>(null);
  let editEnv = $state<Record<string, string>>({});
  let settingsLoading = $state(false);
  let settingsSaving = $state(false);
  let settingsError = $state<string | null>(null);
  let showApplyModal = $state(false);

  async function loadSettings() {
    settingsLoading = true;
    settingsError = null;
    try {
      recipe = await getRecipeDetail(server.recipe_id);
      editEnv = { ...parseEnv(server.environment) };
    } catch {
      settingsError = "Failed to load settings.";
    } finally {
      settingsLoading = false;
    }
  }

  async function applySettings() {
    settingsSaving = true;
    settingsError = null;
    showApplyModal = false;
    try {
      await updateServerSettings(server.id, editEnv);
    } catch (e) {
      settingsError = String(e);
    } finally {
      settingsSaving = false;
    }
  }

  $effect(() => {
    if (activeTab === "settings") loadSettings();
  });

  const tabs = [
    { id: "overview" as const, label: "Overview" },
    { id: "settings" as const, label: "Settings" },
  ];
</script>

<!-- Apply Settings Modal -->
<Modal bind:open={showApplyModal} onclose={() => showApplyModal = false} title="Apply Settings">
  <p class="text-sm text-cubelit-muted mb-6">
    Applying settings will <span class="text-cubelit-warning font-medium">restart your server</span>. Continue?
  </p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => showApplyModal = false}>Cancel</Button>
    <Button variant="primary" onclick={applySettings}>Apply & Restart</Button>
  </div>
</Modal>

<!-- Tabs -->
<div class="flex gap-1 mb-6 border-b border-cubelit-border">
  {#each tabs as tab}
    <button
      class="px-4 py-2.5 text-sm font-medium transition-colors relative {activeTab === tab.id
        ? 'text-cubelit-accent'
        : 'text-cubelit-muted hover:text-cubelit-text'}"
      onclick={() => activeTab = tab.id}
    >
      {tab.label}
      {#if activeTab === tab.id}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-cubelit-accent rounded-t" />
      {/if}
    </button>
  {/each}
</div>

{#if activeTab === "overview"}
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Status</h3>
      <p class="text-lg font-semibold capitalize {STATUS_COLORS[server.status] ?? 'text-cubelit-muted'}">
        {server.status}
      </p>
    </div>

    <StatsCards serverId={server.id} serverStatus={server.status} />

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Ports</h3>
      {#each Object.entries(parsePorts(server.port_mappings)) as [container, host]}
        <p class="text-cubelit-text text-sm">{container} &rarr; {host}</p>
      {/each}
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Image</h3>
      <p class="text-cubelit-text text-sm font-mono">{server.docker_image}</p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Container ID</h3>
      <p class="text-cubelit-text text-sm font-mono truncate">{server.container_id ?? "N/A"}</p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Volume Path</h3>
      <p class="text-cubelit-text text-sm font-mono truncate">{server.volume_path}</p>
    </div>
  </div>

{:else if activeTab === "settings"}
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-cubelit-text">Server Configuration</h3>
      <Button size="sm" onclick={() => showApplyModal = true} disabled={settingsSaving || settingsLoading}>
        {settingsSaving ? "Applying…" : "Apply Changes"}
      </Button>
    </div>

    {#if settingsError}
      <p class="text-xs text-cubelit-error px-3 py-2 bg-cubelit-error/5 border border-cubelit-error/30 rounded-lg">{settingsError}</p>
    {/if}

    {#if settingsLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading settings...</p>
    {:else if recipe}
      <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5 space-y-4">
        {#each recipe.environment as field}
          <div class="space-y-1">
            <label class="text-xs font-medium text-cubelit-muted">{field.label}</label>
            {#if field.type === "boolean"}
              <div class="flex items-center gap-2">
                <input
                  type="checkbox"
                  class="w-4 h-4 accent-cubelit-accent"
                  checked={editEnv[field.key]?.toLowerCase() === "true"}
                  onchange={(e) => editEnv[field.key] = (e.currentTarget as HTMLInputElement).checked ? "TRUE" : "FALSE"}
                />
                <span class="text-xs text-cubelit-muted font-mono">{field.key}</span>
              </div>
            {:else if field.options.length > 0}
              <div class="relative">
                <select
                  class="w-full appearance-none bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 pr-8 text-sm text-cubelit-text focus:outline-none focus:border-cubelit-accent"
                  bind:value={editEnv[field.key]}
                >
                  {#each field.options as opt}
                    <option value={opt} style="background-color:#23272f;color:#f5f5f6;">{opt}</option>
                  {/each}
                </select>
                <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2.5">
                  <svg class="w-4 h-4 text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                  </svg>
                </div>
              </div>
            {:else}
              <input
                type="text"
                class="w-full bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 text-sm text-cubelit-text focus:outline-none focus:border-cubelit-accent"
                bind:value={editEnv[field.key]}
              />
            {/if}
          </div>
        {/each}
      </div>
      <p class="text-xs text-cubelit-muted">Changes take effect after the server restarts.</p>
    {/if}
  </div>
{/if}
