<script lang="ts">
  import { onMount } from 'svelte';
  import { performanceService } from '$lib/services/performance-service';
  import type { InferenceMetrics, ModelLoadMetrics } from '$lib/types/performance';
  
  // Типы алертов
  interface Alert {
    id: string;
    type: 'warning' | 'critical' | 'info';
    metric: string;
    message: string;
    value: string;
    target: string;
    timestamp: Date;
    dismissed: boolean;
  }
  
  // MVP целевые показатели
  const MVP_TARGETS = {
    ttft: 3000, // TTFT в миллисекундах
    tokensPerSecond: 10, // Минимум токенов/сек
    modelLoadTime: 30000, // Максимум времени загрузки модели
    memoryLimit: 8192, // Лимит памяти в MB
  };
  
  // Props
  let { class: className = '' } = $props();
  
  // Состояние алертов
  let alerts = $state<Alert[]>([]);
  let showAlerts = $state(true);
  
  // Создание нового алерта
  function createAlert(
    type: Alert['type'],
    metric: string,
    message: string,
    value: string,
    target: string
  ): Alert {
    return {
      id: `alert-${Date.now()}-${Math.random()}`,
      type,
      metric,
      message,
      value,
      target,
      timestamp: new Date(),
      dismissed: false,
    };
  }
  
  // Добавление алерта
  function addAlert(alert: Alert) {
    // Проверяем, нет ли уже такого алерта
    const exists = alerts.some(
      (a) => a.metric === alert.metric && a.message === alert.message && !a.dismissed
    );
    
    if (!exists) {
      alerts = [alert, ...alerts];
      
      // Автоматически скрываем info алерты через 10 секунд
      if (alert.type === 'info') {
        setTimeout(() => {
          dismissAlert(alert.id);
        }, 10000);
      }
    }
  }
  
  // Отклонение алерта
  function dismissAlert(id: string) {
    alerts = alerts.map((a) => (a.id === id ? { ...a, dismissed: true } : a));
    
    // Удаляем отклоненные алерты через 1 секунду (для анимации)
    setTimeout(() => {
      alerts = alerts.filter((a) => a.id !== id);
    }, 1000);
  }
  
  // Очистка всех алертов
  function clearAllAlerts() {
    alerts = alerts.map((a) => ({ ...a, dismissed: true }));
    setTimeout(() => {
      alerts = [];
    }, 1000);
  }
  
  // Проверка метрик и генерация алертов
  function checkMetrics(inference: InferenceMetrics | null, modelLoad: ModelLoadMetrics | null) {
    // TTFT
    if (inference) {
      const ttft = inference.prefill_duration_ms;
      
      if (ttft > MVP_TARGETS.ttft * 1.5) {
        // Критически медленно
        addAlert(
          createAlert(
            'critical',
            'TTFT',
            'Время до первого токена критически превышает цель',
            performanceService.formatDuration(ttft),
            `< ${performanceService.formatDuration(MVP_TARGETS.ttft)}`
          )
        );
      } else if (ttft > MVP_TARGETS.ttft) {
        // Предупреждение
        addAlert(
          createAlert(
            'warning',
            'TTFT',
            'Время до первого токена превышает целевое значение',
            performanceService.formatDuration(ttft),
            `< ${performanceService.formatDuration(MVP_TARGETS.ttft)}`
          )
        );
      } else if (ttft < MVP_TARGETS.ttft * 0.5) {
        // Отличный результат
        addAlert(
          createAlert(
            'info',
            'TTFT',
            'Отличная производительность TTFT!',
            performanceService.formatDuration(ttft),
            `< ${performanceService.formatDuration(MVP_TARGETS.ttft)}`
          )
        );
      }
      
      // Скорость генерации
      const tokensPerSec = inference.tokens_per_second;
      
      if (tokensPerSec < MVP_TARGETS.tokensPerSecond * 0.5) {
        // Критически медленно
        addAlert(
          createAlert(
            'critical',
            'Tokens/sec',
            'Скорость генерации критически низкая',
            performanceService.formatSpeed(tokensPerSec),
            `> ${MVP_TARGETS.tokensPerSecond} t/s`
          )
        );
      } else if (tokensPerSec < MVP_TARGETS.tokensPerSecond) {
        // Предупреждение
        addAlert(
          createAlert(
            'warning',
            'Tokens/sec',
            'Скорость генерации ниже целевой',
            performanceService.formatSpeed(tokensPerSec),
            `> ${MVP_TARGETS.tokensPerSecond} t/s`
          )
        );
      } else if (tokensPerSec > MVP_TARGETS.tokensPerSecond * 2) {
        // Отличный результат
        addAlert(
          createAlert(
            'info',
            'Tokens/sec',
            'Отличная скорость генерации!',
            performanceService.formatSpeed(tokensPerSec),
            `> ${MVP_TARGETS.tokensPerSecond} t/s`
          )
        );
      }
    }
    
    // Время загрузки модели
    if (modelLoad) {
      const loadTime = modelLoad.total_duration_ms;
      
      if (loadTime > MVP_TARGETS.modelLoadTime * 1.5) {
        addAlert(
          createAlert(
            'critical',
            'Model Load Time',
            'Время загрузки модели критически превышает цель',
            performanceService.formatDuration(loadTime),
            `< ${performanceService.formatDuration(MVP_TARGETS.modelLoadTime)}`
          )
        );
      } else if (loadTime > MVP_TARGETS.modelLoadTime) {
        addAlert(
          createAlert(
            'warning',
            'Model Load Time',
            'Время загрузки модели превышает целевое значение',
            performanceService.formatDuration(loadTime),
            `< ${performanceService.formatDuration(MVP_TARGETS.modelLoadTime)}`
          )
        );
      }
      
      // Использование памяти
      const memoryDelta = modelLoad.memory_delta_mb;
      
      if (memoryDelta > MVP_TARGETS.memoryLimit) {
        addAlert(
          createAlert(
            'critical',
            'Memory Usage',
            'Модель потребляет больше памяти, чем разрешено',
            performanceService.formatMemory(memoryDelta),
            `< ${performanceService.formatMemory(MVP_TARGETS.memoryLimit)}`
          )
        );
      } else if (memoryDelta > MVP_TARGETS.memoryLimit * 0.75) {
        addAlert(
          createAlert(
            'warning',
            'Memory Usage',
            'Модель потребляет много памяти',
            performanceService.formatMemory(memoryDelta),
            `< ${performanceService.formatMemory(MVP_TARGETS.memoryLimit)}`
          )
        );
      }
    }
  }
  
  // Форматирование времени
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('ru-RU', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }
  
  // Инициализация
  onMount(() => {
    // Подписываемся на события метрик
    performanceService.setupEventListeners(
      (modelLoad) => {
        checkMetrics(null, modelLoad);
      },
      (inference) => {
        checkMetrics(inference, null);
      }
    );
    
    // Проверяем текущие метрики при монтировании
    performanceService.getPerformanceSummary().then(summary => {
      checkMetrics(summary.last_inference || null, summary.last_model_load || null);
    });
    
    return () => {
      performanceService.cleanup();
    };
  });
