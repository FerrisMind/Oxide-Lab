<script lang="ts">
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Heart from 'phosphor-svelte/lib/Heart';
  import type { HFModel } from '$lib/services/huggingface';
  import { createEventDispatcher } from 'svelte';

  export let models: HFModel[] = [];
  export let selectedModelId: string | null = null;
  export let loading = false;

  const dispatch = createEventDispatcher<{
    selectModel: { model: HFModel };
  }>();

  function handleModelSelect(model: HFModel) {
    dispatch('selectModel', { model });
  }

  // Форматирование числа загрузок
  function formatDownloads(downloads: number): string {
    if (downloads >= 1000000) {
      return `${(downloads / 1000000).toFixed(1)}M`;
    } else if (downloads >= 1000) {
      return `${(downloads / 1000).toFixed(1)}K`;
    }
    return downloads.toString();
  }

  // Форматирование даты
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('ru-RU', {
      month: 'short',
      day: 'numeric'
    });
  }

  // Получение цвета для формата
  function getFormatColor(format: string): string {
    const formatColors: Record<string, string> = {
      'gguf': '#10b981',
      'safetensors': '#3b82f6',
      'pytorch': '#ee4b2b',
      'onnx': '#5a67d8'
    };
    return formatColors[format.toLowerCase()] || '#6b7280';
  }
</script>

<div class="model-list">
  {#if loading}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Загрузка моделей...</p>
    </div>
  {:else if models.length === 0}
    <div class="empty-state">
      <p>Модели не найдены</p>
      <span class="empty-hint">Попробуйте изменить параметры поиска</span>
    </div>
  {:else}
    <div class="models-container">
      {#each models as model (model.id)}
        <div 
          class="model-item" 
          class:selected={selectedModelId === model.id}
          on:click={() => handleModelSelect(model)}
          on:keydown={(e) => e.key === 'Enter' && handleModelSelect(model)}
          role="button"
          tabindex="0"
        >
          <div class="model-header">
            <div class="model-title-row">
              <div class="title-with-formats">
                <h4 class="model-name">{model.name}</h4>
                <div class="formats">
                  {#each model.formats.slice(0, 2) as format}
                    <span 
                      class="format-badge" 
                      style="background-color: {getFormatColor(format)}"
                    >
                      {format}
                    </span>
                  {/each}
                  {#if model.formats.length > 2}
                    <span class="format-badge more">+{model.formats.length - 2}</span>
                  {/if}
                </div>
              </div>
              <div class="title-stats">
                <span class="stat likes" title="Лайки">
                  <Heart size={12} /> {formatDownloads(model.likes)}
                </span>
                <span class="stat downloads" title="Загрузки">
                  <DownloadSimple size={12} /> {formatDownloads(model.downloads)}
                </span>
              </div>
            </div>
            
            <div class="model-info-row">
              <span class="model-author">by {model.author}</span>
              <div class="model-stats">
                <span class="last-modified">{formatDate(model.lastModified)}</span>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .model-list {
    height: 100%;
    overflow-y: auto;
    background: var(--card);
    border-right: 1px solid var(--border-color);
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--muted);
    text-align: center;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-color);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .empty-hint {
    font-size: 0.875rem;
    color: var(--muted);
    opacity: 0.8;
  }

  .models-container {
    padding: 8px;
  }

  .model-item {
    padding: 16px;
    margin-bottom: 8px;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    cursor: default;
    transition: all 0.2s ease;
    background: var(--card);
  }

  .model-item:hover {
    border-color: var(--accent);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }

  .model-item.selected {
    border-color: var(--accent);
    background: var(--panel-alt-bg);
    box-shadow: var(--shadow-hover);
  }

  .model-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .model-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .title-stats {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-shrink: 0;
  }

  .title-with-formats {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .model-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
    margin: 0;
    line-height: 1.4;
    flex-shrink: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .model-author {
    font-size: 0.875rem;
    color: var(--muted);
    margin: 0;
    flex: 1;
  }

  .model-stats {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .stat {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .formats {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .format-badge {
    font-size: 0.75rem;
    color: white;
    padding: 2px 6px;
    border-radius: 6px;
    font-weight: 500;
    white-space: nowrap;
  }

  .format-badge.more {
    background-color: var(--muted);
  }

  .last-modified {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    opacity: 0.8;
  }

  /* Скроллбар */
  .model-list::-webkit-scrollbar {
    width: 12px;
  }

  .model-list::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 6px;
  }

  .model-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.8);
    border-radius: 6px;
    border: 2px solid transparent;
    background-clip: content-box;
    transition: all 0.3s ease;
  }

  .model-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 1);
  }
</style>