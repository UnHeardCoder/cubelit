<script lang="ts">
  import { GAME_ART, GAME_HUE } from '$lib/games/art';
  import { getGameDefinition } from '$lib/games/registry';

  interface Props {
    recipeId: string;
    gameName?: string;
    size?: number;
    radius?: number;
  }
  let { recipeId, gameName = '', size = 32, radius = 8 }: Props = $props();

  const art = $derived(GAME_ART[recipeId] ?? {});
  const hue = $derived(GAME_HUE[recipeId] ?? 30);
  const mono = $derived(getGameDefinition(recipeId).tileMonogram ?? (gameName || recipeId).charAt(0).toUpperCase());
</script>

{#if art.icon}
  <img
    src={art.icon}
    alt={gameName}
    style="width: {size}px; height: {size}px; border-radius: {radius}px; object-fit: cover; display: block; flex-shrink: 0;"
  />
{:else}
  <div style="
    width: {size}px;
    height: {size}px;
    border-radius: {radius}px;
    flex-shrink: 0;
    background: linear-gradient(135deg, oklch(0.55 0.15 {hue}), oklch(0.3 0.1 {hue}));
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    font-weight: 700;
    font-size: {Math.round(size * 0.42)}px;
    font-family: var(--font-mono);
  ">{mono}</div>
{/if}
