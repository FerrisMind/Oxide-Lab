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
import Check from 'phosphor-svelte/lib/CheckCircle';
import X from 'phosphor-svelte/lib/X';
  import { t } from '$lib/i18n';

  const validationColors: Record<ValidationLevel, string> = {
    ok: 'badge-success',
    warning: 'badge-warning',
    error: 'badge-error',
  };

  let metadataExpanded = $state(false);
  let editingModelPath = $state<string | null>(null);
  let editPublisher = $state('');
  let editName = $state('');
  let candleOnlyFilter = $state(false);

  function startEditing(model: ModelInfo) {
    editingModelPath = model.path;
    editPublisher = model.metadata?.author ?? model.source_repo_id?.split('/')[0] ?? 'local';
    editName = model.format === 'safetensors' ? (model.source_repo_name ?? model.name) : model.name;
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
    // Инициализируем значение из store
    candleOnlyFilter = $filterOptions.candleOnly ?? false;

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

  // Обработка изменения чекбокса пользователем
  function handleCandleOnlyChange(newValue: boolean) {
    // Обновляем store
    // Локальное состояние уже обновлено через bind:checked
    updateFilter({ candleOnly: newValue });
  }

  async function handleDelete(model: ModelInfo) {
    // Используем простую интерполяцию для confirm сообщения
    const confirmMessage = $t('models.local.details.deleteConfirm').replace('{name}', model.name);
    const confirmed = confirm(confirmMessage);
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

  const menuItems = $derived([
    {
      label: $t('models.local.menu.selectFolder'),
      onclick: handleSelectFolder,
    },
    {
      label: $t('models.local.menu.rescan'),
      onclick: () => $folderPath && scanFolder($folderPath, true),
      disabled: !$folderPath,
    },
  ]);
</script>

<div class="local-models-panel">
  <section class="controls">
    <div class="controls-left">
      <div class="path-group">
        <span class="label">{$t('models.local.folderLabel')}</span>
        <div class="path-display">
          <span title={$folderPath || $t('models.local.folderNotSelected')}>
            {$folderPath || $t('models.local.notSelected')}
          </span>
        </div>
      </div>
      <Dropdown items={menuItems} label="⋯" />
    </div>

    <div class="controls-right">
      <div class="filter-group">
        <label>
          {$t('models.local.search')}
          <input
            type="search"
            placeholder={$t('models.local.searchPlaceholder')}
            value={$filterOptions.searchText ?? ''}
            oninput={(event) => updateFilter({ searchText: event.currentTarget.value })}
          />
        </label>
      </div>
    </div>

    <div class="checkbox-wrapper">
      <Checkbox
        id="candle-only"
        label={$t('models.local.candleOnly')}
        bind:checked={candleOnlyFilter}
        onchange={handleCandleOnlyChange}
      />
    </div>
  </section>

  {#if $error}
    <div class="error-banner">
      <span>{$error}</span>
      <button class="btn secondary" onclick={() => $folderPath && scanFolder($folderPath, true)}>
        {$t('models.local.errors.retry')}
      </button>
    </div>
  {/if}

  <div class:loading={$isLoading} class="content">
    <div class="list">
      {#if !$filteredModels.length && !$isLoading}
        <div class="empty-state">
          <p>{$t('models.local.noModels')}</p>
          {#if !$models.length}
            <p>{$t('models.local.selectFolder')}</p>
          {/if}
        </div>
      {/if}

      <table>
        <thead>
          <tr>
            <th>{$t('models.local.table.architecture')}</th>
            <th>{$t('models.local.table.parameters')}</th>
            <th>{$t('models.local.table.publisher')}</th>
            <th>{$t('models.local.table.modelName')}</th>
            <th>{$t('models.local.table.quant')}</th>
            <th>{$t('models.local.table.size')}</th>
            <th>{$t('models.local.table.format')}</th>
          </tr>
        </thead>
        <tbody>
          {#each $filteredModels as model (model.path)}
            <tr
              class:selected={$selectedModel?.path === model.path}
              onclick={() => toggleModelSelection(model)}
            >
              <td>{model.architecture ?? '—'}</td>
              <td>{model.parameter_count ?? '—'}</td>
              <td class="publisher-cell">
                <button
                  type="button"
                  class="icon-btn"
                  onclick={() => startEditing(model)}
                  aria-label={$t('models.local.details.edit.ariaLabel')}
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
                  onclick={() => startEditing(model)}
                  aria-label={$t('models.local.details.edit.ariaLabel')}
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
                      {$t('models.local.details.edit.publisher')}
                      <input
                        type="text"
                        placeholder={$t('models.local.details.edit.publisherPlaceholder')}
                        bind:value={editPublisher}
                      />
                    </label>
                    <label>
                      {$t('models.local.details.edit.name')}
                      <input
                        type="text"
                        placeholder={$t('models.local.details.edit.namePlaceholder')}
                        bind:value={editName}
                      />
                    </label>
                    <div class="edit-actions">
                      <button type="button" class="btn" onclick={() => saveEditing(model)}>
                        <Check size={16} />
                        {$t('models.local.details.edit.save')}
                      </button>
                      <button type="button" class="btn secondary" onclick={cancelEditing}>
                        <X size={16} />
                        {$t('models.local.details.edit.cancel')}
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
            <button class="btn danger" onclick={() => handleDelete($selectedModel!)}>
              {$t('models.local.details.delete')}
            </button>
            <button class="btn primary" onclick={loadSelectedModel}>
              <DownloadSimple size={16} />
              {$t('models.local.details.loadToChat')}
            </button>
          </div>
        </header>

        <dl class="properties">
          <div>
            <dt>{$t('models.local.details.path')}</dt>
            <dd class="path">{$selectedModel.path}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.size')}</dt>
            <dd>{LocalModelsService.formatFileSize($selectedModel.file_size)}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.date')}</dt>
            <dd>{LocalModelsService.formatDate($selectedModel.created_at)}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.architecture')}</dt>
            <dd>{$selectedModel.architecture ?? '—'}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.format')}</dt>
            <dd class="format-cell">{$selectedModel.format.toUpperCase()}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.detected')}</dt>
            <dd>{$selectedModel.detected_architecture ?? '—'}</dd>
          </div>
          <div>
            <dt>{$t('models.local.details.context')}</dt>
            <dd>{$selectedModel.context_length ?? '—'}</dd>
          </div>
        </dl>

        <section class="validation">
          <h4>{$t('models.local.details.validation')}</h4>
          <span class={`badge ${validationColors[$selectedModel.validation_status.level]}`}>
            {$t(
              `models.local.details.${$selectedModel.validation_status.level === 'ok' ? 'valid' : $selectedModel.validation_status.level}`,
            )}
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
            <button class="btn secondary" onclick={() => (metadataExpanded = !metadataExpanded)}>
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
    gap: var(--space-3);
    height: 100%;
    min-height: 0;
    color: var(--text);
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    background: var(--card);
    border-radius: var(--radius-lg);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow);
    flex-wrap: wrap;
  }

  .controls-left {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-width: 0;
  }

  .controls-right {
    display: flex;
    align-items: center;
    justify-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .path-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    flex: 0 1 250px;
  }

  .path-group .label {
    font-size: var(--font-size-xs);
    color: var(--muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .path-display {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
    font-size: var(--font-size-xs);
    min-width: 0;
    flex: 1;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-1) var(--space-2);
    background: var(--panel-bg);
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
    gap: var(--space-3);
  }

  .metric {
    background: var(--panel-bg);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    box-shadow: var(--shadow);
  }

  .metric.ok {
    border-color: color-mix(in srgb, var(--accent-2) 55%, transparent);
  }
  .metric.warn {
    border-color: color-mix(in srgb, var(--warning) 55%, transparent);
  }
  .metric.error {
    border-color: color-mix(in srgb, var(--danger) 55%, transparent);
  }

  .metric-title {
    font-size: var(--font-size-xs);
    color: var(--muted);
  }

  .metric-value {
    font-weight: var(--font-weight-semibold);
    font-size: var(--font-size-lg);
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--font-size-xs);
  }

  .filter-group input {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    background: var(--panel-bg);
    font-size: var(--font-size-xs);
    min-width: 200px; /* fixed input width */
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
    padding: var(--space-3) var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--danger) 35%, transparent 65%);
    background: color-mix(in srgb, var(--danger) 12%, transparent 88%);
    color: color-mix(in srgb, var(--danger) 85%, black 15%);
  }

  .content {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-3);
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
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    background: var(--card);
    box-shadow: var(--shadow);
    min-height: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: var(--panel-bg);
  }

  th,
  td {
    padding: var(--space-2) var(--space-3);
    text-align: left;
    border-bottom: 1px solid var(--border-color);
    font-size: var(--font-size-sm);
  }

  td.format-cell {
    font-weight: var(--font-weight-semibold);
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
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-full);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
  }

  .badge-success {
    background: color-mix(in srgb, var(--accent-2) 18%, transparent 82%);
    color: color-mix(in srgb, var(--accent-2) 70%, var(--text) 30%);
  }

  .badge-warning {
    background: color-mix(in srgb, var(--warning) 18%, transparent 82%);
    color: color-mix(in srgb, var(--warning) 70%, var(--text) 30%);
  }

  .badge-error {
    background: color-mix(in srgb, var(--danger) 18%, transparent 82%);
    color: color-mix(in srgb, var(--danger) 70%, var(--text) 30%);
  }

  .badge-muted {
    background: color-mix(in srgb, var(--muted) 16%, transparent 84%);
    color: var(--muted);
  }

  .details {
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    background: var(--card);
    padding: var(--space-3) var(--space-4);
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
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
    gap: var(--space-2);
  }

  .details h3 {
    margin: 0;
    font-size: var(--font-size-lg);
  }

  .properties {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-3);
  }

  .properties div {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .properties dt {
    font-size: var(--font-size-xs);
    color: var(--muted);
  }

  .properties dd {
    margin: 0;
    font-weight: var(--font-weight-medium);
  }

  .properties dd.path {
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
    font-size: var(--font-size-xs);
    word-break: break-all;
  }

  .validation ul {
    margin: var(--space-2) 0 0;
    padding-left: var(--space-3);
    font-size: var(--font-size-sm);
  }

  .metadata header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .meta-grid div {
    display: flex;
    flex-direction: column;
    padding: var(--space-2);
    border: 1px dashed var(--border-color);
    border-radius: var(--radius-lg);
    background: var(--panel-bg);
  }

  .meta-grid dt {
    font-size: var(--font-size-xs);
    color: var(--muted);
  }

  .meta-grid dd {
    margin: 0;
    font-weight: var(--font-weight-semibold);
  }

  .meta-table {
    margin-top: var(--space-3);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .meta-table table {
    width: 100%;
  }

  .meta-table pre {
    margin: 0;
    font-size: var(--font-size-xs);
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--mono-font, 'JetBrains Mono', monospace);
  }

  .placeholder,
  .empty-state {
    padding: var(--space-4);
    text-align: center;
    color: var(--muted);
  }

  .btn {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-lg);
    border: none;
    background: var(--accent);
    color: #ffffff;
    cursor: default;
    font-size: var(--font-size-xs);
    transition: opacity 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    white-space: nowrap;
  }

  .btn.secondary {
    background: color-mix(in srgb, var(--accent) 12%, transparent 88%);
    color: var(--accent);
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
    margin-right: var(--space-1);
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
    gap: var(--space-1);
  }

  .edit-row {
    background: color-mix(in srgb, var(--panel-bg) 70%, transparent 30%);
    border-top: 1px solid color-mix(in srgb, var(--border-color) 60%, transparent);
  }

  .edit-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: var(--space-3);
    align-items: end;
  }

  .edit-grid label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: var(--font-size-xs);
    color: var(--muted);
  }

  .edit-grid input {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-1) var(--space-2);
    background: var(--panel-bg);
    color: var(--text);
  }

  .edit-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
