<script lang="ts">
  import type { RecipeSummary } from "$lib/types/recipe";
  import GameTile from "./GameTile.svelte";

  interface Props {
    recipes: RecipeSummary[];
    selectedId: string | null;
    onselect: (id: string) => void;
  }

  let { recipes, selectedId, onselect }: Props = $props();

  function availableRecipes(): RecipeSummary[] {
    return recipes.filter((r) => r.available).sort((a, b) => a.name.localeCompare(b.name));
  }

  function comingSoonRecipes(): RecipeSummary[] {
    return recipes.filter((r) => !r.available).sort((a, b) => a.name.localeCompare(b.name));
  }
</script>

<div>
  <h2 class="text-lg font-semibold text-cubelit-text mb-4">Choose a Game</h2>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
    {#each availableRecipes() as recipe (recipe.id)}
      <GameTile
        {recipe}
        selected={selectedId === recipe.id}
        onclick={() => onselect(recipe.id)}
      />
    {/each}
  </div>

  {#if comingSoonRecipes().length > 0}
    <div class="mt-8">
      <p class="text-sm text-cubelit-muted mb-3">Coming Soon</p>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each comingSoonRecipes() as recipe (recipe.id)}
          <GameTile {recipe} selected={false} comingSoon />
        {/each}
      </div>
    </div>
  {/if}
</div>
