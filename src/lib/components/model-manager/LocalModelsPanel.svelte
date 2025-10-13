<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    folderPath,
    models,
    filteredModels,
    filterOptions,
    sortOptions,
    uniqueArchitectures,
    uniqueQuantizations,
    validationCounters,
    scanFolder,
    deleteModel,
    selectedModel,
    isLoading,
    error,
  } from '$lib/stores/local-models';
  import { LocalModelsService } from '$lib/services/local-models';
  import type { FilterOptions, ModelInfo, ValidationLevel } from '$lib/types/local-models';
  import Checkbox from '$lib/components/ui/Checkbox.svelte';

  const validationLabels: Record<ValidationLevel, string> = {
    ok: 'Валидно',
    warning: 'Предупреждение',
    error: 'Ошибка',
  };

  const validationColors: Record<ValidationLevel, string> = {
    ok: 'badge-success',
    warning: 'badge-warning',
    error: 'badge-error',
  };

  let metadataExpanded = false;

  onMount(async () => {
    if ($folderPath) {
      await scanFolder($folderPath);
    }
  });

  async function handleSelectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        recursive: false,
      });

      if (typeof selected === 'string' && selected.length > 0) {
        folderPath.set(selected);
        await scanFolder(selected, true);
      }
    } catch (err) {
      console.error('Failed to select folder', err);
    }
  }

  function updateFilter(partial: Partial<FilterOptions>) {
    filterOptions.update((prev) => ({
      ...prev,
      ...partial,
    }));
  }

  function updateValidationFilter(level: ValidationLevel | 'all') {
    updateFilter({ validation: level });
  }

  async function handleDelete(model: ModelInfo) {
    const confirmed = confirm(`Удалить модель "${model.name}"?\nФайл будет перемещен в корзину.`);
    if (!confirmed) return;

    await deleteModel(model.path);
  }
</script>

