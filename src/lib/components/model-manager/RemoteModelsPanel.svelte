<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { LocalModelsService } from '$lib/services/local-models';
  import { folderPath, scanFolder } from '$lib/stores/local-models';
  import {
    remoteResults,
    remoteIsLoading,
    remoteError,
    searchQuery,
    remoteFilters,
    searchRemoteModels,
    updateRemoteFilters,
    downloadRemoteModel,
  } from '$lib/stores/remote-models';
  import {
    activeDownloads as managerActiveDownloads,
    downloadHistory,
    downloadsLoaded,
    ensureDownloadManager,
    stopDownloadManager,
  } from '$lib/stores/download-manager';
  import type { RemoteGGUFFile, RemoteModelInfo } from '$lib/types/local-models';
  import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Clock from 'phosphor-svelte/lib/Clock';
  import Heart from 'phosphor-svelte/lib/Heart';
  import Cube from 'phosphor-svelte/lib/Cube';

  const QUANT_DESCRIPTIONS: Record<string, string> = {
    Q4_K_M:
      'Стандарт для ограниченных ресурсов. Хороший компромисс для устройств с малым объёмом памяти.',
    Q5_K_M:
      'Оптимальный баланс для большинства устройств. Высокое качество при умеренном объёме памяти.',
    Q6_K_M: 'Для приоритета качества. Практически идентичен оригиналу по качеству.',
    Q8_K_M:
      'Максимальная точность среди квантизованных. Оптимизированная версия с наивысшим качеством.',
  };

  const ALLOWED_QUANTIZATIONS = ['Q4_K_M', 'Q5_K_M', 'Q6_K_M', 'Q8_K_M'];

  const RECOMMENDED_QUANT = 'Q5_K_M';
  let destinationDir = '';
  let hasSearched = false;
  let selectedModel: RemoteModelInfo | null = null;
  let selectedQuantizations: Record<string, string> = {};
  let currentSelectedQuant: string | undefined;
  let currentQuantDescription: string | undefined;
  let readmeHtml = '';
  let readmeLoading = false;
  let readmeError: string | null = null;
  let lastReadmeRepoId: string | null = null;
  let initialHistoryPrepared = false;
  const readmeCache = new Map<string, string>();
  const notifiedCompletions = new Set<string>();

  onMount(async () => {
    await ensureDownloadManager();
  });

  onDestroy(() => {
    stopDownloadManager();
  });

  $: destinationDir = $folderPath || '';
  $: if ($remoteResults.length > 0 && !hasSearched) {
    hasSearched = true;
  }
  $: if ($remoteResults.length) {
    const existing =
      selectedModel && $remoteResults.find((model) => model.repo_id === selectedModel?.repo_id);
    if (!existing) {
      selectedModel = $remoteResults[0];
    }
  } else {
    selectedModel = null;
  }

  $: if ($remoteResults.length) {
    const next = { ...selectedQuantizations };
    let changed = false;
    for (const model of $remoteResults) {
      if (!next[model.repo_id]) {
        const defaultQuant = getDefaultQuantization(model);
        if (defaultQuant) {
          next[model.repo_id] = defaultQuant;
          changed = true;
        }
      }
    }
    if (changed) {
      selectedQuantizations = next;
    }
  }

  $: currentSelectedQuant = selectedModel ? getSelectedQuantization(selectedModel) : undefined;
  $: currentQuantDescription = currentSelectedQuant
    ? (QUANT_DESCRIPTIONS[currentSelectedQuant] ?? 'Описание недоступно')
    : undefined;

  $: if (selectedModel) {
    initializeSelectionForModel(selectedModel);
    if (lastReadmeRepoId !== selectedModel.repo_id) {
      lastReadmeRepoId = selectedModel.repo_id;
      loadReadmeForModel(selectedModel);
    } else if (readmeCache.has(selectedModel.repo_id)) {
      readmeHtml = readmeCache.get(selectedModel.repo_id) ?? '';
      readmeError = null;
      readmeLoading = false;
    }
  } else {
    readmeHtml = '';
    readmeError = null;
    readmeLoading = false;
    lastReadmeRepoId = null;
  }

  $: if ($downloadsLoaded && !initialHistoryPrepared) {
    for (const entry of $downloadHistory) {
      notifiedCompletions.add(entry.id);
    }
    initialHistoryPrepared = true;
  }

  $: if ($downloadsLoaded && initialHistoryPrepared && destinationDir) {
    for (const entry of $downloadHistory) {
      if (entry.status === 'completed' && !notifiedCompletions.has(entry.id)) {
        notifiedCompletions.add(entry.id);
        setTimeout(() => {
          scanFolder(destinationDir, true);
        }, 1500);
      }
    }
  }

  async function handleSearch() {
    hasSearched = true;
    await searchRemoteModels(true);
  }

  function canonicalizeQuantization(value?: string | null): string | null {
    if (!value) return null;
    let upper = value.toUpperCase();
    if (upper.includes('K_M') && !upper.includes('_K_M')) {
      upper = upper.replace('K_M', '_K_M');
    }
    return upper;
  }

  function getQuantizationOptions(model: RemoteModelInfo): string[] {
    return Array.from(
      new Set(
        (model.quantizations ?? [])
          .map((quant) => canonicalizeQuantization(quant))
          .filter((quant): quant is string => {
            if (!quant) return false;
            return ALLOWED_QUANTIZATIONS.includes(quant);
          }),
      ),
    );
  }

  function getDefaultQuantization(model: RemoteModelInfo): string | undefined {
    const options = getQuantizationOptions(model);
    if (!options.length) return undefined;
    if (options.includes(RECOMMENDED_QUANT)) {
      return RECOMMENDED_QUANT;
    }
    return options[0];
  }

  function initializeSelectionForModel(model: RemoteModelInfo) {
    if (!selectedQuantizations[model.repo_id]) {
      const defaultQuant = getDefaultQuantization(model);
      if (defaultQuant) {
        selectedQuantizations = { ...selectedQuantizations, [model.repo_id]: defaultQuant };
      }
    }
  }

  function getSelectedQuantization(model: RemoteModelInfo): string | undefined {
    const selected = selectedQuantizations[model.repo_id] ?? getDefaultQuantization(model);
    if (selected && ALLOWED_QUANTIZATIONS.includes(selected)) {
      return selected;
    }
    const options = getQuantizationOptions(model);
    return options.length ? options[0] : undefined;
  }

  function handleQuantizationChange(model: RemoteModelInfo, value: string) {
    selectedQuantizations = { ...selectedQuantizations, [model.repo_id]: value };
  }

  async function handleDownload(model: RemoteModelInfo, quant?: string) {
    if (!destinationDir) {
      alert('Сначала укажите папку в секции <Локальные> и повторите попытку.');
      goto('/models');
      return;
    }

    const effectiveQuant = quant ?? getSelectedQuantization(model);
    const file = effectiveQuant
      ? findFileForQuantization(model, effectiveQuant)
      : (model.gguf_files.find((item) => {
          const canonical = canonicalizeQuantization(item.quantization);
          return canonical ? ALLOWED_QUANTIZATIONS.includes(canonical) : false;
        }) ?? model.gguf_files[0]);

    if (!file) {
      alert('Нет доступных файлов для скачивания.');
      return;
    }

    try {
      await downloadRemoteModel(model.repo_id, destinationDir, file);
    } catch (error) {
      console.error('Download failed', error);
      alert(
        `Не удалось начать загрузку: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  function findFileForQuantization(
    model: RemoteModelInfo,
    quant: string,
  ): RemoteGGUFFile | undefined {
    const canonical = canonicalizeQuantization(quant);
    return model.gguf_files.find((file) => {
      const fileQuant = canonicalizeQuantization(file.quantization);
      if (fileQuant) {
        return fileQuant === canonical;
      }
      return Boolean(canonical) && file.filename.toUpperCase().includes(String(canonical));
    });
  }

  function uniqueArchitectures(result: RemoteModelInfo) {
    return Array.from(new Set(result.architectures));
  }

  function handleInputChange(event: Event, key: 'architecture' | 'license' | 'quantization') {
    const target = event.currentTarget as HTMLInputElement;
    updateRemoteFilters({ [key]: target.value || undefined });
  }

  function handleMinDownloadsChange(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    const value = target.value ? Number(target.value) : undefined;
    updateRemoteFilters({ min_downloads: value });
  }

  function handleSortChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    updateRemoteFilters({ sort_by: target.value as NonNullable<typeof $remoteFilters.sort_by> });
  }

  function handleSortOrderChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    updateRemoteFilters({
      sort_order: target.value as NonNullable<typeof $remoteFilters.sort_order>,
    });
  }

  function formatStat(value?: number) {
    if (!value) return '0';
    if (value >= 1_000_000) {
      return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
    }
    if (value >= 1_000) {
      return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
    }
    return value.toLocaleString();
  }

  function formatRelativeTime(dateString?: string | null) {
    if (!dateString) return null;
    const timestamp = Date.parse(dateString);
    if (Number.isNaN(timestamp)) return null;
    const diffMs = Date.now() - timestamp;
    const minutes = Math.floor(diffMs / 60000);
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    if (days < 30) return `${days}d ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months}mo ago`;
    const years = Math.floor(months / 12);
    return `${years}y ago`;
  }

  function formatUpdatedAt(model: RemoteModelInfo) {
    return formatRelativeTime(model.last_modified ?? model.created_at);
  }

  function formatContextLength(value?: number | null) {
    if (!value) return null;
    return `${value.toLocaleString()} ctx`;
  }

  function formatAuthor(author?: string | null) {
    return author?.trim() || 'Unknown author';
  }

  function openRepoLink(repoId: string) {
    return `https://huggingface.co/${repoId}`;
  }

  function handleSelectModel(model: RemoteModelInfo) {
    selectedModel = model;
  }

  async function loadReadmeForModel(model: RemoteModelInfo) {
    const repoId = model.repo_id;
    if (readmeCache.has(repoId)) {
      readmeHtml = readmeCache.get(repoId) ?? '';
      readmeLoading = false;
      readmeError = null;
      return;
    }
    const currentRepo = repoId;
    readmeLoading = true;
    readmeError = null;
    try {
      const markdown = await LocalModelsService.getModelReadme(repoId);
      const html = renderMarkdownToSafeHtml(markdown ?? '');
      readmeCache.set(repoId, html);
      if (selectedModel?.repo_id === currentRepo) {
        readmeHtml = html;
        readmeError = null;
      }
    } catch (error) {
      if (selectedModel?.repo_id === currentRepo) {
        readmeError = error instanceof Error ? error.message : String(error);
        readmeHtml = '';
      }
    } finally {
      if (selectedModel?.repo_id === currentRepo) {
        readmeLoading = false;
      }
    }
  }

</script>

<div class="remote-models-panel">
  <section class="search-bar">
    <input
      type="search"
      placeholder="Название модели или ключевые слова..."
      bind:value={$searchQuery}
      on:keydown={(event) => event.key === 'Enter' && handleSearch()}
    />
    <button class="btn" on:click={handleSearch} disabled={$remoteIsLoading}>
      {$remoteIsLoading ? 'Поиск...' : 'Поиск'}
    </button>
  </section>

  <section class="filters">
    <label>
      Архитектура
      <input
        type="text"
        placeholder="например, llama"
        value={$remoteFilters.architecture ?? ''}
        on:input={(event) => handleInputChange(event, 'architecture')}
      />
    </label>
    <label>
      Лицензия
      <input
        type="text"
        placeholder="например, mit"
        value={$remoteFilters.license ?? ''}
        on:input={(event) => handleInputChange(event, 'license')}
      />
    </label>
    <label>
      Квантизация
      <input
        type="text"
        placeholder="Q4, FP16..."
        value={$remoteFilters.quantization ?? ''}
        on:input={(event) => handleInputChange(event, 'quantization')}
      />
    </label>
    <label>
      Мин. загрузок
      <input
        type="number"
        min="0"
        value={$remoteFilters.min_downloads ?? ''}
        on:input={handleMinDownloadsChange}
      />
    </label>
    <label>
      Сортировать по
      <select on:change={handleSortChange}>
        <option value="downloads" selected={$remoteFilters.sort_by === 'downloads'}>
          Загрузки
        </option>
        <option value="likes" selected={$remoteFilters.sort_by === 'likes'}>Лайки</option>
        <option value="updated" selected={$remoteFilters.sort_by === 'updated'}>Обновление</option>
        <option value="file_size" selected={$remoteFilters.sort_by === 'file_size'}>Размер</option>
      </select>
    </label>
    <label>
      Порядок
      <select on:change={handleSortOrderChange}>
        <option value="desc" selected={$remoteFilters.sort_order !== 'asc'}>По убыванию</option>
        <option value="asc" selected={$remoteFilters.sort_order === 'asc'}>По возрастанию</option>
      </select>
    </label>
  </section>

  {#if $remoteError}
    <div class="error-banner">
      {$remoteError}
      <button class="btn secondary" on:click={handleSearch}>Повторить</button>
    </div>
  {/if}

  <section class:loading={$remoteIsLoading} class="results">
    {#if !$remoteResults.length && !$remoteIsLoading}
      <div class="empty-state">
        {#if hasSearched}
          <p>Модели не найдены. Попробуйте изменить фильтры или запрос.</p>
        {:else}
          <p>Введите запрос и нажмите «Поиск», чтобы найти доступные GGUF модели.</p>
        {/if}
      </div>
    {:else if $remoteResults.length}
      <div class="results-layout">
        <aside class="results-list">
          {#each $remoteResults as model (model.repo_id)}
            <button
              type="button"
              class="results-item"
              class:selected={selectedModel && selectedModel.repo_id === model.repo_id}
              on:click={() => handleSelectModel(model)}
            >
              <span class="item-name">{model.name}</span>
              <span class="item-meta">
                <span class="meta-stat" title="Лайки">
                  <Heart size={12} weight="bold" />
                  {formatStat(model.likes)}
                </span>
                <span class="meta-stat" title="Загрузки">
                  <DownloadSimple size={12} weight="bold" />
                  {formatStat(model.downloads)}
                </span>
              </span>
            </button>
          {/each}
        </aside>

        <div class="results-detail">
          {#if selectedModel}
            <article class="model-card">
              <header class="model-card__header">
                <div class="model-card__title">
                  <span class="model-card__icon" aria-hidden="true">
                    <Cube size={20} weight="fill" />
                  </span>
                  <div class="model-card__heading">
                    <h3>
                      <a
                        href={openRepoLink(selectedModel.repo_id)}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {selectedModel.name}
                      </a>
                    </h3>
                    <p class="model-card__repo">{selectedModel.repo_id}</p>
                  </div>
                </div>
                <div class="model-card__stats">
                  <span class="stat-badge" title="Лайки">
                    <Heart size={14} weight="bold" />
                    {formatStat(selectedModel.likes)}
                  </span>
                  <span class="stat-badge" title="Загрузки">
                    <DownloadSimple size={14} weight="bold" />
                    {formatStat(selectedModel.downloads)}
                  </span>
                </div>
              </header>

              <div class="model-card__meta">
                <span class="meta-pill">{formatAuthor(selectedModel.author)}</span>
                {#if formatUpdatedAt(selectedModel)}
                  <span class="meta-pill updated">
                    <Clock size={12} weight="bold" />
                    {formatUpdatedAt(selectedModel)}
                  </span>
                {/if}
                {#if uniqueArchitectures(selectedModel).length}
                  <span class="meta-pill">{uniqueArchitectures(selectedModel).join(', ')}</span>
                {/if}
                {#if selectedModel.license}
                  <span class="meta-pill">{selectedModel.license}</span>
                {/if}
                {#if selectedModel.parameter_count}
                  <span class="meta-pill">{selectedModel.parameter_count}</span>
                {/if}
                {#if formatContextLength(selectedModel.context_length)}
                  <span class="meta-pill">{formatContextLength(selectedModel.context_length)}</span>
                {/if}
              </div>

              {#if selectedModel.description}
                <p class="description">{selectedModel.description}</p>
              {/if}

              {#if getQuantizationOptions(selectedModel).length}
                <div class="quantization-select">
                  <label for="quant-select">Выберите квантизацию</label>
                  <select
                    id="quant-select"
                    bind:value={selectedQuantizations[selectedModel.repo_id]}
                    on:change={(event) =>
                      selectedModel &&
                      handleQuantizationChange(
                        selectedModel,
                        (event.currentTarget as HTMLSelectElement).value,
                      )}
                    disabled={$remoteIsLoading}
                  >
                    {#each getQuantizationOptions(selectedModel) as option}
                      <option value={option}>{option}</option>
                    {/each}
                  </select>
                  {#if currentQuantDescription}
                    <p class="quant-description">{currentQuantDescription}</p>
                  {/if}
                </div>

                <div class="download-actions">
                  <button
                    class="btn primary"
                    on:click={() =>
                      selectedModel &&
                      handleDownload(selectedModel, selectedQuantizations[selectedModel.repo_id])}
                    disabled={$remoteIsLoading}
                  >
                    Скачать выбранную квантизацию
                  </button>
                </div>
              {:else}
                <div class="quantization-warning">
                  <p>
                    Нет поддерживаемых квантизаций (Q4_K_M, Q5_K_M, Q6_K_M, Q8_K_M) для этой модели.
                  </p>
                </div>
              {/if}

              <section class="readme-section" aria-live="polite">
                <h4>README модели</h4>
                {#if readmeLoading}
                  <p class="readme-status">Загружаем README…</p>
                {:else if readmeError}
                  <p class="readme-status error">Не удалось загрузить README: {readmeError}</p>
                {:else if readmeHtml}
                  <div class="readme-content" aria-live="polite">
                    {@html readmeHtml}
                  </div>
                {:else}
                  <p class="readme-status">README недоступен для этой модели.</p>
                {/if}
              </section>
            </article>
          {:else}
            <div class="detail-placeholder">Выберите модель, чтобы увидеть подробности.</div>
          {/if}
        </div>
      </div>
    {/if}
  </section>

  {#if $managerActiveDownloads.length}
    <section class="downloads">
      <h4>Загрузки</h4>
      <ul>
        {#each $managerActiveDownloads as progress (progress.id)}
          <li>
            <div class="download-header">
              <strong>{progress.filename}</strong>
              <span>
                {#if progress.total_bytes}
                  {Math.round((progress.downloaded_bytes / progress.total_bytes) * 100)}%
                {:else}
                  —
                {/if}
              </span>
            </div>
            <div class="progress-bar">
              <div
                class="progress-bar-fill"
                style={`width: ${progress.total_bytes ? (progress.downloaded_bytes / progress.total_bytes) * 100 : 0}%`}
              ></div>
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</div>

<style>
  .remote-models-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
    min-height: 0;
    color: var(--text);
  }

  .search-bar {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    background: var(--card);
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 12px;
    padding: 1rem 1.25rem;
    box-shadow: var(--shadow);
  }

  .search-bar input {
    flex: 1;
    padding: 0.6rem 0.8rem;
    border-radius: 10px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--panel-bg);
  }

  .filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
    gap: 0.75rem;
    background: var(--card);
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 12px;
    padding: 1rem 1.25rem;
    box-shadow: var(--shadow);
  }

  .filters label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
  }

  .filters input,
  .filters select {
    padding: 0.45rem 0.6rem;
    border-radius: 8px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--card);
  }

  .error-banner {
    padding: 0.75rem 1rem;
    background: color-mix(in srgb, var(--danger) 12%, transparent 88%);
    border: 1px solid color-mix(in srgb, var(--danger) 35%, transparent 65%);
    border-radius: 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-height: 0;
    height: 100%;
  }

  .results.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .results-layout {
    display: flex;
    gap: 1rem;
    min-height: 0;
    height: 100%;
  }

  .results-list {
    flex: 0 0 40%;
    max-width: 40%;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 14px;
    border: 1px solid color-mix(in srgb, var(--border-color, #d8dee5) 45%, transparent 55%);
    background: color-mix(in srgb, var(--card) 95%, rgba(12, 16, 24, 0.85));
    box-shadow: 0 10px 24px rgb(15 23 42 / 0.18);
    overflow: auto;
  }

  .results-item {
    width: 100%;
    border: none;
    border-radius: 10px;
    padding: 0.6rem 0.75rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: color-mix(in srgb, var(--card) 85%, rgba(12, 16, 24, 0.9));
    color: var(--text);
    cursor: default;
    transition:
      background 0.2s ease,
      transform 0.15s ease;
    text-align: left;
    font-size: 0.85rem;
    gap: 0.75rem;
  }

  .results-item:hover {
    background: color-mix(in srgb, var(--accent, #3498db) 18%, transparent 82%);
    transform: translateY(-1px);
  }

  .results-item.selected {
    background: color-mix(in srgb, var(--accent, #3498db) 26%, transparent 74%);
    color: #fff;
  }

  .item-name {
    flex: 1;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    color: inherit;
  }

  .meta-stat {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.75rem;
    opacity: 0.85;
  }

  .results-detail {
    flex: 0 0 60%;
    max-width: 60%;
    min-width: 0;
    overflow: auto;
  }

  .detail-placeholder {
    padding: 1.5rem;
    border-radius: 12px;
    border: 1px dashed color-mix(in srgb, var(--border-color, #d8dee5) 55%, transparent 45%);
    background: color-mix(in srgb, var(--card) 95%, rgba(12, 16, 24, 0.85));
    color: var(--muted);
    text-align: center;
  }

  .empty-state {
    padding: 1.5rem;
    text-align: center;
    color: var(--muted);
    border: 1px dashed var(--border-color, #d8dee5);
    border-radius: 12px;
  }

  .model-card {
    border-radius: 14px;
    padding: 1rem 1.1rem;
    background: color-mix(in srgb, var(--card) 92%, rgba(8, 12, 20, 0.82));
    border: 1px solid color-mix(in srgb, var(--border-color, #d8dee5) 45%, transparent 55%);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    box-shadow: 0 12px 26px rgb(15 23 42 / 0.18);
    transition:
      transform 0.15s ease,
      box-shadow 0.15s ease;
  }

  .model-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 18px 36px rgb(15 23 42 / 0.24);
  }

  .model-card__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .model-card__title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .model-card__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--accent, #3498db) 22%, transparent 78%);
    color: var(--accent, #3498db);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent, #3498db) 35%, transparent 65%);
  }

  .model-card__heading h3 {
    margin: 0;
    font-size: 1rem;
  }

  .model-card__heading a {
    color: var(--text);
    text-decoration: none;
  }

  .model-card__heading a:hover {
    color: var(--accent, #3498db);
  }

  .model-card__repo {
    margin: 0;
    font-size: 0.8rem;
    color: var(--muted);
  }

  .model-card__stats {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .stat-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    font-size: 0.78rem;
    background: color-mix(in srgb, var(--accent, #3498db) 14%, transparent 86%);
    color: var(--accent, #3498db);
    border: 1px solid color-mix(in srgb, var(--accent, #3498db) 28%, transparent 72%);
  }

  .description {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .model-card__meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    font-size: 0.78rem;
  }

  .meta-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.22rem 0.55rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--border-color, #d8dee5) 18%, transparent 82%);
    color: color-mix(in srgb, var(--text) 92%, var(--muted) 8%);
    border: 1px solid color-mix(in srgb, var(--border-color, #d8dee5) 35%, transparent 65%);
    white-space: nowrap;
  }

  .meta-pill.updated {
    background: color-mix(in srgb, var(--accent, #3498db) 12%, transparent 88%);
    color: var(--accent, #3498db);
    border-color: color-mix(in srgb, var(--accent, #3498db) 28%, transparent 72%);
  }

  .quantization-select {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .quantization-select select {
    padding: 0.5rem 0.6rem;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--border-color, #d8dee5) 35%, transparent 65%);
    background: color-mix(in srgb, var(--card) 88%, rgba(12, 16, 24, 0.85));
    color: var(--text);
  }

  .quant-description {
    font-size: 0.9rem;
    color: var(--muted);
    margin: 0;
  }

  .quantization-warning {
    margin: 1rem 0;
    padding: 0.75rem 0.9rem;
    border-radius: 10px;
    background: color-mix(in srgb, var(--danger, #ef4444) 12%, transparent 88%);
    color: color-mix(in srgb, var(--danger, #ef4444) 75%, #fff 25%);
    font-size: 0.9rem;
  }

  .download-actions {
    margin-top: 1rem;
  }

  .readme-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid color-mix(in srgb, var(--border-color, #d8dee5) 35%, transparent 65%);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .readme-status {
    font-size: 0.9rem;
    color: var(--muted);
  }

  .readme-status.error {
    color: #ef4444;
  }

  .readme-content {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    max-height: 360px;
    overflow: auto;
  }

  .readme-content :global(h1),
  .readme-content :global(h2),
  .readme-content :global(h3) {
    margin: 0.8rem 0 0.4rem;
  }

  .readme-content :global(p) {
    margin: 0;
    line-height: 1.6;
  }

  .readme-content :global(pre) {
    padding: 0.65rem;
    background: rgba(15, 23, 42, 0.12);
    border-radius: 10px;
    overflow-x: auto;
  }

  .readme-content :global(code) {
    font-family: var(--code-font, 'Fira Code', monospace);
    font-size: 0.85rem;
    background: rgba(15, 23, 42, 0.12);
    padding: 0.15rem 0.3rem;
    border-radius: 6px;
  }

  .downloads {
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 12px;
    background: var(--card);
    padding: 0.75rem 1rem;
    box-shadow: var(--shadow);
  }

  .downloads ul {
    list-style: none;
    padding: 0;
    margin: 0.75rem 0 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .download-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 15%, transparent 85%);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent, #3498db);
    transition: width 0.2s ease;
  }

  .btn {
    padding: 0.45rem 0.8rem;
    border-radius: 8px;
    border: none;
    background: var(--accent, #3498db);
    color: #fff;
    cursor: default;
    font-size: 0.85rem;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .btn.secondary {
    background: color-mix(in srgb, var(--accent) 12%, transparent 88%);
    color: var(--accent, #3498db);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 1024px) {
    .results-layout {
      flex-direction: column;
    }

    .results-list {
      flex: 1 1 auto;
      max-width: 100%;
    }

    .results-detail {
      flex: 1 1 auto;
    }
  }
</style>
