<script lang="ts">
  // Fixed: Model descriptions now load properly on every model selection
  import type { HFModel } from '$lib/services/huggingface';
  import { huggingFaceService } from '$lib/services/huggingface';
  import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
  import { getCodeMirrorRenderer } from '$lib/chat/codemirror-renderer';
  import { enableExternalLinks } from '$lib/chat/external-links';
  import { onMount, mount, unmount, onDestroy } from 'svelte';
  import Robot from 'phosphor-svelte/lib/Robot';
  import Download from 'phosphor-svelte/lib/Download';
  import Heart from 'phosphor-svelte/lib/Heart';
  import ArrowSquareOut from 'phosphor-svelte/lib/ArrowSquareOut';

  export let model: HFModel | null = null;
  export let loading = false;


  let detailedModel: HFModel | null = null;
  let detailsLoading = false;
  let tagsCollapsed = false;
  let emptyIconEl: HTMLElement;
  let robotIcon: any;
  let downloadIconEl: HTMLElement;
  let heartIconEl: HTMLElement;
  let huggingFaceIconEl: HTMLElement;
  let downloadIcon: any;
  let heartIcon: any;
  let huggingFaceIcon: any;
  let descriptionEl: HTMLElement;
  let codeMirrorRenderer: any;
  let isDescriptionWatched = false;

  let currentModelId: string | null = null;

  $: handleModelChange(model);
  
  function handleModelChange(newModel: HFModel | null) {
    const newModelId = newModel?.id || null;
    
    if (newModelId !== currentModelId) {
      // Model has changed, cleanup previous state
      if (currentModelId !== null) {
        cleanupCodeMirror();
        detailedModel = null;
      }
      
      currentModelId = newModelId;
      
      if (newModel) {
        loadModelDetails();
      }
    }
  }
  
  function cleanupCodeMirror() {
    if (codeMirrorRenderer && isDescriptionWatched) {
      try {
        codeMirrorRenderer.stopWatching();
      } catch {}
    }
    isDescriptionWatched = false;
  }

  // Mount robot icon when component is ready
  $: if (emptyIconEl && !model) {
    if (robotIcon) {
      try { unmount(robotIcon); } catch {}
    }
    robotIcon = mount(Robot, {
      target: emptyIconEl,
      props: { size: 64, weight: 'regular' }
    });
  }

  // Mount action button icons when model is loaded
  $: if (downloadIconEl && model) {
    if (downloadIcon) {
      try { unmount(downloadIcon); } catch {}
    }
    downloadIcon = mount(Download, {
      target: downloadIconEl,
      props: { size: 16, weight: 'regular' }
    });
  }

  $: if (heartIconEl && model) {
    if (heartIcon) {
      try { unmount(heartIcon); } catch {}
    }
    heartIcon = mount(Heart, {
      target: heartIconEl,
      props: { size: 16, weight: 'regular' }
    });
  }

  $: if (huggingFaceIconEl && model) {
    if (huggingFaceIcon) {
      try { unmount(huggingFaceIcon); } catch {}
    }
    huggingFaceIcon = mount(ArrowSquareOut, {
      target: huggingFaceIconEl,
      props: { size: 16, weight: 'regular' }
    });
  }

  // Apply CodeMirror to description content when it's rendered
  $: setupCodeMirror(descriptionEl, detailedModel?.description || model?.description);
  
  function setupCodeMirror(element: HTMLElement, description: string | undefined) {
    if (element && description) {
      // Clean up previous CodeMirror setup
      cleanupCodeMirror();
      
      // Enable external links for the description content
      enableExternalLinks(element);
      
      try {
        if (!codeMirrorRenderer) {
          codeMirrorRenderer = getCodeMirrorRenderer();
        }
        codeMirrorRenderer.startWatching(element);
        isDescriptionWatched = true;
      } catch (error) {
        console.error('Failed to apply CodeMirror to model description:', error);
      }
    }
  }

  // Cleanup CodeMirror and icons when component is destroyed or model changes
  onDestroy(() => {
    cleanup();
  });
  
  function cleanup() {
    if (robotIcon) {
      try { unmount(robotIcon); } catch {}
      robotIcon = null;
    }
    if (downloadIcon) {
      try { unmount(downloadIcon); } catch {}
      downloadIcon = null;
    }
    if (heartIcon) {
      try { unmount(heartIcon); } catch {}
      heartIcon = null;
    }
    if (huggingFaceIcon) {
      try { unmount(huggingFaceIcon); } catch {}
      huggingFaceIcon = null;
    }
    cleanupCodeMirror();
  }

  async function loadModelDetails() {
    if (!model) return;
    
    const currentModelId = model.id;
    detailsLoading = true;
    
    try {
      const details = await huggingFaceService.getModelDetails(currentModelId);
      
      // Check if the model hasn't changed while we were loading
      if (model?.id === currentModelId && details) {
        detailedModel = details;
      }
    } catch (error) {
      console.error('Ошибка загрузки деталей модели:', error);
      // Only set detailedModel to null if we're still on the same model
      if (model?.id === currentModelId) {
        detailedModel = null;
      }
    } finally {
      // Only update loading state if we're still on the same model
      if (model?.id === currentModelId) {
        detailsLoading = false;
      }
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

  // Открытие карточки модели на Hugging Face в браузере
  async function openHuggingFaceCard() {
    if (!model) return;
    
    try {
      const { open } = await import('@tauri-apps/plugin-opener');
      await open(`https://huggingface.co/${model.id}`);
    } catch (error) {
      console.error('Ошибка при открытии карточки модели на Hugging Face:', error);
    }
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
      <div class="empty-icon" bind:this={emptyIconEl}></div>
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
            <span class="btn-icon" bind:this={downloadIconEl}></span>
            Скачать
          </button>
          <button class="btn btn-secondary">
            <span class="btn-icon" bind:this={heartIconEl}></span>
            {formatNumber(model.likes)}
          </button>
          <button class="btn btn-secondary" on:click={openHuggingFaceCard} title="Открыть на Hugging Face">
            <span class="btn-icon" bind:this={huggingFaceIconEl}></span>
            Hugging Face
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
          <div class="description-content" bind:this={descriptionEl}>
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
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1rem;
    color: var(--muted);
  }
  
  .empty-icon :global(svg) {
    color: inherit;
    fill: currentColor;
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

  /* Responsive images in markdown content */
  .description-content :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    margin: 1em 0;
    display: block;
    transition: all 0.2s ease;
  }

  .description-content :global(img:hover) {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    transform: translateY(-2px);
  }

  /* Handle images inside paragraphs or other containers */
  .description-content :global(p) :global(img) {
    margin: 0.5em 0;
  }

  /* Responsive image containers */
  .description-content :global(figure) {
    margin: 1em 0;
    text-align: center;
  }

  .description-content :global(figure) :global(img) {
    margin: 0 auto 0.5em;
  }

  .description-content :global(figcaption) {
    font-size: 0.875rem;
    color: var(--muted);
    font-style: italic;
    margin-top: 0.5em;
  }

  /* Responsive lists in markdown content */
  .description-content :global(ul),
  .description-content :global(ol) {
    margin: 1em 0;
    padding-left: 2em;
    line-height: 1.6;
  }

  .description-content :global(li) {
    margin: 0.5em 0;
    padding-left: 0.25em;
  }

  .description-content :global(ul) :global(li) {
    list-style-type: disc;
  }

  .description-content :global(ol) :global(li) {
    list-style-type: decimal;
  }

  /* Nested lists */
  .description-content :global(ul) :global(ul),
  .description-content :global(ol) :global(ol),
  .description-content :global(ul) :global(ol),
  .description-content :global(ol) :global(ul) {
    margin: 0.25em 0;
    padding-left: 1.5em;
  }

  .description-content :global(ul) :global(ul) :global(li) {
    list-style-type: circle;
  }

  .description-content :global(ul) :global(ul) :global(ul) :global(li) {
    list-style-type: square;
  }

  /* Additional markdown elements */
  .description-content :global(h1),
  .description-content :global(h2),
  .description-content :global(h3),
  .description-content :global(h4),
  .description-content :global(h5),
  .description-content :global(h6) {
    margin: 1.5em 0 0.75em 0;
    font-weight: 600;
    line-height: 1.3;
    color: var(--text);
  }

  .description-content :global(h1) { font-size: 1.8rem; }
  .description-content :global(h2) { font-size: 1.5rem; }
  .description-content :global(h3) { font-size: 1.3rem; }
  .description-content :global(h4) { font-size: 1.1rem; }
  .description-content :global(h5) { font-size: 1rem; }
  .description-content :global(h6) { font-size: 0.9rem; }

  /* Paragraphs */
  .description-content :global(p) {
    margin: 0.75em 0;
    line-height: 1.6;
  }

  /* Code elements */
  .description-content :global(code) {
    background: var(--code-bg);
    color: var(--code-fg);
    padding: 0.2em 0.4em;
    border-radius: 4px;
    font-family: ui-monospace, SFMono-Regular, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 0.875em;
  }

  .description-content :global(pre) {
    background: var(--code-bg);
    color: var(--code-fg);
    padding: 1em;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1em 0;
    font-family: ui-monospace, SFMono-Regular, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 0.875em;
    line-height: 1.4;
  }

  .description-content :global(pre) :global(code) {
    background: transparent;
    padding: 0;
    border-radius: 0;
  }

  /* Blockquotes */
  .description-content :global(blockquote) {
    margin: 1em 0;
    padding: 0.75em 1em;
    border-left: 4px solid var(--accent);
    background: var(--panel-alt-bg);
    border-radius: 0 8px 8px 0;
    font-style: italic;
    color: var(--text);
  }

  .description-content :global(blockquote) :global(p) {
    margin: 0;
  }

  /* Links */
  .description-content :global(a) {
    color: var(--accent);
    text-decoration: underline;
    transition: color 0.2s ease;
  }

  .description-content :global(a:hover) {
    color: var(--accent-hover);
  }

  /* Emphasis and strong */
  .description-content :global(em) {
    font-style: italic;
  }

  .description-content :global(strong) {
    font-weight: 600;
  }

  /* Strikethrough */
  .description-content :global(del),
  .description-content :global(s) {
    text-decoration: line-through;
    opacity: 0.7;
  }

  /* Horizontal rules */
  .description-content :global(hr) {
    border: none;
    height: 1px;
    background: var(--border-color);
    margin: 2em 0;
  }

  /* Details and summary */
  .description-content :global(details) {
    margin: 1em 0;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 0.5em 1em;
    background: var(--card);
  }

  .description-content :global(summary) {
    font-weight: 600;
    cursor: pointer;
    padding: 0.5em 0;
    color: var(--accent);
    user-select: none;
  }

  .description-content :global(summary:hover) {
    color: var(--accent-hover);
  }

  /* Keyboard, sample, and variable elements */
  .description-content :global(kbd) {
    background: var(--panel-alt-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 0.2em 0.4em;
    font-family: ui-monospace, SFMono-Regular, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 0.875em;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .description-content :global(samp),
  .description-content :global(var) {
    font-family: ui-monospace, SFMono-Regular, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-style: italic;
  }

  /* Abbreviations and definitions */
  .description-content :global(abbr),
  .description-content :global(dfn) {
    border-bottom: 1px dotted var(--muted);
    cursor: help;
  }

  /* Quotes */
  .description-content :global(q) {
    font-style: italic;
  }

  .description-content :global(q):before {
    content: '"';
  }

  .description-content :global(q):after {
    content: '"';
  }

  /* Citations */
  .description-content :global(cite) {
    font-style: italic;
    opacity: 0.8;
  }

  /* Subscript and superscript */
  .description-content :global(sub),
  .description-content :global(sup) {
    font-size: 0.75em;
    line-height: 0;
    position: relative;
    vertical-align: baseline;
  }

  .description-content :global(sub) {
    bottom: -0.25em;
  }

  .description-content :global(sup) {
    top: -0.5em;
  }

  /* Mark (highlight) */
  .description-content :global(mark) {
    background: #ffeb3b;
    color: #000;
    padding: 0.1em 0.2em;
    border-radius: 2px;
  }

  /* Small text */
  .description-content :global(small) {
    font-size: 0.875em;
    opacity: 0.8;
  }



  /* Responsive tables in markdown content */
  .description-content :global(table) {
    width: 100%;
    max-width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transition: all 0.2s ease;
    font-size: 0.875rem;
  }

  .description-content :global(table:hover) {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  }

  .description-content :global(th),
  .description-content :global(td) {
    padding: 0.75em 1em;
    text-align: left;
    border: 1px solid var(--border-color);
    word-wrap: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
  }

  .description-content :global(th) {
    background: var(--panel-alt-bg);
    font-weight: 600;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text);
  }

  .description-content :global(td) {
    background: var(--card);
  }

  .description-content :global(tr:nth-child(even)) :global(td) {
    background: var(--panel-alt-bg);
  }

  .description-content :global(tr:last-child) :global(td) {
    border-bottom: none;
  }

  /* Responsive table container for horizontal scrolling on small screens */
  .description-content :global(.table-responsive) {
    overflow-x: auto;
    margin: 1em 0;
  }

  /* Mobile-friendly table adjustments */
  @media (max-width: 768px) {
    .description-content :global(table) {
      font-size: 0.8rem;
    }

    .description-content :global(th),
    .description-content :global(td) {
      padding: 0.5em 0.75em;
    }

    .description-content :global(th) {
      font-size: 0.75rem;
    }
  }

  /* Very small screens - stack table cells vertically */
  @media (max-width: 480px) {
    .description-content :global(table),
    .description-content :global(thead),
    .description-content :global(tbody),
    .description-content :global(th),
    .description-content :global(td),
    .description-content :global(tr) {
      display: block;
    }

    .description-content :global(thead) :global(tr) {
      position: absolute;
      top: -9999px;
      left: -9999px;
    }

    .description-content :global(tr) {
      border: 1px solid var(--border-color);
      margin-bottom: 0.5em;
      border-radius: 8px;
      padding: 0.5em;
      background: var(--card);
    }

    .description-content :global(td) {
      border: none !important;
      position: relative;
      padding-left: 50% !important;
      padding-top: 0.5em;
      padding-bottom: 0.5em;
    }

    .description-content :global(td):before {
      content: attr(data-label) ': ';
      position: absolute;
      left: 6px;
      width: 45%;
      padding-right: 10px;
      white-space: nowrap;
      font-weight: 600;
      color: var(--muted);
    }
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