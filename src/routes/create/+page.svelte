<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { listen } from '@tauri-apps/api/event';
  import { getRecipesStore } from '$lib/stores/recipes.svelte';
  import { getServersStore } from '$lib/stores/servers.svelte';
  import { getSettingsStore } from '$lib/stores/settings.svelte';
  import { getRecipeDetail } from '$lib/api/recipes';
  import { createServer } from '$lib/api/servers';
  import { getGameDefinition } from '$lib/games/registry';
  import { GAME_ART, GAME_HUE } from '$lib/games/art';
  import type { Recipe } from '$lib/types/recipe';
  import type { ServerCreateProgress } from '$lib/types/server';
  import CreateProgress from '$lib/components/CreateProgress.svelte';
  import GameArt from '$lib/components/GameArt.svelte';
  import Button from '$lib/components/Button.svelte';

  const recipesStore = getRecipesStore();
  const serversStore = getServersStore();
  const settingsStore = getSettingsStore();

  let step = $state(1);
  let selectedRecipeId = $state<string | null>(null);
  let selectedRecipe = $state<Recipe | null>(null);
  let serverName = $state('');
  let envValues = $state<Record<string, string>>({});
  let portValues = $state<Record<string, number>>({});
  let volumePath = $state('');
  let volumePathDirty = $state(false);
  let imageTagOverride = $state<string | null>(null);
  let creating = $state(false);
  let createStep = $state('preparing');
  let createProgress = $state<number | null>(0);
  let createMessage = $state('Preparing...');
  let createError = $state<string | null>(null);

  let unlisten: (() => void) | null = null;

  function getDefaultVolumePath(name: string): string {
    const sanitized = name.replace(/[^a-zA-Z0-9 _-]/g, '');
    const root = settingsStore.effectiveInstallRoot;
    return root ? `${root}/${sanitized}` : `~/Cubelit/${sanitized}`;
  }

  onMount(async () => { settingsStore.init(); await recipesStore.load(); });
  onDestroy(() => { if (unlisten) unlisten(); });

  async function selectRecipe(id: string) {
    selectedRecipeId = id;
    try {
      selectedRecipe = await getRecipeDetail(id);
      serverName = `My ${selectedRecipe.name} Server`;
      volumePathDirty = false;
      volumePath = getDefaultVolumePath(serverName);
      envValues = Object.fromEntries(selectedRecipe.environment.map(e => [e.key, e.default_value]));
      portValues = Object.fromEntries(selectedRecipe.ports.map(p => [`${p.container_port}/${p.protocol}`, p.default_host_port]));
      step = 2;
    } catch (e) { console.error('Failed to load recipe:', e); }
  }

  async function handleCreate() {
    if (!selectedRecipeId || !selectedRecipe) return;
    creating = true;
    createError = null;
    createStep = 'preparing';
    createProgress = 0;
    createMessage = 'Preparing...';

    unlisten = await listen<ServerCreateProgress>('server-create-progress', (event) => {
      createStep = event.payload.step;
      createProgress = event.payload.progress;
      createMessage = event.payload.message;
    });

    try {
      const vp = volumePath.startsWith('~/') ? undefined : volumePath;
      const result = await createServer({
        name: serverName,
        recipe_id: selectedRecipeId,
        env_overrides: envValues,
        port_overrides: Object.fromEntries(Object.entries(portValues).map(([k, v]) => [k, v])),
        volume_path: vp,
        tag_override: imageTagOverride ?? undefined,
      });

      createStep = 'ready';
      createProgress = 1;
      createMessage = 'Server is ready!';
      await serversStore.load();
      setTimeout(() => goto(`/server/${result.id}`), 1500);
    } catch (e) {
      createError = String(e);
      creating = false;
    } finally {
      if (unlisten) { unlisten(); unlisten = null; }
    }
  }

  const gameDef = $derived(selectedRecipeId ? getGameDefinition(selectedRecipeId) : null);
  const isFiveM = $derived(selectedRecipeId === 'fivem');
