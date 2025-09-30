<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { performanceService } from '$lib/services/performance-service';
  
  // Props
  let { class: className = '' } = $props();
  
  // Состояние памяти
  let currentMemory = $state(0);
  let memoryBeforeLoad = $state(0);
  let memoryAfterLoad = $state(0);
  let memoryDelta = $state(0);
  let isLoaded = $state(false);
  
  // Пороговые значения для индикации (в MB)
  const WARNING_THRESHOLD = 4096; // 4GB
  const CRITICAL_THRESHOLD = 6144; // 6GB
  
  // Обновление данных о памяти
  async function updateMemoryUsage() {
    try {
      const memory = await performanceService.getMemoryUsage();
      currentMemory = memory;
      
      // Проверяем статус модели
      isLoaded = await invoke<boolean>('is_model_loaded');
    } catch (error) {
      console.error('Failed to get memory usage:', error);
    }
  }
  
  // Принудительная очистка памяти через выгрузку модели
  async function forceMemoryCleanup() {
    try {
      await invoke('unload_model');
      // Даем системе время на очистку
      await new Promise(resolve => setTimeout(resolve, 500));
      await updateMemoryUsage();
    } catch (error) {
      console.error('Failed to cleanup memory:', error);
    }
  }
  
  // Получение уровня предупреждения
  function getMemoryLevel(): 'normal' | 'warning' | 'critical' {
    if (currentMemory >= CRITICAL_THRESHOLD) return 'critical';
    if (currentMemory >= WARNING_THRESHOLD) return 'warning';
    return 'normal';
  }
  
  // Форматирование размера памяти
  function formatMemory(mb: number): string {
    if (mb < 1024) {
      return `${mb.toFixed(0)} MB`;
    }
    return `${(mb / 1024).toFixed(2)} GB`;
  }
  
  // Получение процента использования (относительно 8GB)
  function getMemoryPercentage(): number {
    const maxMemory = 8192; // 8GB в MB
    return Math.min((currentMemory / maxMemory) * 100, 100);
  }
  
  // Подписка на события загрузки модели
  onMount(() => {
    updateMemoryUsage();
    
    // Обновляем каждые 2 секунды
    const interval = setInterval(updateMemoryUsage, 2000);
    
    // Подписываемся на метрики загрузки модели
    performanceService.setupEventListeners(
      (metrics) => {
        memoryBeforeLoad = metrics.memory_before_mb;
        memoryAfterLoad = metrics.memory_after_mb;
        memoryDelta = metrics.memory_delta_mb;
      }
    );
    
    return () => {
      clearInterval(interval);
      performanceService.cleanup();
    };
  });
</script>

