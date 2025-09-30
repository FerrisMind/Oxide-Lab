<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  
  // Props
  let { class: className = '' } = $props();
  
  // Состояние
  let isLoaded = $state(false);
  let isLoading = $state(false);
  let loadingProgress = $state(0);
  let loadingStage = $state('');
  let loadingMessage = $state('');
  let error = $state<string | null>(null);
  let currentDevice = $state('Не определено');
  let modelPath = $state<string | null>(null);
  
  // Слушатели событий
  let listeners: UnlistenFn[] = [];
  
  // Инициализация и проверка состояния модели
  onMount(async () => {
    // Проверяем, загружена ли модель
    try {
      isLoaded = await invoke<boolean>('is_model_loaded');
      
      // Получаем информацию об устройстве
      const deviceInfo = await invoke<{device: string}>('get_device_info');
      currentDevice = deviceInfo.device;
    } catch (err) {
      console.error('Failed to get model status:', err);
    }
    
    // Подписываемся на события загрузки
    const progressListener = await listen<{
      stage: string;
      progress: number;
      message?: string;
      done: boolean;
      error?: string;
    }>('load_progress', (event) => {
      const { stage, progress, message, done, error: loadError } = event.payload;
      
      loadingStage = stage;
      loadingProgress = progress;
      loadingMessage = message || '';
      
      if (loadError) {
        error = loadError;
        isLoading = false;
        isLoaded = false;
      } else if (done) {
        if (stage === 'complete') {
          isLoaded = true;
          isLoading = false;
          error = null;
        } else if (stage === 'unload_complete') {
          isLoaded = false;
          isLoading = false;
          modelPath = null;
        } else if (stage === 'cancel') {
          isLoading = false;
        }
      } else {
        isLoading = true;
        error = null;
      }
    });
    
    listeners.push(progressListener);
  });
  
  onDestroy(() => {
    listeners.forEach(unlisten => unlisten());
  });
  
  // Вспомогательные функции для отображения
  function getStatusText(): string {
    if (error) return 'Ошибка';
    if (isLoading) return 'Загрузка...';
    if (isLoaded) return 'Загружена';
    return 'Не загружена';
  }
  
  function getStatusClass(): string {
    if (error) return 'status-error';
    if (isLoading) return 'status-loading';
    if (isLoaded) return 'status-loaded';
    return 'status-idle';
  }
  
  function getStageText(): string {
    const stageMap: Record<string, string> = {
      'start': 'Начало загрузки',
      'device': 'Выбор устройства',
      'open_file': 'Открытие файла',
      'read_header': 'Чтение заголовка',
      'tokenizer': 'Загрузка токенизатора',
      'model_init': 'Инициализация модели',
      'finalize': 'Финализация',
      'complete': 'Завершено',
      'unload_start': 'Начало выгрузки',
      'unload_model': 'Выгрузка модели',
      'unload_tokenizer': 'Очистка токенизатора',
      'unload_complete': 'Выгрузка завершена',
      'cancel': 'Отменено',
      'error': 'Ошибка',
    };
    return stageMap[loadingStage] || loadingStage;
  }
  
  // Экспортируем состояние для родительских компонентов
  export function getStatus() {
    return {
      isLoaded,
      isLoading,
      loadingProgress,
      loadingStage,
      error,
      currentDevice,
      modelPath
    };
  }
</script>

<div class="model-status {className}" class:has-error={error}>
  <div class="status-header">
    <div class="status-indicator {getStatusClass()}">
      <span class="status-dot"></span>
      <span class="status-text">{getStatusText()}</span>
    </div>
    
    {#if currentDevice}
      <div class="device-info">
        <span class="device-label">Устройство:</span>
        <span class="device-value">{currentDevice}</span>
      </div>
    {/if}
  </div>
  
  {#if isLoading}
    <div class="loading-details">
      <div class="progress-bar">
        <div class="progress-fill" style="width: {loadingProgress}%"></div>
      </div>
      
      <div class="stage-info">
        <span class="stage-name">{getStageText()}</span>
        <span class="stage-progress">{loadingProgress}%</span>
      </div>
      
      {#if loadingMessage}
        <div class="loading-message">{loadingMessage}</div>
      {/if}
    </div>
  {/if}
  
  {#if error}
    <div class="error-details" role="alert">
      <strong>Ошибка загрузки:</strong>
      <p>{error}</p>
    </div>
  {/if}
  
  {#if isLoaded && modelPath}
    <div class="model-info">
      <div class="model-path" title={modelPath}>
        <strong>Модель:</strong> {modelPath.split(/[/\\]/).pop()}
      </div>
    </div>
  {/if}
</div>

<style>
  .model-status {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }
  
  .model-status.has-error {
    border-color: var(--error, #e74c3c);
  }
  
  .status-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }
  
  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
  }
  
  .status-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }
  
  .status-idle .status-dot {
    background: var(--muted, #95a5a6);
    animation: none;
  }
  
  .status-loading .status-dot {
    background: var(--info, #3498db);
  }
  
  .status-loaded .status-dot {
    background: var(--success, #2ecc71);
    animation: none;
  }
  
  .status-error .status-dot {
    background: var(--error, #e74c3c);
    animation: none;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .device-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--muted);
  }
  
  .device-label {
    font-weight: 500;
  }
  
  .device-value {
    padding: 0.25rem 0.5rem;
    background: rgba(52, 152, 219, 0.1);
    border-radius: 4px;
    color: var(--info, #3498db);
    font-family: monospace;
  }
  
  .loading-details {
    margin-top: 0.75rem;
  }
  
  .progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(149, 165, 166, 0.2);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }
  
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #3498db, #2ecc71);
    transition: width 0.3s ease;
    border-radius: 4px;
  }
  
  .stage-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
    color: var(--text);
  }
  
  .stage-name {
    font-weight: 500;
  }
  
  .stage-progress {
    color: var(--muted);
    font-family: monospace;
  }
  
  .loading-message {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(52, 152, 219, 0.1);
    border-radius: 4px;
    font-size: 0.875rem;
    color: var(--info, #3498db);
  }
  
  .error-details {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: rgba(231, 76, 60, 0.1);
    border: 1px solid rgba(231, 76, 60, 0.3);
    border-radius: 4px;
    color: var(--error, #e74c3c);
  }
  
  .error-details strong {
    display: block;
    margin-bottom: 0.25rem;
  }
  
  .error-details p {
    margin: 0;
    font-size: 0.875rem;
  }
  
  .model-info {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: rgba(46, 204, 113, 0.1);
    border-radius: 4px;
  }
  
  .model-path {
    font-size: 0.875rem;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .model-path strong {
    color: var(--success, #2ecc71);
    margin-right: 0.5rem;
  }
</style>
