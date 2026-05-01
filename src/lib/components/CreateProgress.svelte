<script lang="ts">
  interface Props {
    step: string;
    progress: number | null;
    message: string;
    recipeName?: string;
    serverName?: string;
  }

  let { step, progress, message, recipeName = '', serverName = '' }: Props = $props();

  const steps = ['preparing', 'pulling', 'creating', 'starting', 'ready'] as const;
  type Step = typeof steps[number];

  const currentIdx = $derived(steps.indexOf(step as Step));

  const checkmarks: Record<string, string> = {
    pulling:  'recipe loaded',
    creating: 'image pulled',
    starting: 'container created',
    ready:    'port bound · server started',
  };
</script>

<div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-6">
  <!-- Terminal command line -->
  <div class="font-mono text-xs text-cubelit-text-dim mb-4">
    $ cubelit create {recipeName ? `--recipe ${recipeName}` : ''}{serverName ? ` --name "${serverName}"` : ''}
  </div>

  <!-- Progress bar -->
  {#if progress !== null}
    <div class="mb-4">
      <div class="flex justify-between text-xs mb-1.5">
        <span class="text-cubelit-text-dim">{message}</span>
        <span class="font-mono text-cubelit-muted">{Math.round(progress * 100)}%</span>
      </div>
      <div class="h-1.5 bg-cubelit-bg-2 rounded-full overflow-hidden">
        <div
          class="h-full rounded-full transition-all duration-500"
          style="width: {Math.round(progress * 100)}%; background: {step === 'ready' ? 'var(--c-success)' : 'var(--c-accent)'};"
        ></div>
      </div>
    </div>
  {/if}

  <!-- Terminal output -->
  <div class="font-mono text-xs leading-relaxed text-cubelit-text-dim space-y-1">
    {#each steps as s, i}
      {#if i <= currentIdx && checkmarks[s]}
        <div style="color: {step === 'ready' || i < currentIdx ? 'var(--c-success)' : 'var(--c-accent)'};">
          ✓ {checkmarks[s]}
        </div>
      {/if}
    {/each}
    {#if step !== 'ready'}
      <div class="text-cubelit-muted">{message}<span class="cursor-blink">_</span></div>
    {/if}
  </div>
</div>
