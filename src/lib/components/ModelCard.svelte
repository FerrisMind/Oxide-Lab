<script lang="ts">
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Heart from 'phosphor-svelte/lib/Heart';
  export let model: {
    id: string;
    name: string;
    description: string;
    downloads: number;
    likes: number;
    tags: string[];
    author: string;
    lastModified: string;
    modelType: string;
    formats: string[];
  };

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
      year: 'numeric',
      month: 'short',
      day: 'numeric'
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
      'qwen': '#06b6d4'
    };
    return tagColors[tag.toLowerCase()] || '#6b7280';
  }
</script>

<div 
  class="model-card" 
  on:click
  on:keydown={(e) => e.key === 'Enter' && e.currentTarget.click()}
  role="button"
  tabindex="0"
>
  <div class="model-header">
    <div class="model-info">
      <h3 class="model-name">{model.name}</h3>
      <p class="model-author">by {model.author}</p>
    </div>
    <div class="model-stats">
      <div class="stat">
        <DownloadSimple size={14} class="stat-icon" />
        <span class="stat-value">{formatDownloads(model.downloads)}</span>
      </div>
      <div class="stat">
        <Heart size={14} class="stat-icon" />
        <span class="stat-value">{formatDownloads(model.likes)}</span>
      </div>
    </div>
  </div>

  <p class="model-description">{model.description}</p>

  <div class="model-tags">
    {#each model.tags.slice(0, 6) as tag}
      <span 
        class="tag" 
        style="background-color: {getTagColor(tag)}"
      >
        {tag}
      </span>
    {/each}
    {#if model.tags.length > 6}
      <span class="tag more">+{model.tags.length - 6}</span>
    {/if}
  </div>

  <div class="model-formats">
    {#each model.formats as format}
      <span class="format-badge {format.toLowerCase()}">
        {format}
      </span>
    {/each}
  </div>

  <div class="model-footer">
    <span class="last-modified">Обновлено: {formatDate(model.lastModified)}</span>
    <button class="add-model-btn" title="Добавить в менеджер моделей">
      Добавить
    </button>
  </div>
</div>

<style>
  .model-card {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 20px;
    transition: all 0.2s ease;
     cursor: default;
    position: relative;
    overflow: hidden;
  }

  .model-card:hover {
    border-color: var(--accent);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
    transform: translateY(-2px);
  }

  .model-card:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(179, 205, 224, 0.3);
  }

  .model-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }

  .model-info {
    flex: 1;
    min-width: 0;
  }

  .model-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 4px 0;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-author {
    font-size: 14px;
    color: var(--muted);
    margin: 0;
  }

  .model-stats {
    display: flex;
    gap: 12px;
    flex-shrink: 0;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 14px;
    color: var(--muted);
  }

  .stat-icon {
    color: var(--muted);
    flex-shrink: 0;
  }

  .stat-value {
    font-weight: 500;
  }

  .model-description {
    color: var(--text);
    font-size: 14px;
    line-height: 1.5;
    margin: 0 0 16px 0;
    display: -webkit-box;
    /* standard property for line clamping */
    line-clamp: 3;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .model-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
  }

  .tag {
    background: var(--accent);
    color: white;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }

  .tag.more {
    background: var(--muted);
    color: white;
  }

  .model-formats {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .format-badge {
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .format-badge.gguf {
    background: #10b981;
    color: white;
  }

  .format-badge.safetensors {
    background: #3b82f6;
    color: white;
  }

  .model-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 16px;
    border-top: 1px solid var(--border-color);
  }

  .last-modified {
    font-size: 12px;
    color: var(--muted);
  }

  .add-model-btn {
    background: var(--accent-2);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
     cursor: default;
    transition: all 0.2s ease;
  }

  .add-model-btn:hover {
    background: var(--accent);
    transform: translateY(-1px);
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .model-card {
      background: var(--card);
      border-color: var(--border-color);
    }

    .model-card:hover {
      border-color: var(--accent);
      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    }
  }
</style>
