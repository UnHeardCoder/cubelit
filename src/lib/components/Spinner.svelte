<script lang="ts">
  interface Props {
    size?: "sm" | "md" | "lg";
    class?: string;
  }

  let { size = "md", class: className = "" }: Props = $props();

  const sizeClasses = {
    sm: "h-4 w-4",
    md: "h-6 w-6",
    lg: "h-8 w-8",
  };
</script>

<svg
  class="animate-spin {sizeClasses[size]} {className}"
  xmlns="http://www.w3.org/2000/svg"
  fill="none"
  viewBox="0 0 24 24"
>
  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
</svg>
