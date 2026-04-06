<script lang="ts">
  interface Props {
    licenseKey: string;
    onkeychange: (key: string) => void;
    oncontinue: () => void;
  }

  let { licenseKey = $bindable(""), onkeychange, oncontinue }: Props = $props();

  async function openKeymaster() {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl("https://keymaster.fivem.net/");
    } catch (e) {
      console.error("Failed to open URL:", e);
    }
  }
</script>

<div class="space-y-8 max-w-lg mx-auto">
  <div class="text-center">
    <div class="w-16 h-16 rounded-2xl bg-orange-900/30 border border-orange-500/20 flex items-center justify-center mx-auto mb-4">
      <svg class="w-8 h-8 text-cubelit-accent" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 0 1 3 3m3 0a6 6 0 0 1-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1 1 21.75 8.25Z" />
      </svg>
    </div>
    <h2 class="text-xl font-bold text-cubelit-text">FiveM License Key Required</h2>
    <p class="text-cubelit-muted mt-2 text-sm leading-relaxed">
      FiveM requires a free license key from the Cfx.re Keymaster portal. You'll need a Cfx.re account to generate one.
    </p>
  </div>

  <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-4 space-y-3">
    <p class="text-sm text-cubelit-text font-medium">How to get your key:</p>
    <ol class="text-sm text-cubelit-muted space-y-1.5 list-decimal list-inside">
      <li>Visit the Cfx.re Keymaster portal</li>
      <li>Sign in or create a free account</li>
      <li>Click "Register a new server"</li>
      <li>Fill in your server details</li>
      <li>Copy the generated license key</li>
    </ol>
    <button
      type="button"
      class="w-full px-4 py-2.5 bg-cubelit-accent text-white rounded-lg hover:bg-cubelit-accent-hover transition-colors text-sm font-medium"
      onclick={openKeymaster}
    >
      Open Cfx.re Keymaster
    </button>
  </div>

  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text" for="fivem-license-key">Paste your license key</label>
    <input
      id="fivem-license-key"
      type="text"
      class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors font-mono text-sm"
      bind:value={licenseKey}
      placeholder="cfxk_xxxxxxxxxxxxxxxxxxxx"
      oninput={(e) => { licenseKey = (e.target as HTMLInputElement).value; onkeychange(licenseKey); }}
      onpaste={(e) => {
        setTimeout(() => { licenseKey = (e.target as HTMLInputElement).value; onkeychange(licenseKey); }, 0);
      }}
    />
  </div>

  <button
    type="button"
    class="w-full px-4 py-3 bg-cubelit-accent text-white rounded-xl hover:bg-cubelit-accent-hover transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
    disabled={!licenseKey.trim()}
    onclick={oncontinue}
  >
    Continue
  </button>
</div>
