import { invoke } from "@tauri-apps/api/core";
import type { RecipeSummary, Recipe } from "$lib/types/recipe";

export async function listRecipes(): Promise<RecipeSummary[]> {
  return invoke<RecipeSummary[]>("list_recipes");
}

export async function getRecipeDetail(id: string): Promise<Recipe> {
  return invoke<Recipe>("get_recipe_detail", { id });
}
