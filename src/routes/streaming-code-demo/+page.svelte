<script lang="ts">
  import { StreamingCodeDemo } from '$lib/components/streaming-code';
</script>

<svelte:head>
  <title>StreamingCodeBlock Demo</title>
  <meta name="description" content="Демонстрация компонента StreamingCodeBlock с Highlight.js и стримингом" />
</svelte:head>

<main>
  <div class="container">
    <header>
      <h1>StreamingCodeBlock Demo</h1>
      <p>Демонстрация подсветки Highlight.js со стримингом ответа</p>
    </header>
    
    <StreamingCodeDemo />
    
    <footer>
      <p>
        Этот компонент обеспечивает плавную синхронизацию между потоковой передачей кода 
        и его отображением с подсветкой синтаксиса на Highlight.js.
      </p>
    </footer>
  </div>
</main>

<style>
  main {
    min-height: 100vh;
    background: var(--bg-primary, #ffffff);
    color: var(--text-primary, #24292f);
  }

  .container {
    max-width: var(--content-max); /* 1200px */
    margin: 0 auto;
    padding: var(--space-6) var(--space-4); /* 40px 24px → 40px 20px closest */
  }

  header {
    text-align: center;
    margin-bottom: var(--space-6); /* 40px */
  }

  header h1 {
    font-size: var(--font-size-3xl); /* 40px → 2.5rem */
    font-weight: var(--font-weight-bold);
    margin-bottom: var(--space-3); /* 16px */
    color: var(--text-primary, #24292f);
  }

  header p {
    font-size: 1.125rem;
    color: var(--text-secondary, #656d76);
    max-width: 600px; /* 75 units */
    margin: 0 auto;
    line-height: var(--line-height-relaxed);
  }

  footer {
    margin-top: var(--space-9); /* 64px → 60px closest */
    padding-top: var(--space-6); /* 40px */
    border-top: 1px solid var(--border-color, #e1e5e9);
    text-align: center;
  }

  footer p {
    color: var(--text-secondary, #656d76);
    font-size: var(--font-size-sm); /* 14px → 0.875rem */
    line-height: var(--line-height-relaxed);
    max-width: 600px; /* 75 units */
    margin: 0 auto;
  }

  /* Dark theme */
  @media (prefers-color-scheme: dark) {
    main {
      background: var(--bg-primary, #0d1117);
      color: var(--text-primary, #f0f6fc);
    }
  }

  /* Responsive design */
  @media (max-width: 768px) {
    .container {
      padding: var(--space-4) var(--space-3); /* 24px 16px → 20px 16px closest */
    }

    header h1 {
      font-size: var(--font-size-2xl); /* 32px → 2rem */
    }

    header p {
      font-size: var(--font-size-base); /* 16px → 1rem */
    }
  }
</style>