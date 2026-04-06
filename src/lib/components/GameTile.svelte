<script lang="ts">
  import { getGameDefinition } from "$lib/games/registry";
  import type { RecipeSummary } from "$lib/types/recipe";

  interface Props {
    recipe: RecipeSummary;
    selected: boolean;
    comingSoon?: boolean;
    onclick?: () => void;
  }

  let { recipe, selected, comingSoon = false, onclick }: Props = $props();

  function tileMonogram(): string {
    return getGameDefinition(recipe.id).tileMonogram ?? recipe.name.charAt(0);
  }
</script>

{#if comingSoon}
  <div
    class="p-4 rounded-xl border-2 transition-all relative border-cubelit-border bg-cubelit-surface/50 opacity-50 cursor-not-allowed"
  >
    <div class="absolute top-2 right-2 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider bg-cubelit-border text-cubelit-muted rounded-full">
      Coming Soon
    </div>
    <div class="flex items-start gap-3">
      <div class="w-12 h-12 rounded-lg bg-cubelit-bg flex items-center justify-center text-cubelit-accent font-bold text-lg shrink-0 opacity-50">
        {tileMonogram()}
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
{:else}
  <button
    type="button"
    class="w-full text-left p-4 rounded-xl border-2 transition-all relative {selected
      ? 'border-cubelit-accent bg-cubelit-accent/10'
      : 'border-cubelit-border bg-cubelit-surface hover:border-cubelit-accent/40'}"
    onclick={onclick}
  >
    <div class="flex items-start gap-3">
      <div class="w-12 h-12 rounded-lg bg-cubelit-bg flex items-center justify-center text-cubelit-accent font-bold text-lg shrink-0">
        {tileMonogram()}
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
  </button>
{/if}
