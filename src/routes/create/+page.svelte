<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { listen } from "@tauri-apps/api/event";
  import { getRecipesStore } from "$lib/stores/recipes.svelte";
  import { getServersStore } from "$lib/stores/servers.svelte";
  import { getRecipeDetail } from "$lib/api/recipes";
  import { createServer } from "$lib/api/servers";
  import { getGameDefinition } from "$lib/games/registry";
  import type { Recipe } from "$lib/types/recipe";
  import type { ServerCreateProgress } from "$lib/types/server";
  import GameSelector from "$lib/components/GameSelector.svelte";
  import CreateProgress from "$lib/components/CreateProgress.svelte";
  import Button from "$lib/components/Button.svelte";

  const recipesStore = getRecipesStore();
  const serversStore = getServersStore();

  let step = $state(1);
  let selectedRecipeId = $state<string | null>(null);
  let selectedRecipe = $state<Recipe | null>(null);
  let serverName = $state("");
  let envValues = $state<Record<string, string>>({});
  let portValues = $state<Record<string, number>>({});
  let volumePath = $state("");
  let imageTagOverride = $state<string | null>(null);
  let creating = $state(false);
  let createStep = $state("preparing");
  let createProgress = $state<number | null>(0);
  let createMessage = $state("Preparing...");
  let createError = $state<string | null>(null);

  let unlisten: (() => void) | null = null;

  function totalSteps(): number {
    return 3;
  }

  function getDefaultVolumePath(name: string): string {
    // We'll let the backend compute the real path; show a placeholder
    const sanitized = name.replace(/[^a-zA-Z0-9 _-]/g, "");
    return `~/Cubelit/${sanitized}`;
  }

  onMount(async () => {
    await recipesStore.load();
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  async function selectRecipe(id: string) {
    selectedRecipeId = id;
    try {
      selectedRecipe = await getRecipeDetail(id);
      serverName = `My ${selectedRecipe.name} Server`;
      volumePath = getDefaultVolumePath(serverName);

      envValues = {};
      for (const env of selectedRecipe.environment) {
        envValues[env.key] = env.default_value;
      }

      portValues = {};
      for (const port of selectedRecipe.ports) {
        portValues[`${port.container_port}/${port.protocol}`] = port.default_host_port;
      }

      step = 2;
    } catch (e) {
      console.error("Failed to load recipe:", e);
    }
  }

  function goToReview() {
    if (!serverName.trim()) return;
    step = 3;
  }

  async function handleCreate() {
    if (!selectedRecipeId || !selectedRecipe) return;

    creating = true;
    createError = null;
    createStep = "preparing";
    createProgress = 0;
    createMessage = "Preparing...";

    unlisten = await listen<ServerCreateProgress>("server-create-progress", (event) => {
      createStep = event.payload.step;
      createProgress = event.payload.progress;
      createMessage = event.payload.message;
    });

    try {
      // Only pass volumePath if user explicitly changed it from the placeholder
      const vp = volumePath.startsWith("~/") ? undefined : volumePath;

      const result = await createServer({
        name: serverName,
        recipe_id: selectedRecipeId,
        env_overrides: envValues,
        port_overrides: Object.fromEntries(
          Object.entries(portValues).map(([k, v]) => [k, v])
        ),
        volume_path: vp,
        tag_override: imageTagOverride ?? undefined,
      });

      createStep = "ready";
      createProgress = 1;
      createMessage = "Server is ready!";

      await serversStore.load();

      setTimeout(() => {
        goto(`/server/${result.id}`);
      }, 1500);
    } catch (e) {
      createError = String(e);
      creating = false;
    } finally {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    }
  }

  function isConfigStep(): boolean {
    return step === 2;
  }

  function isReviewStep(): boolean {
    return step === 3;
  }

  function selectedGameDefinition() {
    return selectedRecipeId ? getGameDefinition(selectedRecipeId) : null;
  }

  function goBackFromConfig() {
    imageTagOverride = null;
    step = 1;
  }

  function goBackFromReview() {
    step = 2;
  }
</script>

<div class="p-8 max-w-3xl mx-auto">
  <!-- Header -->
  <div class="flex items-center gap-4 mb-8">
    <a href="/" class="text-cubelit-muted hover:text-cubelit-text transition-colors" aria-label="Back to dashboard">
      <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
      </svg>
    </a>
    <div>
      <h1 class="text-2xl font-bold text-cubelit-text">Create Server</h1>
      <p class="text-cubelit-muted text-sm">Step {step} of {totalSteps()}</p>
    </div>
  </div>

  <!-- Step indicators -->
  <div class="flex items-center gap-2 mb-8">
    {#each Array(totalSteps()) as _, i}
      <div class="flex-1 h-1 rounded-full {step >= i + 1 ? 'bg-cubelit-accent' : 'bg-cubelit-border'} transition-colors"></div>
    {/each}
  </div>

  {#if creating}
    <!-- Creating state -->
    <div class="py-16">
      <CreateProgress step={createStep} progress={createProgress} message={createMessage} />
    </div>
  {:else if createError}
    <!-- Error state -->
    <div class="py-16 text-center">
      <div class="w-16 h-16 mx-auto rounded-full bg-cubelit-error/20 flex items-center justify-center mb-4">
        <svg class="w-8 h-8 text-cubelit-error" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
        </svg>
      </div>
      <h2 class="text-lg font-semibold text-cubelit-text mb-2">Failed to create server</h2>
      <p class="text-cubelit-muted mb-6 text-sm">{createError}</p>
      <div class="flex gap-3 justify-center">
        <Button variant="secondary" onclick={() => { createError = null; }}>Try Again</Button>
        <Button variant="ghost" onclick={() => goto("/")}>Go Back</Button>
      </div>
    </div>
  {:else if step === 1}
    <!-- Step 1: Pick a game -->
    <GameSelector
      recipes={recipesStore.recipes}
      selectedId={selectedRecipeId}
      onselect={selectRecipe}
    />
  {:else if isConfigStep() && selectedRecipe}
    <!-- Config Step: Game-specific form -->
    {@const gameDefinition = selectedGameDefinition()}
    {#if gameDefinition}
      {@const SetupComponent = gameDefinition.setupComponent}
      <SetupComponent
        recipe={selectedRecipe}
        bind:serverName
        {envValues}
        {portValues}
        bind:volumePath
        onenvchange={(k, v) => { envValues[k] = v; }}
        onportchange={(k, v) => { portValues[k] = v; }}
        onname={(n) => {
          serverName = n;
          if (selectedRecipeId === "fivem") envValues["SERVER_NAME"] = n;
          volumePath = getDefaultVolumePath(n);
        }}
        onvolumepath={(p) => { volumePath = p; }}
        ontagchange={(tag) => { imageTagOverride = tag === "latest" ? null : tag; }}
      />
    {/if}

    <div class="flex gap-3 mt-8">
      <Button variant="ghost" onclick={goBackFromConfig}>Back</Button>
      <Button onclick={goToReview} disabled={!serverName.trim()}>
        Review
      </Button>
    </div>
  {:else if isReviewStep() && selectedRecipe}
    <!-- Review & Create -->
    {@const gameDefinition = selectedGameDefinition()}
    <div class="space-y-6">
      <h2 class="text-lg font-semibold text-cubelit-text">Review & Create</h2>

      <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5 space-y-4">
        <div class="flex justify-between">
          <span class="text-sm text-cubelit-muted">Game</span>
          <span class="text-sm text-cubelit-text font-medium">{selectedRecipe.name}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-cubelit-muted">Server Name</span>
          <span class="text-sm text-cubelit-text font-medium">{serverName}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-cubelit-muted">Image</span>
          <span class="text-sm text-cubelit-text font-mono">{selectedRecipe.docker_image}:{imageTagOverride ?? selectedRecipe.default_tag}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-cubelit-muted">Location</span>
          <span class="text-sm text-cubelit-text font-mono text-right max-w-[60%] truncate">{volumePath}</span>
        </div>

        {#if Object.keys(portValues).length > 0}
          <div class="border-t border-cubelit-border pt-4">
            <p class="text-sm text-cubelit-muted mb-2">Ports</p>
            {#each Object.entries(portValues) as [container, host]}
              <div class="flex justify-between text-sm">
                <span class="text-cubelit-muted">{container}</span>
                <span class="text-cubelit-text">{host}</span>
              </div>
            {/each}
          </div>
        {/if}

        {#if Object.keys(envValues).length > 0}
          <div class="border-t border-cubelit-border pt-4">
            <p class="text-sm text-cubelit-muted mb-2">Environment</p>
            {#each Object.entries(envValues) as [key, value]}
              <div class="flex justify-between text-sm">
                <span class="text-cubelit-muted font-mono text-xs">{key}</span>
                <span class="text-cubelit-text text-xs">{value}</span>
              </div>
            {/each}
          </div>
        {/if}

        {#if gameDefinition?.reviewNotes}
          <div class="border-t border-cubelit-border pt-4">
            <p class="text-sm text-cubelit-muted mb-2">Notes</p>
            {#each gameDefinition.reviewNotes as note}
              <p class="text-xs text-cubelit-muted">{note}</p>
            {/each}
          </div>
        {/if}
      </div>

      <div class="flex gap-3">
        <Button variant="ghost" onclick={goBackFromReview}>Back</Button>
        <Button onclick={handleCreate}>
          Create Server
        </Button>
      </div>
    </div>
  {/if}
</div>
