<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  
  export let models: HFModel[] = [];
  export let selectedModel: HFModel | null = null;
  export let isLoading: boolean = false;
  export let hasMore: boolean = false;
  export const error: string | null = null;
  export let totalCount: number = 0;
  
  import { createEventDispatcher, mount, unmount, onMount as _onMount } from 'svelte';
  import ModelList from './ModelList.svelte';
  import ModelDetail from './ModelDetail.svelte';
  import Robot from 'phosphor-svelte/lib/Robot';
  
  const dispatch = createEventDispatcher();
  let noSelectionIconEl: HTMLElement;
  let robotIcon: any;
  
  function handleModelSelect(event: CustomEvent) {
    dispatch('selectModel', event.detail);
  }
  
  function handleLoadMore() {
    dispatch('loadMore');
  }
  
  // Mount robot icon when no model is selected
  $: if (noSelectionIconEl && !selectedModel) {
    if (robotIcon) {
      try { unmount(robotIcon); } catch {}
    }
    robotIcon = mount(Robot, {
      target: noSelectionIconEl,
      props: { size: 64, weight: 'regular' }
    });
  }
</script>

<div class="search-layout">
  <!-- Левая панель - список моделей -->
  <div class="models-panel">
    <div class="panel-header">
      <h3>Модели ({totalCount > 0 ? totalCount : models.length})</h3>
    </div>
    <div class="panel-content">
      <ModelList 
        {models} 
        selectedModelId={selectedModel?.id}
        loading={isLoading}
        on:selectModel={handleModelSelect}
      />
      
      {#if hasMore}
        <div class="load-more-container">
          <button 
            class="load-more-btn" 
            on:click={handleLoadMore}
            disabled={isLoading}
          >
            {isLoading ? 'Загрузка...' : 'Загрузить ещё'}
          </button>
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Правая панель - детали модели -->
  <div class="details-panel">
    <div class="panel-header">
      <h3>Детали модели</h3>
    </div>
    <div class="panel-content">
      {#if selectedModel}
        <ModelDetail model={selectedModel} />
      {:else}
        <div class="no-selection">
          <div class="no-selection-content">
            <div class="no-selection-icon" bind:this={noSelectionIconEl}></div>
            <h4>Выберите модель</h4>
            <p>Выберите модель из списка слева, чтобы увидеть подробную информацию</p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .search-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
    flex: 1;
    height: 100%;
    width: 100%;
    min-height: 0;
    overflow: hidden;
  }
  
  .models-panel,
  .details-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }
  
  .panel-header h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .panel-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
  
  .load-more-container {
    padding: 16px;
    border-top: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }
  
  .load-more-btn {
    width: 100%;
    padding: 12px 24px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
     cursor: default;
    transition: all 0.2s ease;
  }
  
  .load-more-btn:hover:not(:disabled) {
    background: var(--accent-2);
    transform: translateY(-1px);
  }
  
  .load-more-btn:disabled {
    opacity: 0.6;
     cursor: default;
  }
  
  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
  }
  
  .no-selection-content {
    text-align: center;
    max-width: 300px;
  }
  
  .no-selection-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
    opacity: 0.5;
    color: var(--muted);
  }
  
  .no-selection-icon :global(svg) {
    color: inherit;
    fill: currentColor;
  }
  
  .no-selection h4 {
    margin: 0 0 12px 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .no-selection p {
    margin: 0;
    color: var(--muted);
    line-height: 1.5;
  }
  
  /* Адаптивность */
  @media (max-width: 1024px) {
    .search-layout {
      grid-template-columns: 1fr;
      gap: 16px;
    }
    
    .models-panel {
      min-height: 300px;
    }
    
    .details-panel {
      min-height: 400px;
    }
  }
  
  @media (max-width: 768px) {
    .search-layout {
      gap: 12px;
    }
    
    .panel-header {
      padding: 12px;
    }
    
    .panel-header h3 {
      font-size: 1rem;
    }
    
    .load-more-container {
      padding: 12px;
    }
    
    .no-selection {
      padding: 20px;
    }
  }
</style>