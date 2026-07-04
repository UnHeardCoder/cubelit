<script lang="ts">
  import { onDestroy } from 'svelte';
  import { getOnboardingStatus, enableWsl2, openDockerDesktop, setWslDefaultVersion } from '$lib/api/system';
  import Cube from '$lib/components/Cube.svelte';
  import type { OnboardingStatus } from '$lib/types/docker';

  interface Props {
    status: OnboardingStatus | null;
    statusError?: string | null;
    checking: boolean;
    oncheck: () => void;
  }

  let { status = null, statusError = null, checking, oncheck }: Props = $props();

  type Step = 'checking' | 'enable_wsl2' | 'set_default_wsl2' | 'enabling_wsl2' | 'setting_default_wsl2' | 'reboot_required' | 'install_docker' | 'start_docker' | 'diagnostic_error';

  let currentStatus = $state<OnboardingStatus | null>(null);
  let actionError = $state<string | null>(null);
  let actionLoading = $state(false);
  let actionStep = $state<'enabling_wsl2' | 'setting_default_wsl2' | null>(null);
  let dockerLaunchError = $state<string | null>(null);
  let dockerLaunchLoading = $state(false);
  let pollTimer: ReturnType<typeof setTimeout> | null = null;
  let pollGeneration = 0;
  let mounted = true;

  const windowsStoreUrl = 'https://apps.microsoft.com/detail/xp8cbj40xlbwkx';
  const dockerDesktopUrl = 'https://www.docker.com/products/docker-desktop/';

  $effect(() => { currentStatus = status; });

  function isWindows(): boolean { return currentStatus?.platform === 'windows'; }

  function statusStep(): Step {
    if (!currentStatus) return checking ? 'checking' : 'diagnostic_error';
    if (currentStatus.platform === 'windows') {
      switch (currentStatus.wsl.state) {
        case 'reboot_required': return 'reboot_required';
        case 'needs_install':   return 'enable_wsl2';
        case 'needs_default_v2': return 'set_default_wsl2';
        case 'check_failed':   return 'diagnostic_error';
      }
    }
    switch (currentStatus.docker.state) {
      case 'ready':            return 'checking';
      case 'not_installed':    return 'install_docker';
      case 'not_running':      return 'start_docker';
      default:                 return 'diagnostic_error';
    }
  }

  function currentStep(): Step { return actionStep ?? statusStep(); }

  function diagnosticsMessage(): string {
    if (!currentStatus) return statusError?.trim() || "Cubelit couldn't verify your Docker setup yet.";
    if (currentStatus.platform === 'windows' && currentStatus.wsl.state === 'check_failed')
      return currentStatus.wsl.error?.trim() || "Cubelit couldn't verify your WSL setup.";
    return currentStatus.docker.error?.trim() || currentStatus.wsl.error?.trim() || "Cubelit couldn't verify your Docker setup.";
  }

  async function refreshStatus() {
    if (!mounted) return;
    currentStatus = await getOnboardingStatus();
  }

  function stopPolling() {
    pollGeneration++;
    if (pollTimer !== null) clearTimeout(pollTimer);
    pollTimer = null;
  }

  onDestroy(() => { mounted = false; stopPolling(); });

  function startPolling(waitForStep: 'enable_wsl2' | 'set_default_wsl2') {
    stopPolling();
    const generation = pollGeneration;
    let attempts = 0;
    const pollOnce = async () => {
      if (!mounted || generation !== pollGeneration) return;
      try {
        await refreshStatus();
        if (!mounted || generation !== pollGeneration) return;
        if (statusStep() !== waitForStep) {
          stopPolling();
          actionStep = null;
          oncheck();
          return;
        }
      } catch (e) { if (mounted && generation === pollGeneration) actionError = String(e); }
      attempts++;
      if (attempts >= 24) {
        stopPolling();
        if (mounted && generation === pollGeneration) {
          actionStep = null;
          actionError = 'Setup is taking longer than expected. Click Recheck after the command window closes.';
          oncheck();
        }
        return;
      }
      if (mounted && generation === pollGeneration) pollTimer = setTimeout(pollOnce, 5000);
    };
    pollTimer = setTimeout(pollOnce, 5000);
  }

  async function handleEnableWsl2() {
    actionLoading = true; actionError = null;
    try {
      await enableWsl2();
      if (!mounted) return;
      actionStep = 'enabling_wsl2';
      startPolling('enable_wsl2');
    } catch (e) { if (mounted) actionError = String(e); }
    finally { if (mounted) actionLoading = false; }
  }

  async function handleSetDefaultWsl2() {
    actionLoading = true; actionError = null;
    try {
      await setWslDefaultVersion();
      if (!mounted) return;
      actionStep = 'setting_default_wsl2';
      await refreshStatus();
      if (!mounted) return;
      if (statusStep() === 'set_default_wsl2') startPolling('set_default_wsl2');
      else { actionStep = null; oncheck(); }
    } catch (e) { if (mounted) { actionError = String(e); actionStep = null; } }
    finally { if (mounted) actionLoading = false; }
  }

  async function handleOpenDockerDesktop() {
    dockerLaunchLoading = true;
    dockerLaunchError = null;
    try {
      await openDockerDesktop();
    } catch (e) {
      dockerLaunchError = String(e);
    } finally {
      dockerLaunchLoading = false;
    }
  }

  function handleCheckAgain() { stopPolling(); actionStep = null; actionError = null; dockerLaunchError = null; oncheck(); }

  // Step status for progress indicator
  const wslDone = $derived(['set_default_wsl2', 'install_docker', 'start_docker', 'diagnostic_error'].includes(currentStep()) && !isWindows() ? true :
    currentStep() === 'install_docker' || currentStep() === 'start_docker' || (currentStatus?.wsl?.state === 'ok'));
  const showWslStep = $derived(isWindows());
