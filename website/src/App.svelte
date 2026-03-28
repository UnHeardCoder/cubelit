<script lang="ts">
  import { onMount } from 'svelte'
  import Navbar from './lib/components/Navbar.svelte'
  import Hero from './lib/components/Hero.svelte'
  import Features from './lib/components/Features.svelte'
  import SupportedGames from './lib/components/SupportedGames.svelte'
  import HowItWorks from './lib/components/HowItWorks.svelte'
  import TechStack from './lib/components/TechStack.svelte'
  import DownloadCTA from './lib/components/DownloadCTA.svelte'
  import Footer from './lib/components/Footer.svelte'
  import Audits from './lib/components/Audits.svelte'

  let page = $state<'home' | 'audits'>('home')

  function navigate(path: string) {
    history.pushState({}, '', path)
    page = path.startsWith('/audits') ? 'audits' : 'home'
  }

  onMount(() => {
    page = window.location.pathname.startsWith('/audits') ? 'audits' : 'home'
    window.addEventListener('popstate', () => {
      page = window.location.pathname.startsWith('/audits') ? 'audits' : 'home'
    })
  })
</script>

<Navbar {navigate} currentPage={page} />

{#if page === 'home'}
  <main>
    <Hero />
    <Features />
    <SupportedGames />
    <HowItWorks />
    <TechStack />
    <DownloadCTA />
  </main>
{:else}
  <Audits />
{/if}

<Footer />

<style>
  main {
    width: 100%;
  }
</style>
