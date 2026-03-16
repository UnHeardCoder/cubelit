import { listRecipes } from "$lib/api/recipes";
import type { RecipeSummary } from "$lib/types/recipe";

let recipes = $state<RecipeSummary[]>([]);
let loaded = $state(false);

export function getRecipesStore() {
  async function load() {
    if (loaded) return;
    try {
      recipes = await listRecipes();
      loaded = true;
    } catch (e) {
      console.error("Failed to load recipes:", e);
    }
  }

  return {
    get recipes() {
      return recipes;
    },
    get loaded() {
      return loaded;
    },
    load,
  };
}
