<script lang="ts">

  import { onMount } from 'svelte';
  import { folderPath, scanFolder } from '$lib/stores/local-models';
  import { ModelCardsService } from '$lib/services/model-cards';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    filteredModelCards,
    importModelCards,
    loadModelCards,
    modelCardFilters,
    modelCardsError,
    modelCardsLoading,
    modelCardsVersion,
    resetModelCards,
    uniqueFamilies,
  } from '$lib/stores/model-cards';
  import {
    activeDownloads,
    downloadHistory,
    downloadsLoaded,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import type { ModelCardSummary } from '$lib/types/model-cards';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Heart from 'phosphor-svelte/lib/Heart';
  import Cube from 'phosphor-svelte/lib/Cube';
  import Gemma from '@lobehub/icons-static-svg/icons/gemma-color.svg';
  import Qwen from '@lobehub/icons-static-svg/icons/qwen-color.svg';
  import { t } from '$lib/i18n';

  function getModelIcon(card: ModelCardSummary) {
    const family = card.family?.toLowerCase() || '';
    const name = card.name.toLowerCase();
    
    if (family.includes('gemma') || name.includes('gemma')) {
      return Gemma;
    }
    if (family.includes('qwen') || name.includes('qwen')) {
      return Qwen;
    }
    
    // Default icon
    return 'cube';
  }

  let selectedCard: ModelCardSummary | null = $state(null);
  let downloadErrors: Record<string, string> = $state({});
  let downloadQueued: Record<string, boolean> = $state({});
  let selectedQuantizations: Record<string, string> = $state({});
  let selectedCardQuantizations: string[] = $state([]);
  let lastProcessedHistoryId: string | null = $state(null);

  onMount(() => {
    void loadModelCards();
    void ensureDownloadManager();
  });




  function getDownloadId(card: ModelCardSummary, format: 'gguf' | 'safetensors') {
    return `model-card::${card.id}::${format}`;
  }

  function jobGroupKey(job: { group_id?: string | null; repo_id: string; filename: string }) {
    return job.group_id ?? `${job.repo_id}::${job.filename}`;
  }

  function setQuantization(cardId: string, quant: string) {
    selectedQuantizations = { ...selectedQuantizations, [cardId]: quant };
  }

  async function handleDownload(card: ModelCardSummary, format: 'gguf' | 'safetensors') {
    if (!$folderPath) {
      alert($t('models.remote.selectFolderAlert'));
      return;
    }
    const downloadId = getDownloadId(card, format);
    downloadErrors = { ...downloadErrors, [downloadId]: '' };
    try {
      const quantization = format === 'gguf' ? selectedQuantizations[card.id] : undefined;
      downloadQueued = { ...downloadQueued, [downloadId]: true };
      await ModelCardsService.downloadModelCardFormat(card.id, format, $folderPath, quantization);
    } catch (error) {
      downloadErrors = {
        ...downloadErrors,
        [downloadId]: error instanceof Error ? error.message : String(error),
      };
      const next = { ...downloadQueued };
      delete next[downloadId];
      downloadQueued = next;
    }
  }

  function isDownloading(card: ModelCardSummary, format: 'gguf' | 'safetensors') {
    const key = getDownloadId(card, format);
    if (downloadQueued[key]) {
      return true;
    }
    if ($downloadsLoaded) {
      return $activeDownloads.some((job) => jobGroupKey(job) === key);
    }
    return false;
  }

  function updateFilter(filter: Partial<{ searchText: string; family: string; format: 'gguf' | 'safetensors' | '' }>) {
    modelCardFilters.update((prev) => ({ ...prev, ...filter }));
  }

  async function refreshCards() {
    await loadModelCards(true);
  }

  async function handleImportConfig() {
    try {
      const selected = (await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      })) as string | string[] | undefined;
      const path = Array.isArray(selected) ? selected[0] : selected;
      if (typeof path === 'string' && path.length > 0) {
        await importModelCards(path);
      }
    } catch (error) {
      console.error('Import failed', error);
    }
  }

  async function handleResetConfig() {
    await resetModelCards();
  }
  $effect(() => {
    if ($filteredModelCards.length && !selectedCard) {
      selectedCard = $filteredModelCards[0];
    }
  });
  $effect(() => {
    if (
      selectedCard &&
      !$filteredModelCards.find((card) => card.id === selectedCard?.id)
    ) {
      selectedCard = $filteredModelCards[0] ?? null;
    }
  });
  $effect(() => {
    selectedCardQuantizations = selectedCard?.gguf_quantizations ?? [];
  });
  $effect(() => {
    if (selectedCard && selectedCardQuantizations.length) {
      const current = selectedQuantizations[selectedCard.id];
      if (!current || !selectedCardQuantizations.includes(current)) {
        setQuantization(selectedCard.id, selectedCardQuantizations[0]);
      }
    }
  });
  $effect(() => {
    if ($downloadHistory.length) {
      const latest = $downloadHistory[0];
      if (latest?.group_id && downloadQueued[latest.group_id]) {
        const next = { ...downloadQueued };
        delete next[latest.group_id];
        downloadQueued = next;
      }
      if (
        latest &&
        $folderPath &&
        latest.id !== lastProcessedHistoryId &&
        latest.status === 'completed' &&
        latest.destination_path?.startsWith($folderPath)
      ) {
        lastProcessedHistoryId = latest.id;
        scanFolder($folderPath, true);
      }
    }
  });
