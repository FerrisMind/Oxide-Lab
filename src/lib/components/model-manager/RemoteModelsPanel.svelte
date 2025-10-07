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
    activeDownloads,
    ensureProgressListener,
    stopProgressListener,
    searchRemoteModels,
    updateRemoteFilters,
    downloadRemoteModel,
  } from '$lib/stores/remote-models';
  import type { RemoteGGUFFile, RemoteModelInfo } from '$lib/types/local-models';

  let destinationDir = '';

  onMount(async () => {
    await ensureProgressListener();
    if (!$remoteResults.length) {
      await searchRemoteModels();
    }
  });

  onDestroy(() => {
    stopProgressListener();
  });

  $: destinationDir = $folderPath;

  async function handleSearch() {
    await searchRemoteModels(true);
  }

  function formatFileSize(bytes: number) {
    return LocalModelsService.formatFileSize(bytes);
  }

  async function handleDownload(model: RemoteModelInfo, file: RemoteGGUFFile) {
    if (!destinationDir) {
      alert('Сначала выберите папку с локальными моделями на вкладке «Мои модели».');
      goto('/models'); // keep same page but ensures user knows
      return;
    }

    try {
      await downloadRemoteModel(model.repo_id, file.filename, destinationDir);
      // Rescan models after download completes (with delay to allow file copy)
      setTimeout(() => {
        scanFolder(destinationDir, true);
      }, 1000);
    } catch (error) {
      console.error('Download failed', error);
      alert(`Не удалось скачать файл: ${error instanceof Error ? error.message : String(error)}`);
    }
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
    updateRemoteFilters({ sort_order: target.value as NonNullable<typeof $remoteFilters.sort_order> });
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
        <p>Модели не найдены. Попробуйте изменить запрос или фильтры.</p>
      </div>
    {/if}

    {#each $remoteResults as model (model.repo_id)}
      <article class="model-card">
        <header>
          <div>
            <h3>{model.name}</h3>
            <a
              href={`https://huggingface.co/${model.repo_id}`}
              target="_blank"
              rel="noreferrer"
            >
              {model.repo_id}
            </a>
          </div>
          <div class="stats">
            <span title="Загрузки">⬇ {model.downloads.toLocaleString()}</span>
            <span title="Лайки">❤ {model.likes.toLocaleString()}</span>
          </div>
        </header>

        {#if model.description}
          <p class="description">{model.description}</p>
        {/if}

        <div class="meta">
          <div>
            <span class="label">Архитектуры</span>
            <span>{uniqueArchitectures(model).join(', ') || '—'}</span>
          </div>
          <div>
            <span class="label">Лицензия</span>
            <span>{model.license ?? '—'}</span>
          </div>
          <div>
            <span class="label">Обновлено</span>
            <span>{model.last_modified ? LocalModelsService.formatDate(model.last_modified) : '—'}</span>
          </div>
          <div>
            <span class="label">Контекст</span>
            <span>{model.context_length ?? '—'}</span>
          </div>
        </div>

        <div class="files">
          <h4>GGUF файлы</h4>
          <table>
            <thead>
              <tr>
                <th>Файл</th>
                <th>Размер</th>
                <th>Квант.</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each model.gguf_files as file (file.filename)}
                <tr>
                  <td>{file.filename}</td>
                  <td>{formatFileSize(file.size)}</td>
                  <td>{file.quantization ?? '—'}</td>
                  <td>
                    <button class="btn secondary" on:click={() => handleDownload(model, file)}>
                      Скачать
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </article>
    {/each}
  </section>

  {#if $activeDownloads.length}
    <section class="downloads">
      <h4>Загрузки</h4>
      <ul>
        {#each $activeDownloads as progress (progress.download_id)}
          <li>
            <div class="download-header">
              <strong>{progress.filename}</strong>
              <span>{Math.round((progress.current / progress.total) * 100)}%</span>
            </div>
            <div class="progress-bar">
              <div
                class="progress-bar-fill"
                style={`width: ${(progress.current / progress.total) * 100}%`}
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
    display: grid;
    gap: 1rem;
    overflow: auto;
    min-height: 0;
  }

  .results.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .empty-state {
    padding: 1.5rem;
    text-align: center;
    color: var(--muted);
    border: 1px dashed var(--border-color, #d8dee5);
    border-radius: 12px;
  }

  .model-card {
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 14px;
    padding: 1rem 1.2rem;
    background: var(--card);
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    box-shadow: var(--shadow);
  }

  .model-card header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .model-card h3 {
    margin: 0 0 0.2rem;
  }

  .model-card a {
    font-size: 0.85rem;
    color: var(--accent, #3498db);
    text-decoration: none;
  }

  .model-card .stats {
    display: flex;
    gap: 0.75rem;
    font-size: 0.85rem;
  }

  .description {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 0.6rem;
  }

  .meta .label {
    display: block;
    font-size: 0.75rem;
    color: var(--muted);
    margin-bottom: 0.25rem;
  }

  .files table {
    width: 100%;
    border-collapse: collapse;
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--border-color, #d8dee5);
  }

  .files thead {
    background: var(--card, #f6f8fb);
  }

  .files th,
  .files td {
    padding: 0.5rem 0.65rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color, #e5e9ef);
    font-size: 0.85rem;
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
    cursor: pointer;
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
</style>