<div class="memory-monitor {className}">
  <div class="monitor-header">
    <h3>Использование памяти</h3>
    {#if isLoaded}
      <button
        class="btn btn-cleanup"
        onclick={forceMemoryCleanup}
        title="Выгрузить модель и освободить память"
      >
        Очистить память
      </button>
    {/if}
  </div>
  
  <div class="memory-stats">
    <!-- Текущее использование -->
    <div class="stat-card">
      <div class="stat-label">Текущее использование</div>
      <div class="stat-value stat-{getMemoryLevel()}">
        {formatMemory(currentMemory)}
      </div>
      
      <div class="memory-bar">
        <div 
          class="memory-fill memory-fill-{getMemoryLevel()}"
          style="width: {getMemoryPercentage()}%"
        ></div>
      </div>
      
      <div class="memory-percentage">
        {getMemoryPercentage().toFixed(1)}% от 8GB
      </div>
    </div>
    
    <!-- Дельта памяти после загрузки модели -->
    {#if memoryDelta > 0}
      <div class="stat-card">
        <div class="stat-label">Использовано моделью</div>
        <div class="stat-value">
          {formatMemory(memoryDelta)}
        </div>
        
        <div class="memory-details">
          <div class="detail-row">
            <span>До загрузки:</span>
            <span>{formatMemory(memoryBeforeLoad)}</span>
          </div>
          <div class="detail-row">
            <span>После загрузки:</span>
            <span>{formatMemory(memoryAfterLoad)}</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Предупреждения -->
  {#if getMemoryLevel() === 'critical'}
    <div class="alert alert-critical" role="alert">
      <strong>⚠️ Критическое использование памяти!</strong>
      <p>Использование памяти превышает 6GB. Рекомендуется выгрузить модель или перезапустить приложение.</p>
    </div>
  {:else if getMemoryLevel() === 'warning'}
    <div class="alert alert-warning" role="alert">
      <strong>⚠️ Высокое использование памяти</strong>
      <p>Использование памяти превышает 4GB. Рекомендуется следить за использованием памяти.</p>
    </div>
  {/if}
  
  <!-- Информация о VRAM (для GPU моделей) -->
  <div class="vram-info">
    <details>
      <summary>Информация о VRAM/GPU памяти</summary>
      <div class="info-content">
        <p>
          При использовании GPU память модели загружается в VRAM видеокарты.
          Выгрузка модели освобождает как системную RAM, так и VRAM.
        </p>
        <p>
          <strong>Гарантии очистки:</strong>
        </p>
        <ul>
          <li>При выгрузке модели все тензоры удаляются из памяти</li>
          <li>VRAM освобождается через CUDA memory pools</li>
          <li>Токенизатор и связанные ресурсы очищаются</li>
          <li>Rust drop implementation гарантирует освобождение ресурсов</li>
        </ul>
      </div>
    </details>
  </div>
</div>

<style>
  .memory-monitor {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .monitor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .monitor-header h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .btn-cleanup {
    padding: 0.5rem 0.75rem;
    background: var(--error, #e74c3c);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-cleanup:hover {
    background: #c0392b;
  }
  
  .memory-stats {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .stat-card {
    padding: 1rem;
    background: rgba(149, 165, 166, 0.05);
    border-radius: 6px;
  }
  
  .stat-label {
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    margin-bottom: 0.5rem;
  }
  
  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    font-family: monospace;
    margin-bottom: 0.75rem;
  }
  
  .stat-normal {
    color: var(--success, #2ecc71);
  }
  
  .stat-warning {
    color: var(--warning, #f39c12);
  }
  
  .stat-critical {
    color: var(--error, #e74c3c);
  }
  
  .memory-bar {
    width: 100%;
    height: 12px;
    background: rgba(149, 165, 166, 0.2);
    border-radius: 6px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }
  
  .memory-fill {
    height: 100%;
    border-radius: 6px;
    transition: width 0.3s ease, background 0.3s ease;
  }
  
  .memory-fill-normal {
    background: linear-gradient(90deg, #2ecc71, #27ae60);
  }
  
  .memory-fill-warning {
    background: linear-gradient(90deg, #f39c12, #e67e22);
  }
  
  .memory-fill-critical {
    background: linear-gradient(90deg, #e74c3c, #c0392b);
  }
  
  .memory-percentage {
    font-size: 0.875rem;
    color: var(--muted);
    text-align: right;
    font-family: monospace;
  }
  
  .memory-details {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(149, 165, 166, 0.2);
  }
  
  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }
  
  .detail-row:last-child {
    margin-bottom: 0;
  }
  
  .detail-row span:first-child {
    color: var(--muted);
  }
  
  .detail-row span:last-child {
    font-weight: 600;
    font-family: monospace;
  }
  
  .alert {
    margin-top: 1rem;
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.875rem;
  }
  
  .alert strong {
    display: block;
    margin-bottom: 0.25rem;
  }
  
  .alert p {
    margin: 0;
  }
  
  .alert-warning {
    background: rgba(243, 156, 18, 0.1);
    border: 1px solid rgba(243, 156, 18, 0.3);
    color: var(--warning, #f39c12);
  }
  
  .alert-critical {
    background: rgba(231, 76, 60, 0.1);
    border: 1px solid rgba(231, 76, 60, 0.3);
    color: var(--error, #e74c3c);
  }
  
  .vram-info {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }
  
  details summary {
    cursor: pointer;
    font-weight: 500;
    font-size: 0.875rem;
    color: var(--text);
    padding: 0.5rem;
    border-radius: 4px;
    transition: background 0.2s ease;
  }
  
  details summary:hover {
    background: rgba(149, 165, 166, 0.1);
  }
  
  .info-content {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: rgba(52, 152, 219, 0.05);
    border-radius: 4px;
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text);
  }
  
  .info-content p {
    margin: 0 0 0.75rem 0;
  }
  
  .info-content p:last-child {
    margin-bottom: 0;
  }
  
  .info-content ul {
    margin: 0.5rem 0 0 1.25rem;
    padding: 0;
  }
  
  .info-content li {
    margin-bottom: 0.25rem;
    color: var(--muted);
  }
</style>
