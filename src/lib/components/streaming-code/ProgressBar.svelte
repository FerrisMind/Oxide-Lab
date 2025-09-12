<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import CircleNotch from 'phosphor-svelte/lib/CircleNotch';
  import Code from 'phosphor-svelte/lib/Code';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import CaretUp from 'phosphor-svelte/lib/CaretUp';

  export let language: string = '';
  export let isStreaming: boolean = false;
  export let isExpanded: boolean = false;

  const dispatch = createEventDispatcher();

  function handleClick() {
    dispatch('click');
  }
  function handleCopyClick(e: Event) {
    e.stopPropagation();
    dispatch('copy');
  }

  function getLanguageDisplayName(lang: string): string {
    const langMap: Record<string, string> = {
      'js': 'JavaScript',
      'javascript': 'JavaScript',
      'ts': 'TypeScript',
      'typescript': 'TypeScript',
      'py': 'Python',
      'python': 'Python',
      'html': 'HTML',
      'css': 'CSS',
      'json': 'JSON',
      'xml': 'XML',
      'sql': 'SQL',
      'jsx': 'React JSX',
      'tsx': 'React TSX',
    };
    
    return langMap[lang.toLowerCase()] || lang.toUpperCase();
  }

  $: displayLanguage = getLanguageDisplayName(language);
  $: statusText = isStreaming ? 'Выполняется' : 'Готово';
  $: isInteractive = !isStreaming;
</script>

{#if isInteractive}
<button 
  class="progress-bar interactive" 
  class:streaming={isStreaming}
  class:expanded={isExpanded}
  on:click={handleClick}
  aria-label={`${isExpanded ? 'Свернуть' : 'Развернуть'} код ${displayLanguage}`}
  aria-expanded={isExpanded}
>
  <div class="progress-content">
    <div class="progress-icon">
      {#if isStreaming}
        <CircleNotch size={16} weight="regular" class="spinning" />
      {:else}
        <Code size={16} weight="regular" />
      {/if}
    </div>
    
    <div class="progress-text">
      <span class="language">{displayLanguage}</span>
      <span class="status" class:streaming-text={isStreaming}>{statusText}</span>
    </div>
    
    <div class="expand-icon">
      {#if isExpanded}
        <CaretUp size={16} weight="regular" />
      {:else}
        <CaretDown size={16} weight="regular" />
      {/if}
    </div>
    <!-- Copy control (use div to avoid nested button issues) -->
    <div class="copy-container">
      <div
        class="copy-btn"
        role="button"
        tabindex="0"
        on:click|stopPropagation={handleCopyClick}
        on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { handleCopyClick(e); } }}
        aria-label="Copy code"
      >
        Copy
      </div>
    </div>
  </div>
  
  {#if isStreaming}
    <div class="progress-indicator">
      <div class="progress-line"></div>
    </div>
  {/if}
</button>
{:else}
<div 
  class="progress-bar" 
  class:streaming={isStreaming}
  role="status"
  aria-label={`Генерация кода ${displayLanguage}: ${statusText}`}
>
  <div class="progress-content">
    <div class="progress-icon">
      {#if isStreaming}
        <CircleNotch size={16} weight="regular" class="spinning" />
      {:else}
        <Code size={16} weight="regular" />
      {/if}
    </div>
    
    <div class="progress-text">
      <span class="language">{displayLanguage}</span>
      <span class="status" class:streaming-text={isStreaming}>{statusText}</span>
    </div>
  </div>
  
  {#if isStreaming}
    <div class="progress-indicator">
      <div class="progress-line"></div>
    </div>
  {/if}
</div>
{/if}

<style>
  .progress-bar {
    display: flex;
    flex-direction: column;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
    transition: all 0.3s ease;
    position: relative;
    width: 100%;
    font-family: inherit;
    font-size: inherit;
    text-align: left;
  }

  .progress-bar.interactive {
    cursor: pointer;
    padding: 0;
  }

  .progress-bar.streaming {
    background: var(--accent-color-alpha);
    border-color: var(--accent-color);
  }



  .progress-bar.interactive:hover {
    background: var(--panel-alt-bg);
    border-color: var(--accent-color);
  }

  .progress-bar.interactive:focus {
    outline: 2px solid var(--accent-color);
    outline-offset: 2px;
  }

  .progress-content {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    gap: 12px;
  }

  .progress-icon {
    display: flex;
    align-items: center;
    color: var(--accent-color);
  }

  .progress-icon :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  .progress-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .language {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .status {
    font-size: 12px;
    color: var(--text-secondary);
    transition: color 0.3s ease;
  }

  .status.streaming-text {
    color: var(--accent-color);
  }

  .expand-icon {
    display: flex;
    align-items: center;
    color: var(--text-secondary);
    transition: color 0.3s ease;
  }

  .progress-bar.interactive:hover .expand-icon {
    color: var(--accent-color);
  }

  .progress-indicator {
    height: 2px;
    background: var(--border-color);
    position: relative;
    overflow: hidden;
  }

  .progress-line {
    height: 100%;
    background: var(--accent-color);
    width: 100%;
    animation: progress-slide 2s ease-in-out infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes progress-slide {
    0% {
      transform: translateX(-100%);
    }
    50% {
      transform: translateX(0%);
    }
    100% {
      transform: translateX(100%);
    }
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .progress-bar,
    .status,
    .expand-icon {
      transition: none;
    }
    
    .progress-icon :global(.spinning) {
      animation: none;
    }
    
    .progress-line {
      animation: none;
      transform: none;
    }
  }

  /* High contrast mode support */
  @media (prefers-contrast: high) {
    .progress-bar {
      border-width: 2px;
    }
    
    .progress-bar.streaming {
      background: transparent;
    }
  }
</style>