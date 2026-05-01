<script lang="ts">
  import { GAME_ART, GAME_HUE } from '$lib/games/art';

  interface Props {
    recipeId: string;
    gameName: string;
    monogram?: string;
    height?: number;
    /** 'hero' = lighter overlay; 'dim' = darker overlay */
    variant?: 'hero' | 'dim';
    showLogo?: boolean;
    borderRadius?: number;
  }
  let {
    recipeId,
    gameName,
    monogram = '',
    height = 140,
    variant = 'hero',
    showLogo = true,
    borderRadius = 0,
  }: Props = $props();

  const art = $derived(GAME_ART[recipeId] ?? {});
  const hue = $derived(GAME_HUE[recipeId] ?? 30);
  const hasHero = $derived(!!art.hero);
</script>

<div
  style="
    width: 100%;
    height: {height}px;
    position: relative;
    border-radius: {borderRadius}px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    {hasHero
      ? `background-image: url(${art.hero}); background-size: cover; background-position: center;`
      : `background: linear-gradient(135deg, oklch(0.55 0.17 ${hue}) 0%, oklch(0.25 0.1 ${hue}) 100%);`
    }
  "
>
  {#if hasHero}
    <!-- Dimming overlay -->
    <div style="
      position: absolute; inset: 0;
      background: {variant === 'dim'
        ? 'linear-gradient(180deg, rgba(0,0,0,0.25) 0%, rgba(0,0,0,0.65) 100%)'
        : 'linear-gradient(180deg, rgba(0,0,0,0.1) 30%, rgba(0,0,0,0.55) 100%)'};
    "></div>
    <!-- Logo -->
    {#if showLogo && art.logo}
      <img
        src={art.logo}
        alt={gameName}
        style="position: relative; max-width: 62%; max-height: 62%; object-fit: contain; filter: drop-shadow(0 2px 8px rgba(0,0,0,0.5));"
      />
    {/if}
  {:else}
    <!-- Mono letter fallback -->
    <div style="
      position: relative;
      font-size: {height * 0.48}px;
      font-weight: 800;
      color: rgba(255,255,255,0.9);
      letter-spacing: -0.04em;
      font-family: var(--font-mono);
    ">{monogram || gameName.charAt(0)}</div>
  {/if}
</div>
