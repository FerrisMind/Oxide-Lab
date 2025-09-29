<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { performanceService } from '$lib/services/performance-service';
  import type { ModelLoadMetrics, InferenceMetrics, PerformanceSummary, StartupMetrics } from '$lib/types/performance';
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

  let summary: PerformanceSummary | null = null;
  let loading = false;
  let error: string | null = null;
  let autoRefresh = false;
  let refreshInterval: number | null = null;

  async function loadSummary() {
    loading = true;
    error = null;
    try {
      summary = await performanceService.getPerformanceSummary();
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

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    
    if (autoRefresh) {
      refreshInterval = window.setInterval(() => {
        loadSummary();
      }, 2000); // Обновление каждые 2 секунды
    } else if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  onMount(async () => {
    await loadSummary();
    
    // Подписываемся на события метрик
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
      }
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
      <button 
        on:click={toggleAutoRefresh} 
        class:active={autoRefresh}
        class="btn-auto-refresh"
      >
        {#if autoRefresh}
          <Pause size={16} />
        {:else}
          <Play size={16} />
        {/if}
        Авто
      </button>
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

  {#if summary}
    <div class="metrics-grid">
      <!-- Startup Metrics -->
      {#if summary.startup}
        <div class="metric-card full-width startup-card">
          <div class="metric-content">
            <StartupMetricsDisplay />
          </div>
        </div>
      {/if}

      <!-- Текущая память -->
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

      <!-- Последняя загрузка модели -->
      {#if summary.last_model_load}
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
              <br>
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
      {#if summary.last_inference}
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
              Prefill: {performanceService.formatDuration(summary.last_inference.prefill_duration_ms)}
              <br>
              Generation: {performanceService.formatDuration(summary.last_inference.generation_duration_ms)}
            </div>
          </div>
        </div>
      {/if}

      <!-- Общая статистика -->
      {#if summary.total_generated_tokens > 0}
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
    cursor: pointer;
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

  button.active {
    background: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
  }

  .btn-refresh {
    font-size: 1rem;
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
    background: linear-gradient(135deg, var(--bg-tertiary, #252525) 0%, rgba(74, 158, 255, 0.05) 100%);
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
