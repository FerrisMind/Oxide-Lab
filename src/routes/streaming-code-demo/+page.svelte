<script lang="ts">
  import { StreamingCodeDemo } from '$lib/components/streaming-code';
</script>

<svelte:head>
  <title>StreamingCodeBlock Demo</title>
  <meta name="description" content="Демонстрация компонента StreamingCodeBlock с синхронизацией CodeMirror и стриминга" />
</svelte:head>

<main>
  <div class="container">
    <header>
      <h1>StreamingCodeBlock Demo</h1>
      <p>Демонстрация синхронизации рендеринга CodeMirror со стримингом ответа</p>
    </header>
    
    <StreamingCodeDemo />
    
    <footer>
      <p>
        Этот компонент обеспечивает плавную синхронизацию между потоковой передачей кода 
        и его отображением в CodeMirror редакторе с поддержкой подсветки синтаксиса.
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
    max-width: 1200px;
    margin: 0 auto;
    padding: 40px 20px;
  }

  header {
    text-align: center;
    margin-bottom: 40px;
  }

  header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin-bottom: 16px;
    color: var(--text-primary, #24292f);
  }

  header p {
    font-size: 1.125rem;
    color: var(--text-secondary, #656d76);
    max-width: 600px;
    margin: 0 auto;
    line-height: 1.6;
  }

  footer {
    margin-top: 60px;
    padding-top: 40px;
    border-top: 1px solid var(--border-color, #e1e5e9);
    text-align: center;
  }

  footer p {
    color: var(--text-secondary, #656d76);
    font-size: 0.875rem;
    line-height: 1.6;
    max-width: 600px;
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
      padding: 20px 16px;
    }

    header h1 {
      font-size: 2rem;
    }

    header p {
      font-size: 1rem;
    }
  }
</style>