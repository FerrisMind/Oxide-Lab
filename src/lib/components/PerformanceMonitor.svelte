<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { performanceService } from '$lib/services/performance-service';
  import type {
    ModelLoadMetrics,
    InferenceMetrics,
    PerformanceSummary,
    StartupMetrics,
    SystemUsage,
  } from '$lib/types/performance';
  import StartupMetricsDisplay from './StartupMetricsDisplay.svelte';
  import ChartBar from 'phosphor-svelte/lib/ChartBar';
  import ArrowClockwise from 'phosphor-svelte/lib/ArrowClockwise';
  import Play from 'phosphor-svelte/lib/Play';
  import Pause from 'phosphor-svelte/lib/Pause';
  import Trash from 'phosphor-svelte/lib/Trash';
  import Memory from 'phosphor-svelte/lib/Memory';
  import Package from 'phosphor-svelte/lib/Package';
  import Lightning from 'phosphor-svelte/lib/Lightning';
  import Clock from 'phosphor-svelte/lib/Clock';
  import TrendUp from 'phosphor-svelte/lib/TrendUp';
  import Warning from 'phosphor-svelte/lib/Warning';
  import Cpu from 'phosphor-svelte/lib/Cpu';
  import GraphicsCard from 'phosphor-svelte/lib/GraphicsCard';

  let summary: PerformanceSummary | null = null;
  let systemUsage: SystemUsage | null = null;
  let loading = false;
  let error: string | null = null;
  let autoRefresh = true; // Включено по умолчанию для постоянного мониторинга
  let refreshInterval: number | null = null;

  async function loadSummary() {
    loading = true;
    error = null;
    try {
      // Загружаем данные параллельно для лучшей производительности
      const [summaryData, systemUsageData] = await Promise.allSettled([
        performanceService.getPerformanceSummary(),
        performanceService.getSystemUsage(),
      ]);

      // Обрабатываем результаты
      if (summaryData.status === 'fulfilled') {
        summary = summaryData.value;
      } else {
        console.error('Failed to load performance summary:', summaryData.reason);
      }

      if (systemUsageData.status === 'fulfilled') {
        systemUsage = systemUsageData.value;
        console.log('System usage loaded:', systemUsage);
      } else {
        console.error('Failed to load system usage:', systemUsageData.reason);
        systemUsage = null;
      }

      // Если оба запроса провалились, показываем ошибку
      if (summaryData.status === 'rejected' && systemUsageData.status === 'rejected') {
        error = 'Не удалось загрузить данные мониторинга производительности';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load performance summary';
      console.error('Error loading performance summary:', e);
    } finally {
      loading = false;
    }
  }

  async function clearMetrics() {
    try {
      await performanceService.clearMetrics();
      summary = null;
      await loadSummary();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to clear metrics';
      console.error('Error clearing metrics:', e);
    }
  }

  function startAutoRefresh() {
    if (!autoRefresh && !refreshInterval) {
      refreshInterval = window.setInterval(() => {
        loadSummary();
      }, 1000); // Обновление каждую секунду для реального времени
    }
  }

  function stopAutoRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;

    if (autoRefresh) {
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  }

  onMount(async () => {
    await loadSummary();

    // Автоматически включаем реальное время обновление
    startAutoRefresh();

    // Подписываемся на события метрик для дополнительного обновления
    await performanceService.setupEventListeners(
      (modelLoadMetrics: ModelLoadMetrics) => {
        console.log('Received model load metrics:', modelLoadMetrics);
        loadSummary();
      },
      (inferenceMetrics: InferenceMetrics) => {
        console.log('Received inference metrics:', inferenceMetrics);
        loadSummary();
      },
      (startupMetrics: StartupMetrics) => {
        console.log('✅ Received startup metrics:', startupMetrics);
        loadSummary();
      },
    );
  });

  onDestroy(() => {
    performanceService.cleanup();
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
</script>

<div class="performance-monitor">
  <div class="header">
    <h3>
      <ChartBar size={20} class="inline mr-2" />
      Мониторинг производительности
    </h3>
    <div class="actions">
      <button on:click={loadSummary} disabled={loading} class="btn-refresh">
        <ArrowClockwise size={16} class={loading ? 'animate-spin' : ''} />
        Обновить
      </button>
      <div class="status-indicator" class:active={autoRefresh}>
        <div class="pulse-dot"></div>
        <span>Реальное время</span>
      </div>
      <button on:click={clearMetrics} class="btn-clear">
        <Trash size={16} />
        Очистить
      </button>
    </div>
  </div>

  {#if error}
    <div class="error">
      <Warning size={16} class="inline mr-2" />
      {error}
    </div>
  {/if}

  {#if summary || systemUsage}
    <div class="metrics-grid">
      <!-- System Resources -->
      {#if systemUsage}
        <!-- CPU Usage -->
        <div class="metric-card">
          <div class="metric-icon">
            <Cpu size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Нагрузка CPU</div>
            <div class="metric-value">
              {systemUsage.cpu_usage_percent.toFixed(1)}%
            </div>
            <div class="metric-sub">Системное использование процессора</div>
          </div>
        </div>

        <!-- GPU Usage (если доступно) -->
        {#if systemUsage.gpu_usage_percent !== undefined && systemUsage.gpu_usage_percent !== null}
          <div class="metric-card">
            <div class="metric-icon">
              <GraphicsCard size={32} />
            </div>
            <div class="metric-content">
              <div class="metric-label">Нагрузка GPU</div>
              <div class="metric-value">
                {systemUsage.gpu_usage_percent.toFixed(1)}%
              </div>
              <div class="metric-sub">
                {#if systemUsage.gpu_memory_mb !== undefined && systemUsage.gpu_memory_mb !== null}
                  Память: {performanceService.formatMemory(systemUsage.gpu_memory_mb)}
                {:else}
                  Графический процессор
                {/if}
              </div>
            </div>
          </div>
        {/if}
      {/if}

      <!-- Startup Metrics -->
      {#if summary && summary.startup}
        <div class="metric-card full-width startup-card">
          <div class="metric-content">
            <StartupMetricsDisplay />
          </div>
        </div>
      {/if}

      <!-- Текущая память -->
      {#if summary}
        <div class="metric-card">
          <div class="metric-icon">
            <Memory size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Использование памяти</div>
            <div class="metric-value">
              {performanceService.formatMemory(summary.current_memory_mb)}
            </div>
          </div>
        </div>
      {/if}

      <!-- Последняя загрузка модели -->
      {#if summary && summary.last_model_load}
        <div class="metric-card">
          <div class="metric-icon">
            <Package size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Время загрузки модели</div>
            <div class="metric-value">
              {performanceService.formatDuration(summary.last_model_load.total_duration_ms)}
            </div>
            <div class="metric-sub">
              Размер: {performanceService.formatMemory(summary.last_model_load.model_size_mb)}
              <br />
              Δ памяти: {performanceService.formatMemory(summary.last_model_load.memory_delta_mb)}
            </div>
          </div>
        </div>

        <!-- Стадии загрузки -->
        <div class="metric-card full-width">
          <div class="metric-icon">
            <Clock size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Стадии загрузки</div>
            <div class="stages">
              {#each summary.last_model_load.stages as stage}
                <div class="stage">
                  <span class="stage-name">{stage.name}</span>
                  <span class="stage-duration">
                    {performanceService.formatDuration(stage.duration_ms)}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}

      <!-- Последний inference -->
      {#if summary && summary.last_inference}
        <div class="metric-card">
          <div class="metric-icon">
            <Lightning size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Скорость генерации</div>
            <div class="metric-value">
              {performanceService.formatSpeed(summary.last_inference.tokens_per_second)}
            </div>
            <div class="metric-sub">
              Токены: {summary.last_inference.generated_tokens}
            </div>
          </div>
        </div>

        <div class="metric-card">
          <div class="metric-icon">
            <Clock size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Время inference</div>
            <div class="metric-value">
              {performanceService.formatDuration(summary.last_inference.total_duration_ms)}
            </div>
            <div class="metric-sub">
              Prefill: {performanceService.formatDuration(
                summary.last_inference.prefill_duration_ms,
              )}
              <br />
              Generation: {performanceService.formatDuration(
                summary.last_inference.generation_duration_ms,
              )}
            </div>
          </div>
        </div>
      {/if}

      <!-- Общая статистика -->
      {#if summary && summary.total_generated_tokens > 0}
        <div class="metric-card">
          <div class="metric-icon">
            <TrendUp size={32} />
          </div>
          <div class="metric-content">
            <div class="metric-label">Средняя скорость</div>
            <div class="metric-value">
              {performanceService.formatSpeed(summary.average_tokens_per_second)}
            </div>
            <div class="metric-sub">
              Всего токенов: {summary.total_generated_tokens}
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else if !loading}
    <div class="no-data">
      <ChartBar size={24} class="inline mr-2 opacity-50" />
      Нет данных о производительности
    </div>
  {/if}
</div>

<style>
  .performance-monitor {
    padding: 1rem;
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 8px;
    color: var(--text-primary, #e0e0e0);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .header h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.4rem 0.8rem;
    border: 1px solid var(--border-color, #444);
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border-radius: 4px;
    cursor: default;
    font-size: 0.85rem;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  button:hover:not(:disabled) {
    background: var(--bg-hover, #333);
    transform: translateY(-1px);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-refresh {
    font-size: 1rem;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.8rem;
    background: var(--bg-tertiary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: var(--text-secondary, #888);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .status-indicator.active {
    color: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-secondary, #888);
    animation: pulse 2s infinite;
  }

  .status-indicator.active .pulse-dot {
    background: var(--accent-color, #4a9eff);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(1.2);
    }
  }

  .error {
    padding: 0.75rem;
    background: rgba(255, 68, 68, 0.1);
    border: 1px solid rgba(255, 68, 68, 0.3);
    border-radius: 4px;
    color: #ff6b6b;
    margin-bottom: 1rem;
  }

  .no-data {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary, #888);
    font-size: 1rem;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .metric-card {
    background: var(--bg-tertiary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    gap: 1rem;
    transition: transform 0.2s;
  }

  .metric-card:hover {
    transform: translateY(-2px);
    border-color: var(--accent-color, #4a9eff);
  }

  .metric-card.full-width {
    grid-column: 1 / -1;
  }

  .startup-card {
    background: linear-gradient(
      135deg,
      var(--bg-tertiary, #252525) 0%,
      rgba(74, 158, 255, 0.05) 100%
    );
  }

  .metric-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .metric-content {
    flex: 1;
  }

  .metric-label {
    font-size: 0.85rem;
    color: var(--text-secondary, #999);
    margin-bottom: 0.25rem;
  }

  .metric-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent-color, #4a9eff);
    margin-bottom: 0.25rem;
  }

  .metric-sub {
    font-size: 0.75rem;
    color: var(--text-secondary, #888);
    line-height: 1.4;
  }

  .stages {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .stage {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem;
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 4px;
  }

  .stage-name {
    color: var(--text-primary, #e0e0e0);
    font-size: 0.85rem;
  }

  .stage-duration {
    color: var(--accent-color, #4a9eff);
    font-weight: 600;
    font-size: 0.85rem;
  }
</style>
