<script lang="ts">
  /**
   * Component for local models actions
   */

  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { LocalModelInfo } from '$lib/types/local-models';
  import { mount, unmount } from 'svelte';
  import FolderOpenIcon from 'phosphor-svelte/lib/FolderOpen';
  import ArrowClockwiseIcon from 'phosphor-svelte/lib/ArrowClockwise';

  let {
    selectedModel = $bindable(null),
    folderPath = '',
    isLoading = false,
    onScanFolder = async () => {},
    onLoadModel = async () => {},
  }: {
    selectedModel: LocalModelInfo | null;
    folderPath: string;
    isLoading: boolean;
    onScanFolder?: (path: string) => Promise<void>;
    onLoadModel?: (model: LocalModelInfo) => Promise<void>;
  } = $props();

  // Local state
  let contextLength = $state(4096);
  let useGPU = $state(true);

  // Handle folder selection
  async function selectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Выберите папку с моделями',
      });

      if (selected && typeof selected === 'string') {
        await onScanFolder(selected);
      }
    } catch (error) {
      console.error('Failed to select folder:', error);
    }
  }

  // Handle rescan
  async function handleRescan() {
    if (folderPath) {
      await onScanFolder(folderPath);
    }
  }

  // Handle model loading
  async function handleLoadModel() {
    if (!selectedModel) return;

    try {
      const ext = selectedModel.path.split('.').pop()?.toLowerCase();

      if (ext === 'gguf') {
        await invoke('load_model', {
          req: {
            Gguf: {
              model_path: selectedModel.path,
              tokenizer_path: null,
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' },
            },
          },
        });
      } else if (ext === 'safetensors') {
        await invoke('load_model', {
          req: {
            LocalSafetensors: {
              model_path: selectedModel.path,
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' },
            },
          },
        });
      }

      await onLoadModel(selectedModel);
    } catch (error) {
      console.error('Failed to load model:', error);
    }
  }

  // Icon mounting helper
  function mountIcon(element: HTMLElement, Component: any) {
    const icon = mount(Component, { target: element, props: { size: 20, weight: 'regular' } });
    return {
      destroy() {
        unmount(icon);
      },
    };
  }
</script>

<div class="local-model-actions">
  <div class="actions-header">
    <h3>Локальные модели</h3>
  </div>

  <div class="actions-content">
    <!-- Folder selection -->
    <div class="folder-section">
      <button class="btn btn-primary" onclick={selectFolder} disabled={isLoading}>
        <span use:mountIcon={FolderOpenIcon}></span>
        <span>Выбрать папку</span>
      </button>

      {#if folderPath}
        <div class="folder-info">
          <span class="folder-label">Текущая папка:</span>
          <span class="folder-path" title={folderPath}>
            {folderPath.split(/[/\\]/).pop()}
          </span>
          <button
            class="rescan-btn"
            aria-label="Пересканировать папку"
            onclick={handleRescan}
            disabled={isLoading}
            title="Пересканировать папку"
          >
            <span use:mountIcon={ArrowClockwiseIcon}></span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Model loading options -->
    {#if selectedModel}
      <div class="load-section">
        <h4>Загрузить модель</h4>

        <div class="selected-model">
          <span class="model-name">{selectedModel.name}</span>
          {#if selectedModel.parameters}
            <span class="model-info">{selectedModel.parameters}</span>
          {/if}
          {#if selectedModel.quantization}
            <span class="model-info">{selectedModel.quantization}</span>
          {/if}
        </div>

        <div class="load-options">
          <div class="option-group">
            <label for="context-length">
              Длина контекста:
              <span class="option-value">{contextLength}</span>
            </label>
            <input
              id="context-length"
              type="range"
              bind:value={contextLength}
              min="512"
              max="8192"
              step="512"
              disabled={isLoading}
            />
          </div>

          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={useGPU} disabled={isLoading} />
              <span>Использовать GPU (если доступно)</span>
            </label>
          </div>
        </div>

        <button class="btn btn-success" onclick={handleLoadModel} disabled={isLoading}>
          Загрузить модель
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .local-model-actions {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .actions-header h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }

  .actions-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .folder-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .folder-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    background: rgba(52, 152, 219, 0.05);
    border-radius: 4px;
  }

  .folder-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--muted);
  }

  .folder-path {
    font-size: 0.9375rem;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rescan-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border: none;
    background: transparent;
    color: var(--accent, #3498db);
    cursor: pointer;
    border-radius: 4px;
    transition: background-color 0.2s ease;
  }

  .rescan-btn:hover:not(:disabled) {
    background: rgba(52, 152, 219, 0.1);
  }

  .rescan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .load-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    background: rgba(46, 204, 113, 0.05);
    border: 1px solid rgba(46, 204, 113, 0.2);
    border-radius: 6px;
  }

  .load-section h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .selected-model {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .model-name {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--text);
  }

  .model-info {
    font-size: 0.8125rem;
    color: var(--muted);
  }

  .load-options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .option-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .option-group label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text);
  }

  .option-value {
    padding: 0.25rem 0.5rem;
    background: rgba(52, 152, 219, 0.1);
    border-radius: 4px;
    color: var(--accent, #3498db);
    font-family: monospace;
  }

  input[type='range'] {
    width: 100%;
    height: 6px;
    background: rgba(149, 165, 166, 0.2);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
  }

  input[type='range']::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent, #3498db);
    cursor: pointer;
    border-radius: 50%;
  }

  input[type='range']::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--accent, #3498db);
    cursor: pointer;
    border-radius: 50%;
    border: none;
  }

  input[type='range']:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    user-select: none;
  }

  .checkbox-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .checkbox-label input[type='checkbox']:disabled {
    cursor: not-allowed;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9375rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    width: 100%;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent, #3498db);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover, #2980b9);
  }

  .btn-success {
    background: var(--success, #2ecc71);
    color: white;
  }

  .btn-success:hover:not(:disabled) {
    background: #27ae60;
  }

  /* Адаптивность */
  @media (max-width: 768px) {
    .local-model-actions {
      padding: 0.875rem;
    }

    .actions-header h3 {
      font-size: 1rem;
      margin-bottom: 0.75rem;
    }

    .actions-content {
      gap: 1.25rem;
    }

    .load-section {
      padding: 0.875rem;
    }

    .load-section h4 {
      font-size: 0.9375rem;
    }

    .btn {
      padding: 0.625rem 0.875rem;
      font-size: 0.875rem;
    }

    .option-group label {
      font-size: 0.8125rem;
    }

    .option-value {
      font-size: 0.8125rem;
    }
  }

  @media (max-width: 480px) {
    .local-model-actions {
      padding: 0.75rem;
    }

    .actions-header h3 {
      font-size: 0.9375rem;
    }

    .folder-path {
      font-size: 0.875rem;
    }

    .folder-label {
      font-size: 0.75rem;
    }

    .load-section h4 {
      font-size: 0.875rem;
    }

    .model-name {
      font-size: 0.875rem;
    }

    .model-info {
      font-size: 0.75rem;
    }

    .btn {
      padding: 0.5rem 0.75rem;
      font-size: 0.8125rem;
      gap: 0.375rem;
    }

    .option-group {
      gap: 0.375rem;
    }

    .checkbox-label {
      gap: 0.375rem;
    }
  }
</style>
