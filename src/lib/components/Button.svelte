<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'md' | 'lg' | 'icon';
    disabled?: boolean;
    loading?: boolean;
    type?: 'button' | 'submit' | 'reset';
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    class?: string;
  }

  let {
    variant = 'primary',
    size = 'md',
    disabled = false,
    loading = false,
    type = 'button',
    onclick,
    children,
    class: className = '',
  }: Props = $props();

  const base = 'inline-flex items-center justify-center gap-1.5 font-medium transition-all focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap';

  const variants: Record<string, string> = {
    primary:   'btn-shimmer bg-cubelit-accent text-white border border-cubelit-accent hover:brightness-110 shadow-sm relative overflow-hidden',
    secondary: 'bg-cubelit-surface text-cubelit-text border border-cubelit-border hover:bg-cubelit-surface-2 hover:border-cubelit-border-2',
    danger:    'text-cubelit-error border border-cubelit-error/40 bg-cubelit-error/10 hover:bg-cubelit-error/18',
    ghost:     'text-cubelit-text-dim bg-transparent border border-transparent hover:text-cubelit-text hover:bg-cubelit-surface',
  };

  const sizes: Record<string, string> = {
    sm:   'px-2.5 py-1.5 text-xs rounded-md',
    md:   'px-3.5 py-2 text-sm rounded-lg',
    lg:   'px-5 py-2.5 text-sm rounded-lg',
    icon: 'p-2 w-8 h-8 rounded-lg text-sm',
  };
</script>

<button
  {type}
  class="{base} {variants[variant]} {sizes[size]} {className}"
  {disabled}
  {onclick}
>
  {#if loading}
    <div class="spin-voxel-stack shrink-0" style="width: 14px; height: 14px; gap: 1.5px;">
      {#each Array(9) as _}<i style="border-radius: 1px;"></i>{/each}
    </div>
  {/if}
  {@render children()}
</button>
