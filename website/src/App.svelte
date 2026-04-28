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
  import Roadmap from './lib/components/Roadmap.svelte'

  let page = $state<'home' | 'audits' | 'roadmap'>('home')

  function navigate(path: string) {
    history.pushState({}, '', path)
    page = path.startsWith('/roadmap') ? 'roadmap' : path.startsWith('/audits') ? 'audits' : 'home'
  }

  onMount(() => {
    const p = window.location.pathname
    page = p.startsWith('/roadmap') ? 'roadmap' : p.startsWith('/audits') ? 'audits' : 'home'
    const handlePopState = () => {
      const p2 = window.location.pathname
      page = p2.startsWith('/roadmap') ? 'roadmap' : p2.startsWith('/audits') ? 'audits' : 'home'
    }
    window.addEventListener('popstate', handlePopState)
    return () => window.removeEventListener('popstate', handlePopState)
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
{:else if page === 'audits'}
  <Audits />
{:else if page === 'roadmap'}
  <Roadmap />
{/if}

<Footer />

<style>
  main {
    width: 100%;
  }
</style>
