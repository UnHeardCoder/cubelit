<script lang="ts">
  import Navbar from './lib/components/Navbar.svelte'
  import Hero from './lib/components/Hero.svelte'
  import Features from './lib/components/Features.svelte'
  import SupportedGames from './lib/components/SupportedGames.svelte'
  import HowItWorks from './lib/components/HowItWorks.svelte'
  import TechStack from './lib/components/TechStack.svelte'
  import DownloadCTA from './lib/components/DownloadCTA.svelte'
  import Footer from './lib/components/Footer.svelte'
</script>

<Navbar />
<main>
  <Hero />
  <Features />
  <SupportedGames />
  <HowItWorks />
  <TechStack />
  <DownloadCTA />
</main>
<Footer />

<style>
  main {
    width: 100%;
  }
</style>
