<script lang="ts">
  import Button from "./Button.svelte";

  interface Props {
    error: string | null;
    checking: boolean;
    oncheck: () => void;
  }

  let { error, checking, oncheck }: Props = $props();
</script>

<div class="min-h-screen bg-cubelit-bg flex items-center justify-center p-8">
  <div class="max-w-md text-center">
    <div class="mb-6">
      <svg class="w-20 h-20 mx-auto text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
      </svg>
    </div>

    <h2 class="text-2xl font-bold text-cubelit-text mb-3">Docker Not Found</h2>

    <p class="text-cubelit-muted mb-2">
      Cubelit needs Docker to create and manage game servers.
    </p>

    {#if error}
      <p class="text-sm text-cubelit-error/80 mb-6 bg-cubelit-error/10 rounded-lg p-3">
        {error}
      </p>
    {:else}
      <p class="text-cubelit-muted mb-6">
        Please install Docker Desktop and make sure it's running.
      </p>
    {/if}

    <div class="flex flex-col gap-3">
      <Button onclick={oncheck} loading={checking}>
        Check Again
      </Button>
      <a
        href="https://www.docker.com/products/docker-desktop/"
        target="_blank"
        rel="noopener noreferrer"
        class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
      >
        Download Docker Desktop
      </a>
    </div>
  </div>
</div>