</script>

<div class="metrics-alerts {className}">
  <div class="alerts-header">
    <div class="header-left">
      <h3>Алерты производительности</h3>
      {#if alerts.filter((a) => !a.dismissed).length > 0}
        <span class="alerts-count">
          {alerts.filter((a) => !a.dismissed).length}
        </span>
      {/if}
    </div>
    
    <div class="header-actions">
      <button
        class="btn btn-toggle"
        onclick={() => (showAlerts = !showAlerts)}
        title={showAlerts ? 'Скрыть алерты' : 'Показать алерты'}
      >
        {showAlerts ? '−' : '+'}
      </button>
      
      {#if alerts.filter((a) => !a.dismissed).length > 0}
        <button class="btn btn-clear" onclick={clearAllAlerts} title="Очистить все алерты">
          Очистить все
        </button>
      {/if}
    </div>
  </div>
  
  {#if showAlerts}
    <div class="alerts-list">
      {#if alerts.filter((a) => !a.dismissed).length === 0}
        <div class="empty-state">
          <p>✓ Нет активных алертов</p>
          <p class="hint">Все метрики в пределах нормы</p>
        </div>
      {:else}
        {#each alerts.filter((a) => !a.dismissed) as alert (alert.id)}
          <div class="alert alert-{alert.type}" class:dismissed={alert.dismissed}>
            <div class="alert-header">
              <div class="alert-type-icon">
                {#if alert.type === 'critical'}
                  ❌
                {:else if alert.type === 'warning'}
                  ⚠️
                {:else}
                  ℹ️
                {/if}
              </div>
              
              <div class="alert-metric">{alert.metric}</div>
              
              <div class="alert-time">{formatTime(alert.timestamp)}</div>
              
              <button
                class="btn btn-dismiss"
                onclick={() => dismissAlert(alert.id)}
                title="Закрыть алерт"
              >
                ×
              </button>
            </div>
            
            <div class="alert-body">
              <div class="alert-message">{alert.message}</div>
              
              <div class="alert-values">
                <div class="value-item">
                  <span class="value-label">Текущее:</span>
                  <span class="value-data">{alert.value}</span>
                </div>
                <div class="value-item">
                  <span class="value-label">Целевое:</span>
                  <span class="value-data">{alert.target}</span>
                </div>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .metrics-alerts {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .alerts-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .alerts-header h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .alerts-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 0.5rem;
    background: var(--error, #e74c3c);
    color: white;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }
  
  .header-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .btn {
    padding: 0.375rem 0.75rem;
    border: none;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-toggle {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text);
    width: 32px;
    height: 32px;
    padding: 0;
    font-size: 1.25rem;
  }
  
  .btn-toggle:hover {
    background: rgba(149, 165, 166, 0.1);
  }
  
  .btn-clear {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text);
  }
  
  .btn-clear:hover {
    background: rgba(149, 165, 166, 0.1);
  }
  
  .alerts-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 400px;
    overflow-y: auto;
  }
  
  .empty-state {
    padding: 2rem 1rem;
    text-align: center;
  }
  
  .empty-state p {
    margin: 0.5rem 0;
    color: var(--success, #2ecc71);
    font-weight: 500;
  }
  
  .hint {
    font-size: 0.75rem;
    color: var(--muted);
    font-weight: 400;
  }
  
  .alert {
    padding: 0.75rem;
    border-radius: 6px;
    border: 2px solid;
    transition: all 0.3s ease;
  }
  
  .alert.dismissed {
    opacity: 0;
    transform: translateX(100%);
  }
  
  .alert-critical {
    background: rgba(231, 76, 60, 0.1);
    border-color: var(--error, #e74c3c);
  }
  
  .alert-warning {
    background: rgba(243, 156, 18, 0.1);
    border-color: var(--warning, #f39c12);
  }
  
  .alert-info {
    background: rgba(52, 152, 219, 0.1);
    border-color: var(--info, #3498db);
  }
  
  .alert-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  
  .alert-type-icon {
    font-size: 1.25rem;
  }
  
  .alert-metric {
    flex: 1;
    font-weight: 600;
    font-size: 0.875rem;
    font-family: monospace;
    color: var(--text);
  }
  
  .alert-time {
    font-size: 0.75rem;
    color: var(--muted);
    font-family: monospace;
  }
  
  .btn-dismiss {
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text);
    font-size: 1.25rem;
    line-height: 1;
    border-radius: 4px;
  }
  
  .btn-dismiss:hover {
    background: rgba(0, 0, 0, 0.1);
  }
  
  .alert-message {
    font-size: 0.875rem;
    color: var(--text);
    margin-bottom: 0.5rem;
    line-height: 1.4;
  }
  
  .alert-values {
    display: flex;
    gap: 1rem;
  }
  
  .value-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .value-label {
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
  }
  
  .value-data {
    font-size: 0.875rem;
    font-weight: 600;
    font-family: monospace;
    color: var(--text);
  }
  
  /* Скроллбар */
  .alerts-list::-webkit-scrollbar {
    width: 6px;
  }
  
  .alerts-list::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .alerts-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 3px;
  }
  
  .alerts-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
  }
</style>
