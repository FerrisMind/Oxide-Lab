<script lang="ts">
  import { onMount } from 'svelte';
  import { performanceService } from '$lib/services/performance-service';
  import type { InferenceMetrics, ModelLoadMetrics } from '$lib/types/performance';

  // MVP целевые показатели
  const MVP_TARGETS = {
    ttft: 3000, // Time To First Token в миллисекундах (3 секунды)
    tokensPerSecond: 10, // Минимум токенов в секунду
    modelLoadTime: 30000, // Максимальное время загрузки модели (30 сек)
    memoryLimit: 8192, // Лимит памяти в MB (8GB)
    startupTime: 5000, // Время запуска приложения (5 сек)
  };

  // Props
  let { class: className = '' } = $props();

  // Состояние метрик
  let lastInference = $state<InferenceMetrics | null>(null);
  let lastModelLoad = $state<ModelLoadMetrics | null>(null);
  let averageTokensPerSecond = $state(0);
  let totalGeneratedTokens = $state(0);
  let currentMemory = $state(0);

  // Обновление метрик
  async function updateMetrics() {
    try {
      const summary = await performanceService.getPerformanceSummary();

      lastInference = summary.last_inference || null;
      lastModelLoad = summary.last_model_load || null;
      averageTokensPerSecond = summary.average_tokens_per_second;
      totalGeneratedTokens = summary.total_generated_tokens;
      currentMemory = summary.current_memory_mb;
    } catch (error) {
      console.error('Failed to update metrics:', error);
    }
  }

  // Проверка соответствия целям
  function checkTarget(
    value: number,
    target: number,
    lowerIsBetter: boolean = true,
  ): 'good' | 'warning' | 'critical' {
    const ratio = value / target;

    if (lowerIsBetter) {
      // Для метрик, где меньше = лучше (время загрузки, TTFT)
      if (ratio <= 0.75) return 'good';
      if (ratio <= 1.0) return 'warning';
      return 'critical';
    } else {
      // Для метрик, где больше = лучше (токены/сек)
      if (ratio >= 1.0) return 'good';
      if (ratio >= 0.75) return 'warning';
      return 'critical';
    }
  }

  // Получение TTFT из последнего inference
  function getTTFT(): number | null {
    return lastInference?.prefill_duration_ms || null;
  }

  // Форматирование времени
  function formatDuration(ms: number): string {
    return performanceService.formatDuration(ms);
  }

  // Форматирование памяти
  function formatMemory(mb: number): string {
    return performanceService.formatMemory(mb);
  }

  // Форматирование скорости
  function formatSpeed(tokensPerSec: number): string {
    return performanceService.formatSpeed(tokensPerSec);
  }

  // Очистка метрик
  async function clearMetrics() {
    try {
      await performanceService.clearMetrics();
      await updateMetrics();
    } catch (error) {
      console.error('Failed to clear metrics:', error);
    }
  }

  // Инициализация
  onMount(() => {
    updateMetrics();

    // Подписываемся на события
    performanceService.setupEventListeners(
      () => {
        updateMetrics();
      },
      () => {
        updateMetrics();
      },
      () => {
        updateMetrics();
      },
    );

    // Обновляем каждые 5 секунд
    const interval = setInterval(updateMetrics, 5000);

    return () => {
      clearInterval(interval);
      performanceService.cleanup();
    };
  });
</script>