</script>

<div class="p-8 max-w-[900px] mx-auto">
  <!-- Back + step header -->
  <div class="mb-5">
    <button
      type="button"
      onclick={() => { if (step === 1) goto('/'); else if (step === 2) { imageTagOverride = null; step = 1; } else step = 2; }}
      disabled={creating}
      class="inline-flex items-center gap-1.5 text-cubelit-text-dim hover:text-cubelit-text transition-colors text-sm mb-4 disabled:opacity-40 disabled:cursor-not-allowed"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 6l-9 6 9 6"/>
      </svg>
      {step === 1 ? 'Dashboard' : 'Back'}
    </button>
    {#if !creating && !createError}
      <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-1.5">step {step} of 3</div>
      <h1 class="text-2xl font-semibold tracking-tight text-cubelit-text">
        {step === 1 ? 'Pick a game' : step === 2 ? 'Configure your server' : 'Review & create'}
      </h1>
    {/if}
  </div>

  <!-- Progress bar -->
  {#if !creating && !createError}
    <div class="flex gap-1.5 mb-7">
      {#each [1, 2, 3] as i}
        <div
          class="flex-1 h-[3px] rounded-full transition-colors"
          style="background: {step >= i ? 'var(--c-accent)' : 'var(--c-border)'};"
        ></div>
      {/each}
    </div>
  {/if}

  {#if creating}
    <!-- Terminal progress -->
    <CreateProgress
      step={createStep}
      progress={createProgress}
      message={createMessage}
      recipeName={selectedRecipeId ?? ''}
      serverName={serverName}
    />

  {:else if createError}
    <!-- Error state -->
    <div class="py-12 text-center">
      <div class="w-14 h-14 mx-auto rounded-full bg-cubelit-error/15 flex items-center justify-center mb-4">
        <svg class="w-7 h-7 text-cubelit-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"/>
        </svg>
      </div>
      <h2 class="text-base font-semibold text-cubelit-text mb-2">Failed to create server</h2>
      <p class="text-sm text-cubelit-text-dim mb-6 max-w-sm mx-auto">{createError}</p>
      <div class="flex gap-2 justify-center">
        <Button variant="secondary" onclick={() => { createError = null; }}>Try Again</Button>
        <Button variant="ghost" onclick={() => goto('/')}>Go Back</Button>
      </div>
    </div>

  {:else if step === 1}
    <!-- Step 1: game picker grid -->
    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 12px;">
      {#each recipesStore.recipes as recipe, i (recipe.id)}
        {@const def = getGameDefinition(recipe.id)}
        {#if !recipe.available}
          <!-- Coming soon -->
          <div class="animate-scale-in stagger-{Math.min(i + 1, 8)} rounded-xl border-2 border-cubelit-border bg-cubelit-surface opacity-55 cursor-not-allowed overflow-hidden relative">
            <div class="absolute top-2 right-2 z-10">
              <span class="pill text-[10px]">Soon</span>
            </div>
            <GameArt recipeId={recipe.id} gameName={recipe.name} monogram={def.tileMonogram ?? recipe.name.charAt(0)} height={160} variant="dim" showLogo={true} />
            <div class="p-3.5">
              <div class="text-sm font-semibold text-cubelit-text">{recipe.name}</div>
              <div class="text-xs text-cubelit-text-dim mt-0.5">{def.cardStyle?.subtitle ?? recipe.description}</div>
            </div>
          </div>
        {:else}
          <button
            type="button"
            onclick={() => selectRecipe(recipe.id)}
            class="card-lift animate-scale-in stagger-{Math.min(i + 1, 8)} text-left rounded-xl border-2 overflow-hidden
              {selectedRecipeId === recipe.id ? 'border-cubelit-accent' : 'border-cubelit-border hover:border-cubelit-border-2'}"
          >
            <GameArt recipeId={recipe.id} gameName={recipe.name} monogram={def.tileMonogram ?? recipe.name.charAt(0)} height={160} variant="dim" showLogo={true} />
            <div class="p-3.5 {selectedRecipeId === recipe.id ? 'bg-cubelit-accent/8' : 'bg-cubelit-surface'}">
              <div class="text-sm font-semibold text-cubelit-text">{recipe.name}</div>
              <div class="text-xs text-cubelit-text-dim mt-0.5">{def.cardStyle?.subtitle ?? recipe.description}</div>
              <div class="text-[11px] text-cubelit-muted font-mono mt-1.5 truncate">{recipe.id}</div>
            </div>
          </button>
        {/if}
      {/each}
    </div>

  {:else if step === 2 && selectedRecipe}
    <!-- Step 2: configure -->
    {#if gameDef}
      {@const SetupComponent = gameDef.setupComponent}
      <SetupComponent
        recipe={selectedRecipe}
        bind:serverName
        {envValues}
        {portValues}
        bind:volumePath
        onenvchange={(k, v) => { envValues[k] = v; }}
        onportchange={(k, v) => { portValues[k] = v; }}
        onname={(n) => { serverName = n; if (!volumePathDirty) volumePath = getDefaultVolumePath(n); }}
        onvolumepath={(p) => { volumePath = p; volumePathDirty = true; }}
        ontagchange={(tag) => { imageTagOverride = tag === 'latest' ? null : tag; }}
      />
    {/if}
    <div class="flex gap-2 mt-7">
      <Button variant="ghost" onclick={() => { imageTagOverride = null; step = 1; }}>Back</Button>
      <Button onclick={() => { if (serverName.trim()) step = 3; }} disabled={!serverName.trim()}>
        Review →
      </Button>
    </div>

  {:else if step === 3 && selectedRecipe}
    <!-- Step 3: review -->
    <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5">
      <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-3">Summary</div>
      <div class="flex flex-col gap-2.5 mb-4">
        {#each [
          ['Game',     selectedRecipe.name],
          ['Name',     serverName],
          ['Image',    `${selectedRecipe.docker_image}:${imageTagOverride ?? selectedRecipe.default_tag}`],
          ['Location', volumePath],
          ['Ports',    Object.entries(portValues).map(([k,v]) => `${k}→${v}`).join(', ') || '—'],
        ] as [k, v]}
          <div class="flex justify-between text-sm gap-4">
            <span class="text-cubelit-muted shrink-0">{k}</span>
            <span class="font-mono text-cubelit-text-dim text-right truncate">{v}</span>
          </div>
        {/each}
      </div>

      {#if isFiveM}
        <div class="flex items-center gap-2 px-3 py-2.5 rounded-lg border border-cubelit-border bg-cubelit-bg-2 text-xs text-cubelit-text-dim mb-4">
          <svg class="w-3.5 h-3.5 text-cubelit-accent shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M13 2L3 14h7l-1 8 10-12h-7l1-8z"/>
          </svg>
          A MariaDB sidecar will be auto-provisioned.
        </div>
      {/if}

      {#if gameDef?.reviewNotes}
        {#each gameDef.reviewNotes as note}
          <div class="flex items-center gap-2 px-3 py-2.5 rounded-lg border border-cubelit-border bg-cubelit-bg-2 text-xs text-cubelit-text-dim mb-2">
            <svg class="w-3.5 h-3.5 text-cubelit-accent shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M13 2L3 14h7l-1 8 10-12h-7l1-8z"/>
            </svg>
            {note}
          </div>
        {/each}
      {/if}

      <div class="flex gap-2 mt-4">
        <Button variant="ghost" onclick={() => { step = 2; }}>Back</Button>
        <Button onclick={handleCreate} loading={creating} disabled={creating}>Create server</Button>
      </div>
    </div>
  {/if}
</div>
