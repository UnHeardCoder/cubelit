<script lang="ts">
  import Spinner from "./Spinner.svelte";

  interface Props {
    step: string;
    progress: number | null;
    message: string;
  }

  let { step, progress, message }: Props = $props();

  const steps = ["preparing", "pulling", "creating", "starting", "ready"];

  function stepIndex(s: string): number {
    return steps.indexOf(s);
  }
</script>

<div class="max-w-md mx-auto text-center">
  <div class="mb-8">
    {#if step === "ready"}
      <div class="w-16 h-16 mx-auto rounded-full bg-cubelit-success/20 flex items-center justify-center mb-4">
        <svg class="w-8 h-8 text-cubelit-success" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
        </svg>
      </div>
    {:else}
      <Spinner size="lg" class="text-cubelit-accent mx-auto mb-4" />
    {/if}
    <p class="text-lg font-medium text-cubelit-text">{message}</p>
  </div>

  <!-- Progress bar -->
  {#if progress !== null}
    <div class="w-full bg-cubelit-bg rounded-full h-2 mb-6">
      <div
        class="h-2 rounded-full transition-all duration-500 {step === 'ready' ? 'bg-cubelit-success' : 'bg-cubelit-accent'}"
        style="width: {Math.round(progress * 100)}%"
      ></div>
    </div>
  {/if}

  <!-- Step indicators -->
  <div class="flex justify-between">
    {#each steps as s, i}
      <div class="flex flex-col items-center gap-1">
        <div
          class="w-3 h-3 rounded-full transition-colors {stepIndex(step) >= i ? (step === 'ready' ? 'bg-cubelit-success' : 'bg-cubelit-accent') : 'bg-cubelit-border'}"
        ></div>
        <span class="text-xs text-cubelit-muted capitalize">{s}</span>
      </div>
    {/each}
  </div>
</div>
