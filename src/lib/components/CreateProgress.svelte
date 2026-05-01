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

  // Boot stage status
  function stageStatus(stage: 'prepare' | 'pull' | 'boot'): 'done' | 'active' | 'todo' {
    const stageMap = {
      prepare: 0,
      pull:    1,
      boot:    3,
    };
    const threshold = stageMap[stage];
    if (currentIdx > threshold) return 'done';
    if (currentIdx === threshold) return 'active';
    return 'todo';
  }
</script>

<div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-6">
  <!-- Boot stages -->
  <div class="boot-stages mb-5">
    {#each [
      { key: 'prepare' as const, label: 'PREPARE' },
      { key: 'pull'    as const, label: 'PULL' },
      { key: 'boot'    as const, label: 'BOOT' },
    ] as s}
      <div class="st {stageStatus(s.key)}">
        <div class="lab">{s.label}</div>
        <div class="bar"></div>
      </div>
    {/each}
  </div>

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