</script>

<div class="min-h-screen bg-cubelit-bg flex items-center justify-center p-8">
  <div style="max-width: 640px; width: 100%; margin: 40px auto 0;">
    <!-- Brand header -->
    <div class="flex items-center gap-3.5 mb-6">
      <Cube size={42} />
      <div>
        <div class="text-xl font-semibold text-cubelit-text">Let's get Cubelit ready</div>
        <div class="text-sm text-cubelit-text-dim">We need{showWslStep ? ' WSL2 and' : ''} Docker on this machine before we can run servers.</div>
      </div>
    </div>

    <!-- Progress strip -->
    {#if showWslStep}
      <div class="flex gap-1.5 mb-1.5">
        <div class="flex-1 h-1 rounded-full" style="background: {wslDone ? 'var(--c-success)' : (currentStep() === 'enable_wsl2' || currentStep() === 'set_default_wsl2' || currentStep() === 'enabling_wsl2' || currentStep() === 'setting_default_wsl2' ? 'var(--c-accent)' : 'var(--c-border)')}"></div>
        <div class="flex-1 h-1 rounded-full" style="background: {currentStep() === 'install_docker' || currentStep() === 'start_docker' ? 'var(--c-accent)' : 'var(--c-border)'}"></div>
      </div>
      <div class="flex justify-between text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-3.5">
        <span>1 · WSL2</span>
        <span>2 · Docker engine</span>
      </div>
    {/if}

    <!-- Step card -->
    <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl overflow-hidden">

      {#if currentStep() === 'checking'}
        <div class="p-6 text-center">
          <svg class="w-10 h-10 mx-auto text-cubelit-muted animate-spin mb-3" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          <p class="text-sm text-cubelit-text-dim">Checking your setup…</p>
        </div>

      {:else if currentStep() === 'enable_wsl2'}
        <div class="p-5 border-b border-cubelit-border">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-accent);">1</div>
            <div class="flex-1">
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Enable WSL2 features</div>
              <div class="text-xs text-cubelit-text-dim">Cubelit enables Windows' WSL and Virtual Machine Platform features so Docker Desktop can use its WSL2 backend.</div>
              <div class="mt-3 px-3 py-2.5 bg-cubelit-bg-2 border border-cubelit-border rounded-lg font-mono text-xs text-cubelit-text-dim mb-3">
                $ wsl --status<span class="cursor-blink">_</span>
              </div>
              {#if actionError}
                <p class="text-xs text-cubelit-error bg-cubelit-error/10 rounded-lg px-3 py-2 mb-3">{actionError}</p>
              {/if}
              <div class="flex gap-2 flex-wrap">
                <button type="button" onclick={handleEnableWsl2} disabled={actionLoading}
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors disabled:opacity-50">
                  {actionLoading ? 'Enabling…' : 'Enable WSL2'}
                </button>
                <a href="https://learn.microsoft.com/windows/wsl/install" target="_blank" rel="noreferrer"
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:text-cubelit-text transition-colors">
                  Microsoft guide
                </a>
              </div>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'enabling_wsl2' || currentStep() === 'setting_default_wsl2'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center shrink-0" style="background: var(--c-accent);">
              <svg class="w-3.5 h-3.5 text-white animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
            </div>
            <div>
              <div class="text-sm font-medium text-cubelit-text mb-0.5">{currentStep() === 'enabling_wsl2' ? 'Enabling WSL2…' : 'Updating WSL default…'}</div>
              <div class="text-xs text-cubelit-text-dim">A system command window may have opened. Accept any Windows prompt. Checking every 5 seconds…</div>
              <button type="button" onclick={handleCheckAgain} class="text-xs text-cubelit-accent hover:brightness-110 transition-colors mt-2">Check Again</button>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'set_default_wsl2'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-accent);">1</div>
            <div class="flex-1">
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Set WSL default to version 2</div>
              <div class="text-xs text-cubelit-text-dim mb-3">WSL is installed but version 2 is not the default. Cubelit can fix this now.</div>
              {#if actionError}
                <p class="text-xs text-cubelit-error bg-cubelit-error/10 rounded-lg px-3 py-2 mb-3">{actionError}</p>
              {/if}
              <div class="flex gap-2">
                <button type="button" onclick={handleSetDefaultWsl2} disabled={actionLoading}
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors disabled:opacity-50">
                  {actionLoading ? 'Setting…' : 'Set Default to v2'}
                </button>
                <button type="button" onclick={handleCheckAgain} class="text-xs text-cubelit-accent hover:brightness-110 transition-colors">Check Again</button>
              </div>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'reboot_required'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-warning);">!</div>
            <div>
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Restart required</div>
              <div class="text-xs text-cubelit-text-dim mb-3">WSL features are enabled but your PC needs to restart. Restart from <strong class="text-cubelit-text">Start → Power → Restart</strong>, then reopen Cubelit.</div>
              <button type="button" onclick={handleCheckAgain} class="text-xs text-cubelit-accent hover:brightness-110 transition-colors">Check Again</button>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'install_docker'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-accent);">{showWslStep ? '2' : '1'}</div>
            <div class="flex-1">
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Check Docker Desktop</div>
              <div class="text-xs text-cubelit-text-dim mb-3">{isWindows() ? 'WSL2 is ready. Install Docker Desktop next; Docker will create its own WSL backend. You do not need to install Ubuntu separately.' : 'Cubelit needs Docker to create and manage game servers.'}</div>
              <div class="mt-2 px-3 py-2.5 bg-cubelit-bg-2 border border-cubelit-border rounded-lg font-mono text-xs text-cubelit-text-dim mb-3">
                $ docker version<span class="cursor-blink">_</span>
              </div>
              <div class="flex gap-2 flex-wrap">
                <button type="button" onclick={oncheck} disabled={checking}
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors disabled:opacity-50">
                  {checking ? 'Pinging engine…' : 'Check Docker'}
                </button>
                <a href={isWindows() ? windowsStoreUrl : dockerDesktopUrl} target="_blank" rel="noopener noreferrer"
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:text-cubelit-text transition-colors">
                  {isWindows() ? 'Get Docker Desktop' : 'Download Docker'}
                </a>
              </div>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'start_docker'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-accent);">{showWslStep ? '2' : '1'}</div>
            <div class="flex-1">
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Start Docker Desktop</div>
              <div class="text-xs text-cubelit-text-dim mb-3">Docker Desktop is installed but the engine is not running. Open Docker Desktop, wait for it to finish starting, then check again.</div>
              {#if currentStatus?.docker.error}
                <p class="text-xs text-cubelit-error bg-cubelit-error/10 rounded-lg px-3 py-2 mb-3">{currentStatus.docker.error}</p>
              {/if}
              {#if dockerLaunchError}
                <p class="text-xs text-cubelit-error bg-cubelit-error/10 rounded-lg px-3 py-2 mb-3">{dockerLaunchError}</p>
              {/if}
              <div class="flex gap-2 flex-wrap">
                {#if isWindows()}
                  <button type="button" onclick={handleOpenDockerDesktop} disabled={dockerLaunchLoading}
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors disabled:opacity-50">
                    {dockerLaunchLoading ? 'Opening…' : 'Open Docker Desktop'}
                  </button>
                {/if}
                <button type="button" onclick={handleCheckAgain} disabled={checking}
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:text-cubelit-text transition-colors disabled:opacity-50">
                  {checking ? 'Checking…' : 'Check Again'}
                </button>
              </div>
            </div>
          </div>
        </div>

      {:else if currentStep() === 'diagnostic_error'}
        <div class="p-5">
          <div class="flex gap-3.5 items-start">
            <div class="w-6 h-6 rounded-lg flex items-center justify-center text-xs font-bold text-white shrink-0" style="background: var(--c-warning);">!</div>
            <div class="flex-1">
              <div class="text-sm font-medium text-cubelit-text mb-0.5">Setup check failed</div>
              <div class="text-xs text-cubelit-text-dim mb-3">Cubelit couldn't determine your Docker or WSL state.</div>
              <pre class="text-xs text-cubelit-error bg-cubelit-error/10 rounded-lg px-3 py-2 mb-3 whitespace-pre-wrap break-words">{diagnosticsMessage()}</pre>
              <button type="button" onclick={oncheck} disabled={checking}
                class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors disabled:opacity-50">
                {checking ? 'Checking…' : 'Check Again'}
              </button>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="flex justify-between items-center mt-3.5">
      <button type="button" onclick={handleCheckAgain}
        class="inline-flex items-center gap-1.5 text-xs text-cubelit-text-dim hover:text-cubelit-text transition-colors">
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <path stroke-linecap="round" stroke-linejoin="round" d="M3 12a9 9 0 1 0 3-6.7M3 3v6h6"/>
        </svg>
        Recheck from start
      </button>
      <span class="text-[11px] text-cubelit-muted font-mono">cubelit · pre-flight</span>
    </div>
  </div>
</div>
