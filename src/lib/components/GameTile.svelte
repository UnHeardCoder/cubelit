<script lang="ts">
  import { getGameDefinition } from '$lib/games/registry';
  import { GAME_ART } from '$lib/games/art';
  import GameArt from '$lib/components/GameArt.svelte';
  import type { RecipeSummary } from '$lib/types/recipe';

  interface Props {
    recipe: RecipeSummary;
    selected: boolean;
    comingSoon?: boolean;
    onclick?: () => void;
  }

  let { recipe, selected, comingSoon = false, onclick }: Props = $props();

  const gameDef = $derived(getGameDefinition(recipe.id));
  const mono = $derived(gameDef.tileMonogram ?? recipe.name.charAt(0));
</script>

{#if comingSoon}
  <div class="rounded-xl border-2 border-cubelit-border bg-cubelit-surface opacity-55 cursor-not-allowed overflow-hidden relative">
    <div class="absolute top-2 right-2 z-10">
      <span class="pill text-[10px]">Soon</span>
    </div>
    <GameArt recipeId={recipe.id} gameName={recipe.name} monogram={mono} height={110} variant="dim" showLogo={false} />
    <div class="p-3.5">
      <div class="text-sm font-semibold text-cubelit-text">{recipe.name}</div>
      <div class="text-xs text-cubelit-text-dim mt-0.5">{gameDef.cardStyle?.subtitle ?? recipe.description}</div>
    </div>
  </div>
{:else}
  <button
    type="button"
    onclick={onclick}
    class="w-full text-left rounded-xl border-2 overflow-hidden transition-all
      {selected ? 'border-cubelit-accent' : 'border-cubelit-border hover:border-cubelit-border-2'}"
  >
    <GameArt recipeId={recipe.id} gameName={recipe.name} monogram={mono} height={110} variant="dim" showLogo={false} />
    <div class="p-3.5 {selected ? 'bg-cubelit-accent/8' : 'bg-cubelit-surface'}">
      <div class="flex items-center justify-between">
        <div class="text-sm font-semibold text-cubelit-text">{recipe.name}</div>
      </div>
      <div class="text-xs text-cubelit-text-dim mt-0.5">{gameDef.cardStyle?.subtitle ?? recipe.description}</div>
      <div class="text-[11px] text-cubelit-muted font-mono mt-1.5 truncate">{recipe.tags.slice(0, 3).join(' · ')}</div>
    </div>
  </button>
{/if}
