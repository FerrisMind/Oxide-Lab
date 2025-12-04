<script lang="ts">

  import CodeMirrorDemo from '$lib/components/CodeMirrorDemo.svelte';
  import { mount, unmount } from 'svelte';
  import TestTube from 'phosphor-svelte/lib/TestTube';
  import CheckCircle from 'phosphor-svelte/lib/CheckCircle';
  
  let testIconEl: HTMLElement | undefined = $state();
  let testIcon: any = $state();
  
  $effect(() => {
    if (testIconEl) {
      if (testIcon) {
        try { unmount(testIcon); } catch {}
      }
      testIcon = mount(TestTube, {
        target: testIconEl,
        props: { size: 32, weight: 'regular' }
      });
    }
  });
</script>

<svelte:head>
  <title>CodeMirror Test - Oxide Lab</title>
</svelte:head>

<main class="test-page">
  <div class="container">
    <div class="header">
      <h1><span class="test-icon" bind:this={testIconEl}></span> CodeMirror Integration Test</h1>
      <p>This page demonstrates the CodeMirror integration for enhanced code display in chat messages.</p>
    </div>
    
    <CodeMirrorDemo />
    
    <div class="features">
      <h3>Features Implemented:</h3>
      <ul>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Automatic language detection for code blocks</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Syntax highlighting with theme support</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Copy button with Phosphor icons for each code block</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Line numbers and code folding</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Responsive design for mobile devices</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Integration with existing markdown renderer</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Support for JavaScript, Python, CSS, JSON, HTML, SQL, and more</li>
        <li class="feature-item"><CheckCircle size={16} weight="fill" class="check-icon" /> Automatic theme switching (light/dark)</li>
      </ul>
    </div>

    <div class="usage">
      <h3>How it works:</h3>
      <ol>
        <li>When the AI generates a response with code blocks, they are first processed by the markdown renderer</li>
        <li>The CodeMirror renderer then identifies code blocks and replaces them with interactive CodeMirror editors</li>
        <li>Language is automatically detected or specified via markdown code fence syntax</li>
        <li>Each code block gets a toolbar with language indicator and copy button (using Phosphor icons)</li>
        <li>The editor is read-only but fully interactive for scrolling and selection</li>
      </ol>
    </div>
  </div>
</main>

<style>
  .test-page {
    min-height: 100vh;
    background: var(--bg);
    padding: 20px;
  }

  .container {
    max-width: 900px;
    margin: 0 auto;
  }

  .header {
    text-align: center;
    margin-bottom: 40px;
    padding: 40px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .header h1 {
    font-size: 2.5rem;
    color: var(--text);
    margin: 0 0 16px 0;
    background: linear-gradient(135deg, var(--accent), #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }
  
  .test-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
  }
  
  .test-icon :global(svg) {
    color: inherit;
    fill: currentColor;
  }

  .header p {
    font-size: 1.2rem;
    color: var(--muted);
    margin: 0;
    max-width: 600px;
    margin: 0 auto;
  }

  .features, .usage {
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 32px;
    margin: 32px 0;
  }

  .features h3, .usage h3 {
    color: var(--text);
    margin: 0 0 20px 0;
    font-size: 1.3rem;
    font-weight: 600;
  }

  .features ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .features li {
    color: var(--text);
    padding: 8px 0;
    font-size: 1.1rem;
  }
  
  .feature-item {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  :global(.check-icon) {
    color: #10b981 !important;
    fill: currentColor !important;
    flex-shrink: 0;
  }

  .usage ol {
    color: var(--text);
    padding-left: 20px;
    margin: 0;
  }

  .usage li {
    padding: 6px 0;
    font-size: 1.1rem;
    line-height: 1.6;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .test-page {
      padding: 12px;
    }

    .header h1 {
      font-size: 2rem;
    }

    .header p {
      font-size: 1rem;
    }

    .features, .usage {
      padding: 20px;
    }

    .features h3, .usage h3 {
      font-size: 1.2rem;
    }

    .features li, .usage li {
      font-size: 1rem;
    }
  }
</style>