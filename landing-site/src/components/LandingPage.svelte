<script lang="ts">
  import '../styles/landing.css';
  import LandingHero from './LandingHero.svelte';
  import FeatureShowcase from './FeatureShowcase.svelte';
  import FlowTimeline from './FlowTimeline.svelte';
  import CTASection from './CTASection.svelte';
</script>

<div class="landing">
  <LandingHero />
  <FeatureShowcase />
  <FlowTimeline />
  <CTASection />
</div>

<style>
  .landing {
    flex: 1 0 auto;
    min-height: 100vh;
    min-height: 100svh;
    width: 100vw;
    min-width: 100vw;
    margin-left: calc(50% - 50vw);
    margin-right: calc(50% - 50vw);
    display: flex;
    flex-direction: column;
    align-items: stretch;
    background: radial-gradient(circle at top, rgba(76, 29, 149, 0.45), transparent 55%),
      radial-gradient(circle at bottom right, rgba(14, 116, 144, 0.25), transparent 45%),
      #0f172a;
    color: #e2e8f0;
    overflow-x: hidden;
  }

  .landing :global(section) {
    width: 100%;
    margin: 0;
    box-sizing: border-box;
  }

  @media (max-width: 640px) {
    .landing :global(section) {
      /* padding handled inside section-specific wrappers */
    }
  }
</style>