</script>

<div class="remote-models-panel">
  <section class="search-bar">
    <input
      type="search"
      placeholder={$t('models.remote.searchPlaceholder')}
      value={$modelCardFilters.searchText}
      oninput={(event) => updateFilter({ searchText: (event.currentTarget as HTMLInputElement).value })}
    />
    <select
      value={$modelCardFilters.family}
      onchange={(event) => updateFilter({ family: (event.currentTarget as HTMLSelectElement).value })}
    >
      <option value="">{$t('models.remote.allFamilies')}</option>
      {#each $uniqueFamilies as family}
        <option value={family}>{family}</option>
      {/each}
    </select>
    <select
      value={$modelCardFilters.format}
      onchange={(event) => updateFilter({ format: (event.currentTarget as HTMLSelectElement).value as 'gguf' | 'safetensors' | '' })}
    >
      <option value="">{$t('models.remote.allFormats')}</option>
      <option value="gguf">GGUF</option>
      <option value="safetensors">safetensors</option>
    </select>
    <button class="btn primary" onclick={refreshCards} disabled={$modelCardsLoading}>
      {$modelCardsLoading ? $t('models.remote.refreshing') : $t('models.remote.refresh')}
    </button>
    <div class="config-actions">
      <button class="btn secondary" onclick={handleImportConfig} disabled={$modelCardsLoading}>
        {$t('models.remote.importConfig')}
      </button>
      <button class="btn secondary" onclick={handleResetConfig} disabled={$modelCardsLoading}>
        {$t('models.remote.resetConfig')}
      </button>
      <span class="config-version">
        {$t('models.remote.version')} {$modelCardsVersion ?? '—'}
      </span>
    </div>
  </section>

  {#if $modelCardsError}
    <div class="error-banner">
      <span>{$modelCardsError}</span>
      <button class="btn secondary" onclick={refreshCards}>{$t('models.remote.retry')}</button>
    </div>
  {/if}

  <section class="results">
    {#if $modelCardsLoading}
      <div class="loading-state">{$t('models.remote.loading')}</div>
    {:else if !$filteredModelCards.length}
      <div class="empty-state">
        <p>{$t('models.remote.noResults')}</p>
      </div>
    {:else}
      <div class="results-layout">
        <aside class="results-list">
          {#each $filteredModelCards as card (card.id)}
            <button
              type="button"
              class="results-item"
              class:selected={selectedCard?.id === card.id}
              onclick={() => (selectedCard = card)}
            >
              <div>
                <strong>{card.name}</strong>
                <div class="item-subtitle">{card.family ?? '—'} · {card.hf_repo_id}</div>
                <div class="tag-row">
                  {#each card.tags.slice(0, 3) as tag}
                    <span class="tag-pill">{tag}</span>
                  {/each}
                </div>
              </div>
              {#if isDownloading(card, 'gguf') || isDownloading(card, 'safetensors')}
                <div class="card-progress">
                  <span>{$t('models.remote.downloading')}</span>
                </div>
              {/if}
            </button>
          {/each}
        </aside>

        <div class="results-detail">
          {#if selectedCard}
            <article class="model-card">
              <header class="model-card__header">
                <div class="model-card__title">
                  <span class="model-card__icon" aria-hidden="true">
                    {#if getModelIcon(selectedCard) === 'cube'}
                      <Cube size={40} />
                    {:else}
                      <img src={getModelIcon(selectedCard)} alt="" width="40" height="40" />
                    {/if}
                  </span>
                  <div>
                    <h3>
                      <a href={`https://huggingface.co/${selectedCard.hf_repo_id}`} target="_blank" rel="noreferrer">
                        {selectedCard.name}
                      </a>
                    </h3>
                    <p class="model-card__repo">{selectedCard.hf_repo_id}</p>
                  </div>
                </div>
                <div class="model-card__stats">
                  <span class="stat-badge">
                    <Heart size={14} weight="bold" />
                    {selectedCard.tags.length || '—'}
                  </span>
                </div>
              </header>

              <p class="description">{selectedCard.description ?? $t('models.remote.noDescription')}</p>

              {#if selectedCard.sources}
                <div class="source-row">
                  {#if selectedCard.sources.gguf}
                    <span>{$t('models.remote.sources.gguf')} {selectedCard.sources.gguf.repo_id}</span>
                  {/if}
                  {#if selectedCard.sources.safetensors}
                    <span>{$t('models.remote.sources.safetensors')} {selectedCard.sources.safetensors.repo_id}</span>
                  {/if}
                </div>
              {/if}

              <div class="tag-row">
                {#each selectedCard.tags as tag}
                  <span class="tag-pill">{tag}</span>
                {/each}
              </div>

              <div class="formats">
                {#if selectedCardQuantizations.length}
                  <div class="quantization-row">
                    <label for={`quant-${selectedCard.id}`}>{$t('models.remote.quantization')}</label>
                    <select
                      id={`quant-${selectedCard.id}`}
                      value={selectedQuantizations[selectedCard.id] ?? selectedCardQuantizations[0]}
                    onchange={(event) => {
                      if (!selectedCard) return;
                      setQuantization(
                        selectedCard.id,
                        (event.currentTarget as HTMLSelectElement).value,
                      );
                    }}
                    >
                      {#each selectedCardQuantizations as quant}
                        <option value={quant}>{quant}</option>
                      {/each}
                    </select>
                  </div>
                {/if}
                {#if selectedCard.has_gguf}
                  <button
                    class="btn primary"
                    onclick={() => handleDownload(selectedCard!, 'gguf')}
                    disabled={isDownloading(selectedCard!, 'gguf')}
                  >
                    <DownloadSimple size={16} />
                    GGUF
                  </button>
                  {#if downloadErrors[getDownloadId(selectedCard!, 'gguf')]}
                    <p class="error-text">{downloadErrors[getDownloadId(selectedCard!, 'gguf')]}</p>
                  {/if}
                {/if}
                {#if selectedCard.has_safetensors}
                  <button
                    class="btn primary"
                    onclick={() => handleDownload(selectedCard!, 'safetensors')}
                    disabled={isDownloading(selectedCard!, 'safetensors')}
                  >
                    <DownloadSimple size={16} />
                    safetensors
                  </button>
                  {#if downloadErrors[getDownloadId(selectedCard!, 'safetensors')]}
                    <p class="error-text">{downloadErrors[getDownloadId(selectedCard!, 'safetensors')]}</p>
                  {/if}
                {/if}
              </div>

              <p class="destination">
                {#if $folderPath}
                  {$t('models.remote.modelsFolder')} <code>{$folderPath}</code>
                {:else}
                  {$t('models.remote.folderNotSelected')}
                {/if}
              </p>
            </article>
          {:else}
            <div class="detail-placeholder">
              {$t('models.remote.selectCard')}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </section>
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
    background: #1a1a1a;
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 12px;
    padding: 1rem;
    flex-wrap: wrap;
    box-shadow: var(--shadow);
  }

  .search-bar input,
  .search-bar select {
    padding: 0.6rem 0.8rem;
    border-radius: 10px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--panel-bg);
    min-width: 160px;
  }

  .config-actions {
    display: inline-flex;
    gap: 0.5rem;
    align-items: center;
  }

  .config-version {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .results {
    flex: 1;
    min-height: 0;
  }

  .results-layout {
    display: flex;
    height: 100%;
    gap: 1rem;
    min-height: 0;
  }

  .results-list {
    flex: 0 0 40%;
    max-width: 40%;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 14px;
    border: 1px solid var(--border-color, #e2e8f0);
    background: var(--panel-bg);
    overflow-y: auto;
    min-height: 0;
  }

  .results-item {
    width: 100%;
    border: none;
    border-radius: 10px;
    padding: 0.75rem;
    background: #1a1a1a;
    color: var(--text);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    transition: background 0.2s ease, transform 0.15s ease;
  }

  .results-item.selected {
    background: color-mix(in srgb, var(--accent, #3498db) 25%, transparent 75%);
  }

  .results-item:hover {
    transform: translateY(-1px);
    background: color-mix(in srgb, var(--accent, #3498db) 12%, transparent 88%);
  }

  .item-subtitle {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .tag-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.35rem;
  }

  .tag-pill {
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    font-size: 0.7rem;
    background: #1a1a1a;
    border: 1px solid var(--border-color, #d8dee5);
  }

  .card-progress {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .results-detail {
    flex: 0 0 60%;
    max-width: 60%;
    min-width: 0;
    overflow: auto;
  }

  .model-card {
    border-radius: 14px;
    padding: 1rem 1.25rem;
    border: 1px solid var(--border-color, #d8dee5);
    background: #1a1a1a;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    box-shadow: var(--shadow);
  }

  .source-row {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.8rem;
    color: var(--muted);
  }

  .model-card__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .model-card__title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .model-card__icon {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--accent, #3498db) 25%, transparent 75%);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .model-card__repo {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .model-card__stats {
    display: flex;
    gap: 0.45rem;
  }

  .stat-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    border-radius: 999px;
    border: 1px solid var(--border-color, #d8dee5);
    padding: 0.25rem 0.55rem;
  }

  .description {
    margin: 0;
    color: var(--muted);
    line-height: 1.5;
  }

  .formats {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .quantization-row {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 180px;
  }

  .quantization-row label {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .quantization-row select {
    padding: 0.5rem;
    border-radius: 12px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--panel-bg);
    font-size: 0.9rem;
  }

  .btn {
    padding: 0.5rem 0.85rem;
    border-radius: 10px;
    border: none;
    background: var(--accent, #3498db);
    color: #fff;
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
  }

  .btn.secondary {
    background: color-mix(in srgb, var(--accent, #3498db) 12%, transparent 88%);
    color: var(--accent, #3498db);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .destination {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .empty-state,
  .loading-state {
    padding: 1.5rem;
    border-radius: 12px;
    border: 1px dashed var(--border-color, #d8dee5);
    text-align: center;
    color: var(--muted);
  }

  .error-banner {
    padding: 0.75rem 1rem;
    border-radius: 10px;
    border: 1px solid color-mix(in srgb, var(--danger, #ef4444) 35%, transparent 65%);
    background: color-mix(in srgb, var(--danger, #ef4444) 12%, transparent 88%);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .error-text {
    margin: 0.35rem 0 0;
    font-size: 0.8rem;
    color: #ef4444;
  }

  @media (max-width: 1024px) {
    .results-layout {
      flex-direction: column;
    }

    .results-list {
      max-width: 100%;
      order: 2;
    }

    .results-detail {
      order: 1;
    }
  }
</style>
