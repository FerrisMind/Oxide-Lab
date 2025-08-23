<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { formatNumber } from './formatters';
  import { createIconManager } from './icon-manager';
  import { onMount, onDestroy } from 'svelte';

  export let model: HFModel;

  let downloadIconEl: HTMLElement;
  let heartIconEl: HTMLElement;
  let huggingFaceIconEl: HTMLElement;

  const iconManager = createIconManager();

  // Mount action button icons when elements are ready
  $: if (downloadIconEl && heartIconEl && huggingFaceIconEl && model) {
    iconManager.mountActionIcons(downloadIconEl, heartIconEl, huggingFaceIconEl);
  }

  onDestroy(() => {
    iconManager.cleanup();
  });

  // Open model card on Hugging Face in browser
  async function openHuggingFaceCard() {
    if (!model) return;
    
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(`https://huggingface.co/${model.id}`);
    } catch (error) {
      console.error('Ошибка при открытии карточки модели на Hugging Face:', error);
    }
  }
</script>

<div class="model-header">
  <div class="model-title">
    <h2 class="model-name">{model.name}</h2>
    <p class="model-author">by <strong>{model.author}</strong></p>
  </div>
  
  <div class="model-actions">
    <div class="action-row">
      <button class="btn btn-primary">
        <span class="btn-icon" bind:this={downloadIconEl}></span>
        Скачать
      </button>
      <button class="btn btn-secondary">
        <span class="btn-icon" bind:this={heartIconEl}></span>
        {formatNumber(model.likes)}
      </button>
    </div>
    <button class="btn btn-secondary btn-hf" on:click={openHuggingFaceCard} title="Открыть на Hugging Face">
      <span class="btn-icon" bind:this={huggingFaceIconEl}></span>
      Hugging Face
    </button>
  </div>
</div>

<style>
  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .model-name {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 4px 0;
    line-height: 1.3;
  }

  .model-author {
    color: var(--muted);
    margin: 0;
    font-size: 1rem;
  }

  .model-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 200px;
  }

  .action-row {
    display: flex;
    gap: 8px;
  }

  .btn-hf {
    width: 100%;
  }

  @media (max-width: 768px) {
    .model-header {
      flex-direction: column;
      align-items: stretch;
      gap: 16px;
    }
    
    .model-title {
      text-align: center;
    }
    
    .model-actions {
      min-width: auto;
    }
  }

  @media (max-width: 480px) {
    .action-row {
      flex-direction: column;
    }
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    border-radius: 10px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-icon :global(svg) {
    color: inherit;
    fill: currentColor;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }

  .btn-secondary {
    background: var(--panel-alt-bg);
    color: var(--text);
    border: 1px solid var(--border-color);
  }

  .btn-secondary:hover {
    background: #fcfbfa;
    border-color: var(--accent);
    transform: translateY(-1px);
  }
</style>