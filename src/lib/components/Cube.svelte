<script lang="ts">
  interface Props {
    size?: number;
    color?: string;
  }
  let { size = 32, color = 'var(--c-accent)' }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 40 40"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  style="color: {color}; display: block; flex-shrink: 0;"
>
  <polygon points="20,2 36,11 36,29 20,38 4,29 4,11" fill="currentColor" fill-opacity="0.15" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
  <polygon points="20,2 36,11 20,20 4,11" fill="currentColor" fill-opacity="0.28" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
  <polygon points="4,11 20,20 20,38 4,29" fill="currentColor" fill-opacity="0.42" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
  <polygon points="36,11 20,20 20,38 36,29" fill="currentColor" fill-opacity="0.22" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
  <line x1="20" y1="2" x2="20" y2="20" stroke="currentColor" stroke-width="1" stroke-opacity="0.6" />
</svg>
