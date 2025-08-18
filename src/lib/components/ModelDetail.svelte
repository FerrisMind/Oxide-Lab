<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { huggingFaceService } from '$lib/services/huggingface';
  import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
  import { onMount } from 'svelte';

  export let model: HFModel | null = null;
  export let loading = false;


  let detailedModel: HFModel | null = null;
  let detailsLoading = false;
  let tagsCollapsed = false;

  $: if (model) {
    loadModelDetails();
  }

  async function loadModelDetails() {
    if (!model) return;
    
    detailsLoading = true;
    try {
      const details = await huggingFaceService.getModelDetails(model.id);
      if (details) {
        detailedModel = details;
      }
    } catch (error) {
      console.error('Ошибка загрузки деталей модели:', error);
    } finally {
      detailsLoading = false;
    }
  }





  // Форматирование числа
  function formatNumber(num: number): string {
    if (num >= 1000000) {
      return `${(num / 1000000).toFixed(1)}M`;
    } else if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}K`;
    }
    return num.toString();
  }

  // Форматирование даты
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('ru-RU', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  // Получение цвета для тега
  function getTagColor(tag: string): string {
    const tagColors: Record<string, string> = {
      'gguf': '#10b981',
      'safetensors': '#3b82f6',
      'llama': '#8b5cf6',
      'mistral': '#f59e0b',
      'gemma': '#ef4444',
      'qwen': '#06b6d4',
      'pytorch': '#ee4b2b',
      'transformers': '#ff6b6b',
      'text-generation': '#4ecdc4',
      'conversational': '#45b7d1'
    };
    return tagColors[tag.toLowerCase()] || '#6b7280';
  }


</script>

<div class="model-detail">
  {#if loading}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Загрузка модели...</p>
    </div>
  {:else if !model}
    <div class="empty-state">
      <div class="empty-icon">🤖</div>
      <h3>Выберите модель</h3>
      <p>Выберите модель из списка слева, чтобы увидеть подробную информацию</p>
    </div>
  {:else}
    <div class="detail-content">
      <!-- Заголовок модели -->
      <div class="model-header">
        <div class="model-title">
          <h2 class="model-name">{model.name}</h2>
          <p class="model-author">by <strong>{model.author}</strong></p>
        </div>
        
        <div class="model-actions">
          <button class="btn btn-primary">
            <span class="btn-icon">⬇️</span>
            Скачать
          </button>
          <button class="btn btn-secondary">
            <span class="btn-icon">❤️</span>
            {formatNumber(model.likes)}
          </button>
        </div>
      </div>

      <!-- Статистика -->
      <div class="model-stats">
        <div class="stat-item">
          <span class="stat-label">Загрузки</span>
          <span class="stat-value">{formatNumber(model.downloads)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Лайки</span>
          <span class="stat-value">{formatNumber(model.likes)}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Обновлено</span>
          <span class="stat-value">{formatDate(model.lastModified)}</span>
        </div>
      </div>

      <!-- Теги -->
      {#if model.tags && model.tags.length > 0}
        <div class="model-tags">
          <div class="section-header">
            <button 
              class="collapse-btn" 
              on:click={() => tagsCollapsed = !tagsCollapsed}
              aria-label={tagsCollapsed ? 'Развернуть теги' : 'Свернуть теги'}
            >
              <h3>Теги</h3>
              <span class="collapse-icon" class:collapsed={tagsCollapsed}>▼</span>
            </button>
          </div>
          {#if !tagsCollapsed}
            <div class="tags-container">
              {#each model.tags as tag}
                <span 
                  class="tag" 
                  style="background-color: {getTagColor(tag)}"
                >
                  {tag}
                </span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Форматы -->
      {#if model.formats && model.formats.length > 0}
        <div class="model-formats">
          <h3>Доступные форматы</h3>
          <div class="formats-container">
            {#each model.formats as format}
              <div class="format-item">
                <span class="format-name">{format}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Дополнительная информация -->
      {#if detailedModel}
        <div class="additional-info">
          <h3>Дополнительная информация</h3>
          <div class="info-grid">
            {#if detailedModel.pipeline_tag}
              <div class="info-item">
                <span class="info-label">Pipeline Tag:</span>
                <span class="info-value">{detailedModel.pipeline_tag}</span>
              </div>
            {/if}
            {#if detailedModel.library_name}
              <div class="info-item">
                <span class="info-label">Библиотека:</span>
                <span class="info-value">{detailedModel.library_name}</span>
              </div>
            {/if}
            {#if detailedModel.license}
              <div class="info-item">
                <span class="info-label">Лицензия:</span>
                <span class="info-value">{detailedModel.license}</span>
              </div>
            {/if}
            {#if detailedModel.language}
              <div class="info-item">
                <span class="info-label">Язык:</span>
                <span class="info-value">
                  {Array.isArray(detailedModel.language) 
                    ? detailedModel.language.join(', ') 
                    : detailedModel.language}
                </span>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Описание -->
      {#if detailedModel?.description || model.description}
        <div class="model-description">
          <h3>Описание</h3>
          <div class="description-content">
            {@html renderMarkdownToSafeHtml(detailedModel?.description || model.description || '')}
          </div>
        </div>
      {:else if !detailsLoading}
        <div class="model-description">
          <h3>Описание</h3>
          <p class="no-description">Описание недоступно</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .model-detail {
    height: 100%;
    overflow-y: auto;
    background: var(--card);
    border: 1px solid var(--border-color);
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--muted);
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .empty-state h3 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--panel-alt-bg);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 12px;
  }

  .loading-spinner.small {
    width: 20px;
    height: 20px;
    border-width: 2px;
    margin-bottom: 0;
    margin-right: 8px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .detail-content {
    padding: 24px;
  }

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
    gap: 8px;
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

  .model-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
    padding: 16px;
    background: var(--panel-alt-bg);
    border-radius: 10px;
    border: 1px solid var(--border-color);
  }

  .stat-item {
    text-align: center;
  }

  .stat-label {
    display: block;
    font-size: 0.875rem;
    color: var(--muted);
    margin-bottom: 4px;
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .model-description,
  .model-tags,
  .model-formats,
  .additional-info {
    margin-bottom: 24px;
  }

  .model-description h3,
  .model-tags h3,
  .model-formats h3,
  .additional-info h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 12px 0;
  }

  .section-header {
    margin-bottom: 12px;
  }

  .collapse-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s ease;
    color: var(--muted);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .collapse-btn h3 {
    margin: 0;
  }

  .collapse-btn:hover {
    background: var(--panel-alt-bg);
    color: var(--text);
  }

  .collapse-icon {
    display: inline-block;
    transition: transform 0.2s ease;
    font-size: 0.875rem;
  }

  .collapse-icon.collapsed {
    transform: rotate(-90deg);
  }

  .model-description p {
    color: var(--text);
    line-height: 1.6;
    margin: 0;
    opacity: 0.8;
  }

  .description-content {
    color: var(--text);
    line-height: 1.6;
    opacity: 0.9;
  }

  .description-content h1,
  .description-content h2,
  .description-content h3,
  .description-content h4,
  .description-content h5,
  .description-content h6 {
    color: var(--text);
    margin: 1.5em 0 0.5em 0;
    font-weight: 600;
  }

  .description-content h1 { font-size: 1.5em; }
  .description-content h2 { font-size: 1.3em; }
  .description-content h3 { font-size: 1.2em; }
  .description-content h4 { font-size: 1.1em; }
  .description-content h5 { font-size: 1em; }
  .description-content h6 { font-size: 0.9em; }

  .description-content p {
    margin: 0.8em 0;
    line-height: 1.6;
  }

  .description-content ul,
  .description-content ol {
    margin: 0.8em 0;
    padding-left: 1.5em;
  }

  .description-content li {
    margin: 0.3em 0;
    line-height: 1.5;
  }

  .description-content blockquote {
    margin: 1em 0;
    padding: 0.8em 1em;
    border-left: 4px solid var(--accent);
    background: var(--panel-alt-bg);
    border-radius: 0 8px 8px 0;
    font-style: italic;
  }

  .description-content code {
    background: var(--panel-alt-bg);
    padding: 0.2em 0.4em;
    border-radius: 4px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.9em;
    color: var(--accent);
  }

  .description-content pre {
    background: var(--panel-alt-bg);
    padding: 1em;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1em 0;
    border: 1px solid var(--border-color);
  }

  .description-content pre code {
    background: none;
    padding: 0;
    color: var(--text);
  }

  .description-content a {
    color: var(--accent);
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.2s ease;
  }

  .description-content a:hover {
    border-bottom-color: var(--accent);
  }

  /* Таблицы внутри markdown-описания: снимаем скоупинг с внутренних тегов */
  .description-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .description-content :global(th),
  .description-content :global(td) {
    padding: 0.6em 1em;
    text-align: left;
    border: 1px solid var(--border-color);
  }

  .description-content :global(th) {
    background: var(--panel-alt-bg);
    font-weight: 600;
  }

  .description-content :global(tr:last-child) :global(td) {
    border-bottom: none;
  }

  .description-content strong,
  .description-content b {
    font-weight: 600;
    color: var(--text);
  }

  .description-content em,
  .description-content i {
    font-style: italic;
  }

  .description-content hr {
    border: none;
    height: 1px;
    background: var(--border-color);
    margin: 2em 0;
  }

  .no-description {
    font-style: italic;
    opacity: 0.7;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tag {
    font-size: 0.875rem;
    color: white;
    padding: 4px 12px;
    border-radius: 16px;
    font-weight: 500;
  }

  .formats-container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }

  .format-item {
    padding: 8px 12px;
    background: var(--panel-alt-bg);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    text-align: center;
    transition: all 0.2s ease;
  }

  .format-item:hover {
    background: #fcfbfa;
    border-color: var(--accent);
    transform: translateY(-1px);
  }

  .format-name {
    font-weight: 500;
    color: var(--accent);
  }



  .info-grid {
    display: grid;
    gap: 12px;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .info-item:last-child {
    border-bottom: none;
  }

  .info-label {
    font-weight: 500;
    color: var(--text);
  }

  .info-value {
    color: var(--muted);
    text-align: right;
  }

  /* Скроллбар */
  .model-detail::-webkit-scrollbar {
    width: 12px;
  }

  .model-detail::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 6px;
  }

  .model-detail::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.8);
    border-radius: 6px;
    border: 2px solid transparent;
    background-clip: content-box;
    transition: all 0.3s ease;
  }

  .model-detail::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 1);
  }
</style>