<div class="performance-metrics {className}">
  <div class="metrics-header">
    <h3>Метрики производительности</h3>
    <button class="btn btn-clear" onclick={clearMetrics} title="Очистить собранные метрики">
      Очистить
    </button>
  </div>

  <!-- MVP целевые показатели -->
  <div class="mvp-targets">
    <h4>Целевые показатели MVP</h4>

    <div class="targets-grid">
      <!-- TTFT (Time To First Token) -->
      <div
        class="target-card target-{getTTFT() ? checkTarget(getTTFT()!, MVP_TARGETS.ttft) : 'none'}"
      >
        <div class="target-label">TTFT</div>
        <div class="target-value">
          {#if getTTFT()}
            {formatDuration(getTTFT()!)}
          {:else}
            <span class="no-data">—</span>
          {/if}
        </div>
        <div class="target-goal">Цель: &lt; {formatDuration(MVP_TARGETS.ttft)}</div>
        {#if getTTFT()}
          <div class="target-status">
            {#if checkTarget(getTTFT()!, MVP_TARGETS.ttft) === 'good'}
              ✓ В пределах нормы
            {:else if checkTarget(getTTFT()!, MVP_TARGETS.ttft) === 'warning'}
              ⚠ Близко к лимиту
            {:else}
              ✗ Превышает цель
            {/if}
          </div>
        {/if}
      </div>

      <!-- Токены/сек -->
      <div
        class="target-card target-{averageTokensPerSecond > 0
          ? checkTarget(averageTokensPerSecond, MVP_TARGETS.tokensPerSecond, false)
          : 'none'}"
      >
        <div class="target-label">Скорость генерации</div>
        <div class="target-value">
          {#if averageTokensPerSecond > 0}
            {formatSpeed(averageTokensPerSecond)}
          {:else}
            <span class="no-data">—</span>
          {/if}
        </div>
        <div class="target-goal">Цель: &gt; {MVP_TARGETS.tokensPerSecond} t/s</div>
        {#if averageTokensPerSecond > 0}
          <div class="target-status">
            {#if checkTarget(averageTokensPerSecond, MVP_TARGETS.tokensPerSecond, false) === 'good'}
              ✓ Соответствует цели
            {:else if checkTarget(averageTokensPerSecond, MVP_TARGETS.tokensPerSecond, false) === 'warning'}
              ⚠ Ниже цели
            {:else}
              ✗ Критически низко
            {/if}
          </div>
        {/if}
      </div>

      <!-- Время загрузки модели -->
      <div
        class="target-card target-{lastModelLoad
          ? checkTarget(lastModelLoad.total_duration_ms, MVP_TARGETS.modelLoadTime)
          : 'none'}"
      >
        <div class="target-label">Загрузка модели</div>
        <div class="target-value">
          {#if lastModelLoad}
            {formatDuration(lastModelLoad.total_duration_ms)}
          {:else}
            <span class="no-data">—</span>
          {/if}
        </div>
        <div class="target-goal">Цель: &lt; {formatDuration(MVP_TARGETS.modelLoadTime)}</div>
        {#if lastModelLoad}
          <div class="target-status">
            {#if checkTarget(lastModelLoad.total_duration_ms, MVP_TARGETS.modelLoadTime) === 'good'}
              ✓ Быстрая загрузка
            {:else if checkTarget(lastModelLoad.total_duration_ms, MVP_TARGETS.modelLoadTime) === 'warning'}
              ⚠ Приемлемая скорость
            {:else}
              ✗ Медленная загрузка
            {/if}
          </div>
        {/if}
      </div>

      <!-- Использование памяти -->
      <div
        class="target-card target-{currentMemory > 0
          ? checkTarget(currentMemory, MVP_TARGETS.memoryLimit)
          : 'none'}"
      >
        <div class="target-label">Использование памяти</div>
        <div class="target-value">
          {#if currentMemory > 0}
            {formatMemory(currentMemory)}
          {:else}
            <span class="no-data">—</span>
          {/if}
        </div>
        <div class="target-goal">Лимит: &lt; {formatMemory(MVP_TARGETS.memoryLimit)}</div>
        {#if currentMemory > 0}
          <div class="target-status">
            {#if checkTarget(currentMemory, MVP_TARGETS.memoryLimit) === 'good'}
              ✓ Низкое потребление
            {:else if checkTarget(currentMemory, MVP_TARGETS.memoryLimit) === 'warning'}
              ⚠ Умеренное потребление
            {:else}
              ✗ Высокое потребление
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Детальные метрики inference -->
  {#if lastInference}
    <div class="detailed-metrics">
      <h4>Последняя генерация</h4>

      <div class="metrics-grid">
        <div class="metric-item">
          <span class="metric-label">Токенов промпта:</span>
          <span class="metric-value">{lastInference.prompt_tokens}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Сгенерировано:</span>
          <span class="metric-value">{lastInference.generated_tokens}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Общее время:</span>
          <span class="metric-value">{formatDuration(lastInference.total_duration_ms)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Prefill:</span>
          <span class="metric-value">{formatDuration(lastInference.prefill_duration_ms)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Generation:</span>
          <span class="metric-value">{formatDuration(lastInference.generation_duration_ms)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Prefill speed:</span>
          <span class="metric-value">{formatSpeed(lastInference.prefill_tokens_per_second)}</span>
        </div>
      </div>
    </div>
  {/if}

  <!-- Метрики загрузки модели -->
  {#if lastModelLoad}
    <div class="detailed-metrics">
      <h4>Загрузка модели</h4>

      <div class="metrics-grid">
        <div class="metric-item">
          <span class="metric-label">Размер модели:</span>
          <span class="metric-value">{formatMemory(lastModelLoad.model_size_mb)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Память до:</span>
          <span class="metric-value">{formatMemory(lastModelLoad.memory_before_mb)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Память после:</span>
          <span class="metric-value">{formatMemory(lastModelLoad.memory_after_mb)}</span>
        </div>
        <div class="metric-item">
          <span class="metric-label">Прирост:</span>
          <span class="metric-value">{formatMemory(lastModelLoad.memory_delta_mb)}</span>
        </div>
      </div>

      <!-- Стадии загрузки -->
      <div class="stages">
        <h5>Стадии загрузки</h5>
        {#each lastModelLoad.stages as stage}
          <div class="stage-item">
            <span class="stage-name">{stage.name}</span>
            <span class="stage-duration">{formatDuration(stage.duration_ms)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Статистика -->
  <div class="stats-summary">
    <div class="stat">
      <span class="stat-label">Всего сгенерировано:</span>
      <span class="stat-value">{totalGeneratedTokens} токенов</span>
    </div>
  </div>
</div>

<style>
  .performance-metrics {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .metrics-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .btn-clear {
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text);
    cursor: default;
    transition: all 0.2s ease;
  }

  .btn-clear:hover {
    background: rgba(149, 165, 166, 0.1);
  }

  .mvp-targets {
    margin-bottom: 2rem;
  }

  .mvp-targets h4 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .targets-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .target-card {
    padding: 1rem;
    border: 2px solid transparent;
    border-radius: 6px;
    background: rgba(149, 165, 166, 0.05);
    transition: all 0.2s ease;
  }

  .target-card.target-good {
    border-color: var(--success, #2ecc71);
    background: rgba(46, 204, 113, 0.1);
  }

  .target-card.target-warning {
    border-color: var(--warning, #f39c12);
    background: rgba(243, 156, 18, 0.1);
  }

  .target-card.target-critical {
    border-color: var(--error, #e74c3c);
    background: rgba(231, 76, 60, 0.1);
  }

  .target-label {
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    margin-bottom: 0.5rem;
  }

  .target-value {
    font-size: 1.75rem;
    font-weight: 700;
    font-family: monospace;
    color: var(--text);
    margin-bottom: 0.25rem;
  }

  .no-data {
    color: var(--muted);
    font-size: 1.5rem;
  }

  .target-goal {
    font-size: 0.75rem;
    color: var(--muted);
    margin-bottom: 0.5rem;
  }

  .target-status {
    font-size: 0.75rem;
    font-weight: 500;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    display: inline-block;
  }

  .target-good .target-status {
    color: var(--success, #2ecc71);
    background: rgba(46, 204, 113, 0.1);
  }

  .target-warning .target-status {
    color: var(--warning, #f39c12);
    background: rgba(243, 156, 18, 0.1);
  }

  .target-critical .target-status {
    color: var(--error, #e74c3c);
    background: rgba(231, 76, 60, 0.1);
  }

  .detailed-metrics {
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 1px solid var(--border-color);
  }

  .detailed-metrics h4,
  .detailed-metrics h5 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .detailed-metrics h5 {
    font-size: 0.875rem;
    margin-top: 1rem;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .metric-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: rgba(149, 165, 166, 0.05);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .metric-label {
    color: var(--muted);
  }

  .metric-value {
    font-weight: 600;
    font-family: monospace;
    color: var(--text);
  }

  .stages {
    margin-top: 1rem;
  }

  .stage-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    border-bottom: 1px solid rgba(149, 165, 166, 0.1);
    font-size: 0.875rem;
  }

  .stage-item:last-child {
    border-bottom: none;
  }

  .stage-name {
    color: var(--text);
  }

  .stage-duration {
    font-family: monospace;
    color: var(--muted);
  }

  .stats-summary {
    margin-top: 2rem;
    padding: 1rem;
    background: rgba(52, 152, 219, 0.05);
    border-radius: 6px;
  }

  .stat {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
  }

  .stat-label {
    font-weight: 500;
    color: var(--text);
  }

  .stat-value {
    font-family: monospace;
    font-weight: 600;
    color: var(--info, #3498db);
  }
</style>
