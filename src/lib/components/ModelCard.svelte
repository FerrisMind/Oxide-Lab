<script lang="ts">
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Heart from 'phosphor-svelte/lib/Heart';
  interface Props {
    model: {
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
    onClick?: (event: MouseEvent) => void;
  }

  let { model, onClick }: Props = $props();

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
      day: 'numeric',
    });
  }

  // Получение цвета для тега
  function getTagColor(tag: string): string {
    const tagColors: Record<string, string> = {
      gguf: '#10b981',
      safetensors: '#3b82f6',
      llama: '#8b5cf6',
      mistral: '#f59e0b',
      gemma: '#ef4444',
      qwen: '#06b6d4',
    };
    return tagColors[tag.toLowerCase()] || '#6b7280';
  }
</script>

<div
  class="model-card"
  onclick={(e) => onClick?.(e)}
  onkeydown={(e) => {
    if (e.key === 'Enter') {
      e.currentTarget.click();
    }
  }}
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
      <span class="tag" style="background-color: {getTagColor(tag)}">
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
    <button class="add-model-btn" title="Добавить в менеджер моделей"> Добавить </button>
  </div>
</div>

<style>
  .model-card {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg); /* 16px */
    padding: var(--space-3); /* 16px → 20px closest */
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
    margin-bottom: var(--space-3); /* 16px */
  }

  .model-info {
    flex: 1;
    min-width: 0;
  }

  .model-name {
    font-size: var(--font-size-lg); /* 20px → 18px closest */
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-1) 0; /* 4px */
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-author {
    font-size: var(--font-size-sm); /* 14px */
    color: var(--muted);
    margin: 0;
  }

  .model-stats {
    display: flex;
    gap: var(--space-3); /* 16px */
    flex-shrink: 0;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: var(--space-2); /* 8px */
    font-size: var(--font-size-sm); /* 14px */
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
    font-size: var(--font-size-sm); /* 14px */
    line-height: var(--line-height-normal);
    margin: 0 0 var(--space-3) 0; /* 16px */
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
    gap: var(--space-2); /* 8px */
    margin-bottom: var(--space-3); /* 16px */
  }

  .tag {
    background: var(--accent);
    color: white;
    padding: var(--space-1) var(--space-2); /* 4px 8px */
    border-radius: var(--radius-lg); /* 16px */
    font-size: var(--font-size-base); /* 16px */
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  .tag.more {
    background: var(--muted);
    color: white;
  }

  .model-formats {
    display: flex;
    gap: var(--space-2); /* 8px */
    margin-bottom: var(--space-3); /* 16px */
  }

  .format-badge {
    padding: var(--space-1) var(--space-2); /* 4px 8px */
    border-radius: var(--radius-lg); /* 16px */
    font-size: var(--font-size-base); /* 16px */
    font-weight: var(--font-weight-semibold);
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
    padding-top: var(--space-3); /* 16px */
    border-top: 1px solid var(--border-color);
  }

  .last-modified {
    font-size: var(--font-size-base); /* 16px */
    color: var(--muted);
  }

  .add-model-btn {
    background: var(--accent-2);
    color: white;
    border: none;
    padding: var(--space-2) var(--space-3); /* 8px 16px */
    border-radius: var(--radius-lg); /* 16px */
    font-size: var(--font-size-sm); /* 14px */
    font-weight: var(--font-weight-semibold);
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
