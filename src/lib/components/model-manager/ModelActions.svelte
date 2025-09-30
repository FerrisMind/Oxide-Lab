<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { HFModel } from '$lib/services/huggingface';
  
  // Props
  let {
    selectedModel = $bindable(null),
    isLoaded = false,
    isLoading = false,
    onModelLoad = () => {},
    onModelUnload = () => {},
    class: className = ''
  }: {
    selectedModel: HFModel | null;
    isLoaded: boolean;
    isLoading: boolean;
    onModelLoad?: () => void;
    onModelUnload?: () => void;
    class?: string;
  } = $props();
  
  // Локальное состояние
  let localModelPath = $state('');
  let contextLength = $state(4096);
  let useGPU = $state(true);
  let loadingFromLocal = $state(false);
  
  // Загрузка локальной модели
  async function loadLocalModel() {
    try {
      // Открываем диалог выбора файла
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Model Files',
          extensions: ['gguf', 'safetensors']
        }]
      });
      
      if (!selected || typeof selected !== 'string') {
        return;
      }
      
      localModelPath = selected;
      loadingFromLocal = true;
      
      // Определяем формат по расширению
      const ext = selected.split('.').pop()?.toLowerCase();
      
      if (ext === 'gguf') {
        await invoke('load_model', {
          req: {
            Gguf: {
              model_path: selected,
              tokenizer_path: null,
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' }
            }
          }
        });
      } else if (ext === 'safetensors') {
        await invoke('load_model', {
          req: {
            LocalSafetensors: {
              model_path: selected,
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' }
            }
          }
        });
      }
      
      onModelLoad();
    } catch (error) {
      console.error('Failed to load local model:', error);
    } finally {
      loadingFromLocal = false;
    }
  }
  
  // Загрузка модели из HuggingFace
  async function loadHFModel() {
    if (!selectedModel) return;
    
    try {
      // Проверяем доступные форматы
      const hasGGUF = selectedModel.tags?.includes('gguf');
      const hasSafetensors = selectedModel.tags?.includes('safetensors') || 
                             selectedModel.tags?.includes('pytorch');
      
      if (hasGGUF) {
        // Загружаем GGUF из HuggingFace Hub
        await invoke('load_model', {
          req: {
            HubGguf: {
              repo_id: selectedModel.id,
              revision: 'main',
              filename: selectedModel.id.split('/').pop() + '.gguf', // Упрощенное определение имени файла
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' }
            }
          }
        });
      } else if (hasSafetensors) {
        // Загружаем safetensors из HuggingFace Hub
        await invoke('load_model', {
          req: {
            HubSafetensors: {
              repo_id: selectedModel.id,
              revision: 'main',
              context_length: contextLength,
              device: useGPU ? { Preference: 'Auto' } : { Preference: 'Cpu' }
            }
          }
        });
      } else {
        throw new Error('Model does not have supported formats (GGUF or Safetensors)');
      }
      
      onModelLoad();
    } catch (error) {
      console.error('Failed to load HF model:', error);
    }
  }
  
  // Выгрузка модели
  async function unloadModel() {
    try {
      await invoke('unload_model');
      selectedModel = null;
      localModelPath = '';
      onModelUnload();
    } catch (error) {
      console.error('Failed to unload model:', error);
    }
  }
  
  // Отмена загрузки
  async function cancelLoading() {
    try {
      await invoke('cancel_model_loading');
    } catch (error) {
      console.error('Failed to cancel loading:', error);
    }
  }
</script>

<div class="model-actions {className}">
  <div class="actions-header">
    <h3>Действия с моделью</h3>
  </div>
  
  <div class="actions-content">
    <!-- Параметры загрузки -->
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
          disabled={isLoaded || isLoading}
        />
      </div>
      
      <div class="option-group">
        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={useGPU}
            disabled={isLoaded || isLoading}
          />
          <span>Использовать GPU (если доступно)</span>
        </label>
      </div>
    </div>
    
    <!-- Кнопки действий -->
    <div class="action-buttons">
      {#if isLoading}
        <button
          class="btn btn-danger"
          onclick={cancelLoading}
        >
          Отменить загрузку
        </button>
      {:else if isLoaded}
        <button
          class="btn btn-danger"
          onclick={unloadModel}
        >
          Выгрузить модель
        </button>
      {:else}
        <button
          class="btn btn-primary"
          onclick={loadLocalModel}
          disabled={loadingFromLocal}
        >
          Загрузить локальную модель
        </button>
        
        {#if selectedModel}
          <button
            class="btn btn-success"
            onclick={loadHFModel}
          >
            Загрузить {selectedModel.id}
          </button>
        {/if}
      {/if}
    </div>
    
    {#if localModelPath}
      <div class="local-model-info">
        <strong>Выбранный файл:</strong>
        <span class="file-path" title={localModelPath}>
          {localModelPath.split(/[/\\]/).pop()}
        </span>
      </div>
    {/if}
  </div>
</div>

<style>
  .model-actions {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .actions-header h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .load-options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1rem;
    padding: 1rem;
    background: rgba(149, 165, 166, 0.05);
    border-radius: 4px;
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
    color: var(--info, #3498db);
    font-family: monospace;
  }
  
  input[type="range"] {
    width: 100%;
    height: 6px;
    background: rgba(149, 165, 166, 0.2);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
  }
  
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent, #3498db);
    cursor: pointer;
    border-radius: 50%;
  }
  
  input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--accent, #3498db);
    cursor: pointer;
    border-radius: 50%;
    border: none;
  }
  
  input[type="range"]:disabled {
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
  
  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }
  
  .checkbox-label input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }
  
  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .btn {
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
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
  
  .btn-danger {
    background: var(--error, #e74c3c);
    color: white;
  }
  
  .btn-danger:hover:not(:disabled) {
    background: #c0392b;
  }
  
  .local-model-info {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(52, 152, 219, 0.1);
    border-radius: 4px;
    font-size: 0.875rem;
  }
  
  .local-model-info strong {
    display: block;
    margin-bottom: 0.25rem;
    color: var(--text);
  }
  
  .file-path {
    color: var(--muted);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
  }
</style>
