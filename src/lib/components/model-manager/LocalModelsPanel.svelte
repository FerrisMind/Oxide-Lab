<script lang="ts">
  import { get } from 'svelte/store';
  import { onMount } from 'svelte';
  import {
    folderPath,
    models,
    filteredModels,
    filterOptions,
    scanFolder,
    deleteModel,
    selectedModel,
    isLoading,
    error,
  } from '$lib/stores/local-models';
  import { LocalModelsService } from '$lib/services/local-models';
import type { FilterOptions, ModelInfo, ValidationLevel } from '$lib/types/local-models';
  import Checkbox from '$lib/components/ui/Checkbox.svelte';
  import Dropdown from '$lib/components/ui/Dropdown.svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import PencilSimple from 'phosphor-svelte/lib/PencilSimple';
  import Check from 'phosphor-svelte/lib/Check';
  import X from 'phosphor-svelte/lib/X';

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
  let editingModelPath: string | null = null;
  let editPublisher = '';
  let editName = '';

  function startEditing(model: ModelInfo) {
    editingModelPath = model.path;
    editPublisher =
      model.metadata?.author ??
      model.source_repo_id?.split('/')[0] ??
      'local';
    editName = model.format === 'safetensors'
      ? model.source_repo_name ?? model.name
      : model.name;
  }

  function cancelEditing() {
    editingModelPath = null;
  }

  async function saveEditing(model: ModelInfo) {
    try {
      await LocalModelsService.updateModelMetadata(
        model.path,
        editName || null,
        editPublisher || null,
      );
      models.update(($models) =>
        $models.map((entry) => {
          if (entry.path !== model.path) return entry;
          const updated = { ...entry };
          updated.metadata = {
            ...entry.metadata,
            author: editPublisher || entry.metadata.author,
          };
          if (entry.format === 'safetensors') {
            updated.source_repo_name = editName || entry.source_repo_name;
          } else {
            updated.name = editName || entry.name;
          }
          return updated;
        }),
      );
    } catch (err) {
      console.error('Failed to save metadata', err);
    } finally {
      editingModelPath = null;
    }
  }

  onMount(async () => {
    if ($folderPath) {
      await scanFolder($folderPath);
    }
  });

  async function handleSelectFolder() {
    try {
      const selected = (await open({
        directory: true,
        multiple: false,
        recursive: false,
      })) as string | string[] | undefined;
      const path = Array.isArray(selected) ? selected[0] : selected;
      if (typeof path === 'string' && path.length > 0) {
        folderPath.set(path);
        await scanFolder(path, true);
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

  async function handleDelete(model: ModelInfo) {
    const confirmed = confirm(`Удалить модель "${model.name}"?\nФайл будет перемещен в корзину.`);
    if (!confirmed) return;

    await deleteModel(model.path);
  }

  function loadSelectedModel() {
    const model = get(selectedModel);
    if (!model) return;
    const ox = (window as any).__oxide;
    if (!ox?.loadModelFromManager) return;
    ox.loadModelFromManager({
      path: model.path,
      format: model.format === 'gguf' ? 'gguf' : 'local_safetensors',
    });
  }

  function toggleModelSelection(model: ModelInfo) {
    if ($selectedModel?.path === model.path) {
      // Если модель уже выбрана, отменяем выбор
      selectedModel.set(null);
    } else {
      // Иначе выбираем новую модель
      selectedModel.set(model);
    }
  }

  $: menuItems = [
    {
      label: 'Выбрать папку',
      onclick: handleSelectFolder,
    },
    {
      label: 'Пересканировать',
      onclick: () => $folderPath && scanFolder($folderPath, true),
      disabled: !$folderPath,
    },
  ];
</script>

<div class="local-models-panel">
  <section class="controls">
    <div class="controls-left">
      <div class="path-group">
        <span class="label">Папка с моделями</span>
        <div class="path-display">
          <span title={$folderPath || 'Папка не выбрана'}>
            {$folderPath || 'Не выбрано'}
          </span>
        </div>
      </div>
      <Dropdown items={menuItems} label="⋯" />
    </div>

    <div class="controls-right">
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
    </div>

    <div class="checkbox-wrapper">
      <Checkbox
        id="candle-only"
        label="Только совместимые с Candle"
        bind:checked={$filterOptions.candleOnly}
      />
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
          <th>Архитектура</th>
          <th>Параметры</th>
          <th>Издатель</th>
          <th>Название модели</th>
          <th>Квант</th>
          <th>Размер</th>
          <th>Формат</th>
        </tr>
        </thead>
        <tbody>
          {#each $filteredModels as model (model.path)}
            <tr
              class:selected={$selectedModel?.path === model.path}
              on:click={() => toggleModelSelection(model)}
            >
              <td>{model.architecture ?? '—'}</td>
              <td>{model.parameter_count ?? '—'}</td>
               <td class="publisher-cell">
                 <button
                   type="button"
                   class="icon-btn"
                   on:click={() => startEditing(model)}
                   aria-label="Редактировать название и авторство модели"
                 >
                   <PencilSimple size={16} />
                 </button>
                 <span>
                   {#if model.format === 'safetensors'}
                     {model.metadata?.author ?? '—'}
                   {:else if model.source_repo_id}
                     {model.source_repo_id.split('/')[0]}
                   {:else}
                     {model.metadata?.author ?? '—'}
                   {/if}
                 </span>
               </td>
               <td class="title-cell">
                 <button
                   type="button"
                   class="icon-btn"
                   on:click={() => startEditing(model)}
                   aria-label="Редактировать название и авторство модели"
                 >
                   <PencilSimple size={16} />
                 </button>
                 <div class="model-title">
                   <strong title={model.name}>
                     {#if model.format === 'safetensors'}
                       {model.source_repo_name ?? '—'}
                     {:else}
                       {model.name}
                     {/if}
                   </strong>
                 </div>
               </td>
              <td>
                {#if model.format === 'safetensors'}
                  {model.source_quantization ?? '—'}
                {:else}
                  {model.quantization ?? '—'}
                {/if}
              </td>
              <td>{LocalModelsService.formatFileSize(model.file_size)}</td>
              <td class="format-cell">{model.format.toUpperCase()}</td>
            </tr>
            {#if editingModelPath === model.path}
              <tr class="edit-row">
                <td colspan="7">
                  <div class="edit-grid">
                    <label>
                      Паблишер
                      <input
                        type="text"
                        placeholder="publisher"
                        bind:value={editPublisher}
                      />
                    </label>
                    <label>
                      Название
                      <input
                        type="text"
                        placeholder="publisher/name"
                        bind:value={editName}
                      />
                    </label>
                    <div class="edit-actions">
                      <button type="button" class="btn" on:click={() => saveEditing(model)}>
                        <Check size={16} /> Сохранить
                      </button>
                      <button type="button" class="btn secondary" on:click={cancelEditing}>
                        <X size={16} /> Отменить
                      </button>
                    </div>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>

    {#if $selectedModel}
      <aside class="details">
        <header>
          <h3>{$selectedModel.name}</h3>
          <div class="actions">
            <button class="btn danger" on:click={() => handleDelete($selectedModel!)}>
              Удалить
            </button>
            <button class="btn primary" on:click={loadSelectedModel}>
              <DownloadSimple size={16} />
              Загрузить в чат
            </button>
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
            <dt>Формат</dt>
            <dd class="format-cell">{$selectedModel.format.toUpperCase()}</dd>
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
      </aside>
    {/if}
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

  .controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    background: var(--card);
    border-radius: 12px;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color, #d8dee5);
    box-shadow: var(--shadow);
    flex-wrap: wrap;
  }

  .controls-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
    min-width: 0;
  }

  .controls-right {
    display: flex;
    align-items: center;
    justify-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .path-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    flex: 0 1 250px;
  }

  .path-group .label {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .path-display {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
    font-size: 0.8rem;
    min-width: 0;
    flex: 1;
    border: 1px solid var(--border-color, #d8dee5);
    border-radius: 6px;
    padding: 0.35rem 0.5rem;
    background: #1a1a1a;
  }

  .path-display span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }


  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.75rem;
  }

  .metric {
    background: #1a1a1a;
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


  .filter-group {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.75rem;
  }

  .filter-group input {
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    border: 1px solid var(--border-color, #d8dee5);
    background: #1a1a1a;
    font-size: 0.8rem;
    min-width: 200px;
  }

  .checkbox-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
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
    grid-template-columns: 1fr;
    gap: 1rem;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .content:has(.details) {
    grid-template-columns: 1fr minmax(320px, 360px);
  }

  .content.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .list {
    overflow: auto;
    border-radius: 12px;
    border: 1px solid var(--border-color, #d8dee5);
    background: #1a1a1a;
    box-shadow: var(--shadow);
    min-height: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: #1a1a1a;
  }

  th,
  td {
    padding: 0.65rem 0.9rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color, #e1e5ea);
    font-size: 0.9rem;
  }

  td.format-cell {
    font-weight: 600;
    text-transform: uppercase;
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
    background: #1a1a1a;
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

  .details header .actions {
    display: inline-flex;
    gap: 0.5rem;
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
    padding: 0.35rem 0.7rem;
    border-radius: 6px;
    border: none;
    background: var(--accent, #3498db);
    color: #fff;
    cursor: default;
    font-size: 0.75rem;
    transition: opacity 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    white-space: nowrap;
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

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin-right: 0.35rem;
    cursor: pointer;
    opacity: 0.7;
  }

  .icon-btn:hover {
    opacity: 1;
  }

  .publisher-cell,
  .title-cell {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .edit-row {
    background: rgba(255, 255, 255, 0.03);
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .edit-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 0.75rem;
    align-items: end;
  }

  .edit-grid label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.8rem;
    color: var(--muted);
  }

  .edit-grid input {
    border: 1px solid var(--border-color, #4a4a4a);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    background: var(--bg-secondary, #1f1f1f);
    color: var(--text);
  }

  .edit-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    justify-content: flex-end;
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
