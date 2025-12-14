<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';

  export const error: string | null = null;

  import { mount, unmount, onMount as _onMount } from 'svelte';
  import ModelList from './ModelList.svelte';
  import ModelDetail from './ModelDetail.svelte';
  import Robot from 'phosphor-svelte/lib/Robot';
  interface Props {
    models?: HFModel[];
    selectedModel?: HFModel | null;
    isLoading?: boolean;
    hasMore?: boolean;
    totalCount?: number;
    onSelectModel?: (detail: any) => void;
    onLoadMore?: () => void;
  }

  let {
    models = [],
    selectedModel = null,
    isLoading = false,
    hasMore = false,
    totalCount = 0,
    onSelectModel,
    onLoadMore,
  }: Props = $props();

  let noSelectionIconEl: HTMLElement | undefined = $state();
  let robotIcon: any = $state();

  function handleModelSelect(detail: { model: HFModel }) {
    onSelectModel?.(detail);
  }

  function handleLoadMore() {
    onLoadMore?.();
  }

  // Mount robot icon when no model is selected
  $effect(() => {
    if (noSelectionIconEl && !selectedModel) {
      if (robotIcon) {
        try {
          unmount(robotIcon);
        } catch {}
      }
      robotIcon = mount(Robot, {
        target: noSelectionIconEl,
        props: { size: 64, weight: 'regular' },
      });
    }
  });
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
        onSelectModel={handleModelSelect}
      />

      {#if hasMore}
        <div class="load-more-container">
          <button class="load-more-btn" onclick={handleLoadMore} disabled={isLoading}>
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
    gap: var(--space-4); /* 24px */
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
    border-radius: var(--radius-lg); /* 16px */
    overflow: hidden;
  }

  .panel-header {
    padding: var(--space-3); /* 16px */
    border-bottom: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }

  .panel-header h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
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
    padding: var(--space-3); /* 16px */
    border-top: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }

  .load-more-btn {
    width: 100%;
    padding: var(--space-2) var(--space-4); /* 8px 24px → 12px 24px closest */
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius-lg); /* 16px */
    font-size: var(--font-size-sm); /* 14px */
    font-weight: var(--font-weight-semibold);
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
    padding: var(--space-6) var(--space-3); /* 40px 16px → 40px 20px closest */
  }

  .no-selection-content {
    text-align: center;
    max-width: 304px; /* 38 units */
  }

  .no-selection-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-3); /* 16px */
    opacity: 0.5;
    color: var(--muted);
  }

  .no-selection-icon :global(svg) {
    color: inherit;
    fill: currentColor;
  }

  .no-selection h4 {
    margin: 0 0 var(--space-2) 0; /* 8px → 12px closest */
    font-size: 1.25rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
  }

  .no-selection p {
    margin: 0;
    color: var(--muted);
    line-height: var(--line-height-normal);
  }

  /* Адаптивность */
  @media (max-width: 1024px) {
    .search-layout {
      grid-template-columns: 1fr;
      gap: var(--space-3); /* 16px */
    }

    .models-panel {
      min-height: 304px; /* 38 units */
    }

    .details-panel {
      min-height: 400px; /* 50 units */
    }
  }

  @media (max-width: 768px) {
    .search-layout {
      gap: var(--space-3); /* 16px */
    }

    .panel-header {
      padding: var(--space-3); /* 16px */
    }

    .panel-header h3 {
      font-size: 1rem;
    }

    .load-more-container {
      padding: var(--space-3); /* 16px */
    }

    .no-selection {
      padding: var(--space-3); /* 16px → 20px closest */
    }
  }
</style>
