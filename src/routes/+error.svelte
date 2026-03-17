<script lang="ts">
  import { page } from "$app/state";
</script>

<div class="min-h-screen bg-cubelit-bg flex items-center justify-center p-8">
  <div class="max-w-md text-center">
    <p class="text-6xl font-bold text-cubelit-muted mb-4">{page.status}</p>
    <h1 class="text-xl font-semibold text-cubelit-text mb-2">Something went wrong</h1>
    <p class="text-cubelit-muted text-sm mb-8">{page.error?.message ?? "An unexpected error occurred."}</p>
    <a
      href="/"
      class="inline-flex items-center gap-2 px-4 py-2 bg-cubelit-accent hover:bg-cubelit-accent-hover text-white text-sm font-medium rounded-lg transition-colors"
    >
      Back to Home
    </a>
  </div>
</div>
