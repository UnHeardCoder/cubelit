<script lang="ts">
  import type { RecipeSummary } from "$lib/types/recipe";

  interface Props {
    recipe: RecipeSummary;
    selected: boolean;
    comingSoon?: boolean;
    onclick?: () => void;
  }

  let { recipe, selected, comingSoon = false, onclick }: Props = $props();

  const gameIcons: Record<string, string> = {
    "minecraft-java": "M",
    "minecraft-bedrock": "B",
    "fivem": "V",
    "rust-game": "R",
    "terraria": "T",
    "valheim": "V",
    "ark": "A",
    "cs2": "CS",
    "project-zomboid": "PZ",
    "palworld": "P",
  };
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="p-4 rounded-xl border-2 transition-all relative {comingSoon
    ? 'border-cubelit-border bg-cubelit-surface/50 opacity-50 cursor-not-allowed'
    : selected
      ? 'border-cubelit-accent bg-cubelit-accent/10 cursor-pointer'
      : 'border-cubelit-border bg-cubelit-surface hover:border-cubelit-accent/40 cursor-pointer'}"
  onclick={comingSoon ? undefined : onclick}
>
  {#if comingSoon}
    <div class="absolute top-2 right-2 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider bg-cubelit-border text-cubelit-muted rounded-full">
      Coming Soon
    </div>
  {/if}
  <div class="flex items-start gap-3">
    <div class="w-12 h-12 rounded-lg bg-cubelit-bg flex items-center justify-center text-cubelit-accent font-bold text-lg shrink-0 {comingSoon ? 'opacity-50' : ''}">
      {gameIcons[recipe.id] ?? recipe.name.charAt(0)}
    </div>
    <div class="min-w-0 flex-1">
      <h3 class="font-semibold text-cubelit-text truncate">{recipe.name}</h3>
      <p class="text-xs text-cubelit-muted mt-0.5 line-clamp-2">{recipe.description}</p>
      {#if recipe.tags.length > 0}
        <div class="flex gap-1.5 mt-2 flex-wrap">
          {#each recipe.tags as tag}
            <span class="px-1.5 py-0.5 text-xs rounded bg-cubelit-bg text-cubelit-muted">{tag}</span>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