<div class="local-models-panel">
  <section class="toolbar">
    <div class="path-group">
      <span class="label">Папка с моделями</span>
      <div class="path-display">
        <span title={$folderPath || 'Папка не выбрана'}>
          {$folderPath || 'Не выбрано'}
        </span>
        <div class="toolbar-actions">
          <button class="btn" on:click={handleSelectFolder}>Выбрать папку</button>
          <button
            class="btn secondary"
            disabled={!$folderPath}
            on:click={() => $folderPath && scanFolder($folderPath, true)}
          >
            Пересканировать
          </button>
        </div>
      </div>
    </div>
    <div class="metrics">
      <div class="metric">
        <span class="metric-title">Всего</span>
        <span class="metric-value">{$validationCounters.total}</span>
      </div>
      <div class="metric ok">
        <span class="metric-title">Ок</span>
        <span class="metric-value">{$validationCounters.ok}</span>
      </div>
      <div class="metric warn">
        <span class="metric-title">Предупр.</span>
        <span class="metric-value">{$validationCounters.warning}</span>
      </div>
      <div class="metric error">
        <span class="metric-title">Ошибки</span>
        <span class="metric-value">{$validationCounters.error}</span>
      </div>
    </div>
  </section>

  <section class="filters">
    <div class="filter-group">
      <label>
        Поиск
        <input
          type="search"
          placeholder="Название, архитектура, квантизация..."
          value={$filterOptions.searchText ?? ''}
          on:input={(event) => updateFilter({ searchText: event.currentTarget.value })}
        />
      </label>
    </div>

    <div class="filter-group">
      <label>
        Архитектура
        <select
          on:change={(event) =>
            updateFilter({ architecture: event.currentTarget.value || undefined })}
        >
          <option value="">Все</option>
          {#each $uniqueArchitectures as arch}
            <option value={arch} selected={arch === $filterOptions.architecture}>{arch}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="filter-group">
      <label>
        Квантизация
        <select
          on:change={(event) =>
            updateFilter({ quantization: event.currentTarget.value || undefined })}
        >
          <option value="">Все</option>
          {#each $uniqueQuantizations as quant}
            <option value={quant} selected={quant === $filterOptions.quantization}>
              {quant}
            </option>
          {/each}
        </select>
      </label>
    </div>

    <div class="filter-group checkbox">
      <Checkbox
        id="candle-only"
        label="Только совместимые с Candle"
        bind:checked={$filterOptions.candleOnly}
      />
    </div>

    <div class="filter-group">
      <label>
        Валидация
        <select
          on:change={(event) =>
            updateValidationFilter(event.currentTarget.value as ValidationLevel | 'all')}
        >
          <option value="all" selected={$filterOptions.validation === 'all'}>Все</option>
          <option value="ok" selected={$filterOptions.validation === 'ok'}>Ок</option>
          <option value="warning" selected={$filterOptions.validation === 'warning'}>
            Предупреждение
          </option>
          <option value="error" selected={$filterOptions.validation === 'error'}>Ошибка</option>
        </select>
      </label>
    </div>

    <div class="filter-group">
      <label>
        Сортировка
        <select
          on:change={(event) =>
            sortOptions.update((prev) => ({
              ...prev,
              field: event.currentTarget.value as typeof prev.field,
            }))}
        >
          <option value="name" selected={$sortOptions.field === 'name'}>Имя</option>
          <option value="file_size" selected={$sortOptions.field === 'file_size'}>Размер</option>
          <option value="created_at" selected={$sortOptions.field === 'created_at'}> Дата </option>
          <option value="parameter_count" selected={$sortOptions.field === 'parameter_count'}>
            Параметры
          </option>
          <option value="architecture" selected={$sortOptions.field === 'architecture'}>
            Архитектура
          </option>
        </select>
      </label>
    </div>

    <div class="filter-group">
      <label>
        Порядок
        <select
          on:change={(event) =>
            sortOptions.update((prev) => ({
              ...prev,
              order: event.currentTarget.value as typeof prev.order,
            }))}
        >
          <option value="asc" selected={$sortOptions.order === 'asc'}>Возр.</option>
          <option value="desc" selected={$sortOptions.order === 'desc'}>Убыв.</option>
        </select>
      </label>
    </div>
  </section>

  {#if $error}
    <div class="error-banner">
      <span>{$error}</span>
      <button class="btn secondary" on:click={() => $folderPath && scanFolder($folderPath, true)}>
        Повторить
      </button>
    </div>
  {/if}

  <div class:loading={$isLoading} class="content">
    <div class="list">
      {#if !$filteredModels.length && !$isLoading}
        <div class="empty-state">
          <p>Нет моделей, подходящих под выбранные условия.</p>
          {#if !$models.length}
            <p>Выберите папку с моделями, чтобы начать.</p>
          {/if}
        </div>
      {/if}

      <table>
        <thead>
          <tr>
            <th>Модель</th>
            <th>Архитектура</th>
            <th>Квант.</th>
            <th>Параметры</th>
            <th>Контекст</th>
            <th>Размер</th>
            <th>Валидация</th>
            <th>Candle</th>
          </tr>
        </thead>
        <tbody>
          {#each $filteredModels as model (model.path)}
            <tr
              class:selected={$selectedModel?.path === model.path}
              on:click={() => selectedModel.set(model)}
            >
              <td>
                <div class="model-title">
                  <strong>{model.name}</strong>
                  {#if model.model_name && model.model_name !== model.name}
                    <span class="muted">{model.model_name}</span>
                  {/if}
                </div>
              </td>
              <td>{model.architecture ?? '—'}</td>
              <td>{model.quantization ?? '—'}</td>
              <td>{model.parameter_count ?? '—'}</td>
              <td>{model.context_length ?? '—'}</td>
              <td>{LocalModelsService.formatFileSize(model.file_size)}</td>
              <td>
                <span class={`badge ${validationColors[model.validation_status.level]}`}>
                  {validationLabels[model.validation_status.level]}
                </span>
              </td>
              <td>
                {#if model.candle_compatible}
                  <span class="badge badge-success">Да</span>
                {:else}
                  <span class="badge badge-muted">Нет</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <aside class="details">
      {#if $selectedModel}
        <header>
          <h3>{$selectedModel.name}</h3>
          <div class="actions">
            <button class="btn danger" on:click={() => handleDelete($selectedModel!)}
              >Удалить</button
            >
          </div>
        </header>

        <dl class="properties">
          <div>
            <dt>Путь</dt>
            <dd class="path">{$selectedModel.path}</dd>
          </div>
          <div>
            <dt>Размер</dt>
            <dd>{LocalModelsService.formatFileSize($selectedModel.file_size)}</dd>
          </div>
          <div>
            <dt>Дата</dt>
            <dd>{LocalModelsService.formatDate($selectedModel.created_at)}</dd>
          </div>
          <div>
            <dt>Архитектура</dt>
            <dd>{$selectedModel.architecture ?? '—'}</dd>
          </div>
          <div>
            <dt>Детектировано</dt>
            <dd>{$selectedModel.detected_architecture ?? '—'}</dd>
          </div>
          <div>
            <dt>Контекст</dt>
            <dd>{$selectedModel.context_length ?? '—'}</dd>
          </div>
        </dl>

        <section class="validation">
          <h4>Статус проверки</h4>
          <span class={`badge ${validationColors[$selectedModel.validation_status.level]}`}>
            {validationLabels[$selectedModel.validation_status.level]}
          </span>
          {#if $selectedModel.validation_status.messages.length}
            <ul>
              {#each $selectedModel.validation_status.messages as message}
                <li>{message}</li>
              {/each}
            </ul>
          {/if}
        </section>

        <section class="metadata">
          <header>
            <h4>GGUF метаданные</h4>
            <button class="btn secondary" on:click={() => (metadataExpanded = !metadataExpanded)}>
              {metadataExpanded ? 'Скрыть' : 'Показать все'}
            </button>
          </header>

          <dl class="meta-grid">
            <div>
              <dt>Версия формата</dt>
              <dd>{$selectedModel.metadata.format_version}</dd>
            </div>
            <div>
              <dt>Tensor count</dt>
              <dd>{$selectedModel.metadata.tensor_count}</dd>
            </div>
            <div>
              <dt>Alignment</dt>
              <dd>{$selectedModel.metadata.alignment}</dd>
            </div>
            <div>
              <dt>Token count</dt>
              <dd>
                {$selectedModel.vocab_size ??
                  $selectedModel.metadata.tokenizer_tokens?.length ??
                  '—'}
              </dd>
            </div>
          </dl>

          {#if metadataExpanded}
            <div class="meta-table">
              <table>
                <thead>
                  <tr>
                    <th>Ключ</th>
                    <th>Значение</th>
                  </tr>
                </thead>
                <tbody>
                  {#each $selectedModel.metadata.custom_metadata as entry (entry.key)}
                    <tr>
                      <td>{entry.key}</td>
                      <td><pre>{JSON.stringify(entry.value, null, 2)}</pre></td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </section>
      {:else}
        <div class="placeholder">
          <p>Выберите модель, чтобы посмотреть подробности.</p>
        </div>
      {/if}
    </aside>
  </div>
</div>

<style>
  .local-models-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
  }

  .local-models-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
    min-height: 0;
    color: var(--text);
  }

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--card);
    border-radius: 12px;
    padding: 1rem 1.25rem;
    border: 1px solid var(--border-color, #d8dee5);
    box-shadow: var(--shadow);
  }

  .path-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .path-group .label {
    font-size: 0.85rem;
    color: var(--muted);
  }

  .path-display {
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-content: space-between;
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
    font-size: 0.9rem;
  }

  .toolbar-actions {
    display: flex;
    gap: 0.5rem;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.75rem;
  }

  .metric {
    background: var(--card);
    border-radius: 10px;
    border: 1px solid var(--border-color, #d8dee5);
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    box-shadow: var(--shadow);
  }

  .metric.ok {
    border-color: rgba(46, 204, 113, 0.4);
  }
  .metric.warn {
    border-color: rgba(241, 196, 15, 0.4);
  }
  .metric.error {
    border-color: rgba(231, 76, 60, 0.4);
  }

  .metric-title {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .metric-value {
    font-weight: 600;
    font-size: 1.1rem;
  }

  .filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 0.75rem;
    align-items: end;
    background: var(--card);
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 12px;
    padding: 1rem 1.25rem;
    box-shadow: var(--shadow);
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
  }

  .filter-group input,
  .filter-group select {
    width: 100%;
    padding: 0.45rem 0.6rem;
    border-radius: 8px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--card);
  }

  .filter-group.checkbox {
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-radius: 10px;
    border: 1px solid color-mix(in srgb, var(--danger) 35%, transparent 65%);
    background: color-mix(in srgb, var(--danger) 12%, transparent 88%);
    color: color-mix(in srgb, var(--danger) 85%, black 15%);
  }

  .content {
    display: grid;
    grid-template-columns: 1fr minmax(320px, 360px);
    gap: 1rem;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .content.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .list {
    overflow: auto;
    border-radius: 12px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--card);
    box-shadow: var(--shadow);
    min-height: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: var(--card, #f2f4f8);
  }

  th,
  td {
    padding: 0.65rem 0.9rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color, #e1e5ea);
    font-size: 0.9rem;
  }

  tbody tr {
    cursor: default;
    transition: background 0.15s ease;
  }

  tbody tr:hover {
    background: color-mix(in srgb, var(--accent) 12%, transparent 88%);
  }

  tbody tr.selected {
    background: color-mix(in srgb, var(--accent) 18%, transparent 82%);
  }

  .model-title {
    display: flex;
    flex-direction: column;
  }

  .model-title .muted {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.5rem;
    border-radius: 99px;
    font-size: 0.72rem;
    font-weight: 600;
  }

  .badge-success {
    background: rgba(46, 204, 113, 0.14);
    color: #1e8449;
  }

  .badge-warning {
    background: rgba(241, 196, 15, 0.16);
    color: #b9770e;
  }

  .badge-error {
    background: rgba(231, 76, 60, 0.16);
    color: #c0392b;
  }

  .badge-muted {
    background: rgba(127, 140, 141, 0.16);
    color: #646d6f;
  }

  .details {
    border-radius: 12px;
    border: 1px solid var(--border-color, #d8dee5);
    background: var(--card);
    padding: 1rem 1.2rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: var(--shadow);
    min-height: 0;
  }

  .details header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .details h3 {
    margin: 0;
    font-size: 1.1rem;
  }

  .properties {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 0.75rem;
  }

  .properties div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .properties dt {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .properties dd {
    margin: 0;
    font-weight: 500;
  }

  .properties dd.path {
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
    font-size: 0.8rem;
    word-break: break-all;
  }

  .validation ul {
    margin: 0.5rem 0 0;
    padding-left: 1rem;
    font-size: 0.85rem;
  }

  .metadata header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 0.6rem;
    margin-top: 0.75rem;
  }

  .meta-grid div {
    display: flex;
    flex-direction: column;
    padding: 0.6rem;
    border: 1px dashed var(--border-color, #d8dee5);
    border-radius: 8px;
    background: rgba(248, 249, 251, 0.6);
  }

  .meta-grid dt {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .meta-grid dd {
    margin: 0;
    font-weight: 600;
  }

  .meta-table {
    margin-top: 0.75rem;
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 10px;
    overflow: hidden;
  }

  .meta-table table {
    width: 100%;
  }

  .meta-table pre {
    margin: 0;
    font-size: 0.75rem;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
  }

  .placeholder,
  .empty-state {
    padding: 1.5rem;
    text-align: center;
    color: var(--muted);
  }

  .btn {
    padding: 0.45rem 0.9rem;
    border-radius: 8px;
    border: none;
    background: var(--accent, #3498db);
    color: #fff;
    cursor: default;
    font-size: 0.85rem;
    transition: opacity 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .btn.secondary {
    background: color-mix(in srgb, var(--accent) 12%, transparent 88%);
    color: var(--accent, #3498db);
  }

  .btn.danger {
    background: color-mix(in srgb, var(--danger) 85%, black 15%);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 1024px) {
    .content {
      grid-template-columns: 1fr;
      grid-template-rows: auto auto;
    }

    .details {
      order: -1;
    }
  }
</style>
