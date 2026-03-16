<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary" | "danger" | "ghost";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    loading?: boolean;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    class?: string;
  }

  let {
    variant = "primary",
    size = "md",
    disabled = false,
    loading = false,
    onclick,
    children,
    class: className = "",
  }: Props = $props();

  const baseClasses = "inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-cubelit-accent/50 disabled:opacity-50 disabled:cursor-not-allowed";

  const variantClasses = {
    primary: "bg-cubelit-accent text-white hover:bg-cubelit-accent-hover",
    secondary: "bg-cubelit-surface text-cubelit-text border border-cubelit-border hover:bg-cubelit-border",
    danger: "bg-cubelit-error/10 text-cubelit-error border border-cubelit-error/30 hover:bg-cubelit-error/20",
    ghost: "text-cubelit-muted hover:text-cubelit-text hover:bg-cubelit-surface",
  };

  const sizeClasses = {
    sm: "px-3 py-1.5 text-sm",
    md: "px-4 py-2 text-sm",
    lg: "px-6 py-3 text-base",
  };
</script>

<button
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
  {disabled}
  {onclick}
>
  {#if loading}
    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
    </svg>
  {/if}
  {@render children()}
</button>
