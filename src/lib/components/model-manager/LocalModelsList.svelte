<script lang="ts">
  /**
   * Component for displaying list of local models
   */

  import type { LocalModelInfo, SortField, SortOrder } from '$lib/types/local-models';
  import { LocalModelsService } from '$lib/services/local-models';
  import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
  import { mount, unmount } from 'svelte';
  import TrashIcon from 'phosphor-svelte/lib/Trash';
  import CaretUpIcon from 'phosphor-svelte/lib/CaretUp';
  import CaretDownIcon from 'phosphor-svelte/lib/CaretDown';

  let {
    models = [],
    selectedModel = $bindable(null),
    onDelete = async () => {},
  }: {
    models: LocalModelInfo[];
    selectedModel: LocalModelInfo | null;
    onDelete?: (model: LocalModelInfo) => Promise<void>;
  } = $props();

  // State
  let sortField = $state<SortField>('name');
  let sortOrder = $state<SortOrder>('asc');
  let deleteConfirmOpen = $state(false);
  let modelToDelete = $state<LocalModelInfo | null>(null);

  // Handle sort
  function handleSort(field: SortField) {
    if (sortField === field) {
      sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      sortField = field;
      sortOrder = 'asc';
    }
  }

  // Sorted models
  let sortedModels = $derived(LocalModelsService.sortModels(models, sortField, sortOrder));

  // Handle model selection
  function handleSelect(model: LocalModelInfo) {
    selectedModel = selectedModel?.path === model.path ? null : model;
  }

  // Handle delete confirmation
  function confirmDelete(model: LocalModelInfo) {
    modelToDelete = model;
    deleteConfirmOpen = true;
  }

  async function handleDeleteConfirm() {
    if (modelToDelete) {
      await onDelete(modelToDelete);
      modelToDelete = null;
    }
  }

  // Icon mounting helper
  function mountIcon(element: HTMLElement, Component: any) {
    const icon = mount(Component, { target: element, props: { size: 16, weight: 'bold' } });
    return {
      destroy() {
        unmount(icon);
      },
    };
  }
</script>

<ConfirmDialog
  bind:open={deleteConfirmOpen}
  title="Удалить модель"
  message={`Вы уверены, что хотите удалить модель "${modelToDelete?.name}"? Это действие нельзя отменить.`}
  confirmText="Удалить"
  cancelText="Отмена"
  danger={true}
  onConfirm={handleDeleteConfirm}
/>

<div class="local-models-list">
  {#if models.length === 0}
    <div class="empty-state">
      <p>Модели не найдены</p>
      <span class="empty-hint">Выберите папку с моделями для сканирования</span>
    </div>
  {:else}
    <table class="models-table">
      <thead>
        <tr>
          <th>
            <button class="sort-btn" onclick={() => handleSort('name')}>
              Название
              {#if sortField === 'name'}
                <span
                  class="sort-icon"
                  use:mountIcon={sortOrder === 'asc' ? CaretUpIcon : CaretDownIcon}
                ></span>
              {/if}
            </button>
          </th>
          <th>Архитектура</th>
          <th>Параметры</th>
          <th>Квантизация</th>
          <th>
            <button class="sort-btn" onclick={() => handleSort('size_bytes')}>
              Размер
              {#if sortField === 'size_bytes'}
                <span
                  class="sort-icon"
                  use:mountIcon={sortOrder === 'asc' ? CaretUpIcon : CaretDownIcon}
                ></span>
              {/if}
            </button>
          </th>
          <th>Формат</th>
          <th>Действия</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedModels as model (model.path)}
          <tr
            class="model-row"
            class:selected={selectedModel?.path === model.path}
            onclick={() => handleSelect(model)}
          >
            <td class="model-name" title={model.path}>
              <span class="name">{model.name}</span>
              {#if model.author}
                <span class="author">by {model.author}</span>
              {/if}
            </td>
            <td>{model.architecture || '—'}</td>
            <td>{model.parameters || '—'}</td>
            <td>
              {#if model.quantization}
                <span class="quantization-badge">{model.quantization}</span>
              {:else}
                —
              {/if}
            </td>
            <td>{LocalModelsService.formatFileSize(model.size_bytes)}</td>
            <td>
              <span class="format-badge {model.format}">{model.format.toUpperCase()}</span>
            </td>
            <td>
              <button
                class="delete-btn"
                aria-label="Удалить модель {model.name}"
                title="Удалить модель"
                onclick={(e) => {
                  e.stopPropagation();
                  confirmDelete(model);
                }}
              >
                <span use:mountIcon={TrashIcon}></span>
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .local-models-list {
    height: 100%;
    overflow-y: auto;
    background: var(--card);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 3rem 1.5rem;
    text-align: center;
  }

  .empty-state p {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 500;
    color: var(--text);
  }

  .empty-hint {
    font-size: 0.9375rem;
    color: var(--muted);
  }

  .models-table {
    width: 100%;
    border-collapse: collapse;
  }

  .models-table thead {
    position: sticky;
    top: 0;
    background: var(--bg);
    z-index: 10;
    box-shadow: 0 1px 0 var(--border-color);
  }

  .models-table th {
    padding: 0.75rem 1rem;
    text-align: left;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sort-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    transition: color 0.2s ease;
  }

  .sort-btn:hover {
    color: var(--accent, #3498db);
  }

  .sort-icon {
    display: inline-flex;
    color: var(--accent, #3498db);
  }

  .models-table tbody tr {
    border-bottom: 1px solid var(--border-color);
    transition: background-color 0.2s ease;
    cursor: pointer;
  }

  .models-table tbody tr:hover {
    background: rgba(52, 152, 219, 0.05);
  }

  .models-table tbody tr.selected {
    background: rgba(52, 152, 219, 0.1);
  }

  .models-table td {
    padding: 0.875rem 1rem;
    font-size: 0.9375rem;
  }

  .model-name {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .model-name .name {
    font-weight: 500;
    color: var(--text);
  }

  .model-name .author {
    font-size: 0.8125rem;
    color: var(--muted);
  }

  .quantization-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    background: rgba(52, 152, 219, 0.1);
    color: var(--accent, #3498db);
    font-size: 0.8125rem;
    font-weight: 600;
    border-radius: 4px;
    font-family: monospace;
  }

  .format-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .format-badge.gguf {
    background: rgba(155, 89, 182, 0.1);
    color: #9b59b6;
  }

  .format-badge.safetensors {
    background: rgba(46, 204, 113, 0.1);
    color: #2ecc71;
  }

  .delete-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.375rem;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .delete-btn:hover {
    background: rgba(231, 76, 60, 0.1);
    color: var(--error, #e74c3c);
  }

  /* Скроллбар */
  .local-models-list::-webkit-scrollbar {
    width: 12px;
  }

  .local-models-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .local-models-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 6px;
    border: 2px solid transparent;
    background-clip: content-box;
  }

  .local-models-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
    background-clip: content-box;
  }

  /* Адаптивность */
  @media (max-width: 1200px) {
    .models-table th:nth-child(3),
    .models-table td:nth-child(3) {
      display: none;
    }
  }

  @media (max-width: 768px) {
    /* Скрываем дополнительные колонки на мобильных */
    .models-table th:nth-child(2),
    .models-table td:nth-child(2),
    .models-table th:nth-child(4),
    .models-table td:nth-child(4),
    .models-table th:nth-child(6),
    .models-table td:nth-child(6) {
      display: none;
    }

    .models-table th,
    .models-table td {
      padding: 0.625rem 0.75rem;
      font-size: 0.875rem;
    }
  }
</style>
