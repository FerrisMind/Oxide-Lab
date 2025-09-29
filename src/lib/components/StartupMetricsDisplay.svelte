<script lang="ts">
  import { onMount } from 'svelte';
  import { performanceService } from '$lib/services/performance-service';
  import type { StartupMetrics } from '$lib/types/performance';
  import ChartBar from 'phosphor-svelte/lib/ChartBar';
  import Clock from 'phosphor-svelte/lib/Clock';
  import Memory from 'phosphor-svelte/lib/Memory';

  let metrics: StartupMetrics | null = null;
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      metrics = await performanceService.getStartupMetrics();
      
      // Подписываемся на обновления startup metrics
      await performanceService.setupEventListeners(
        undefined,
        undefined,
        (startupMetrics: StartupMetrics) => {
          metrics = startupMetrics;
        }
      );
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load startup metrics';
      console.error('Error loading startup metrics:', e);
    } finally {
      loading = false;
    }
  });

  function formatDuration(ms: number): string {
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    } else {
      return `${(ms / 1000).toFixed(2)}s`;
    }
  }

  function formatMemory(mb: number): string {
    if (mb < 1024) {
      return `${mb.toFixed(2)} MB`;
    } else {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
  }

  function getStatusColor(ms: number): string {
    if (ms < 3000) return 'text-green-400';
    if (ms < 10000) return 'text-yellow-400';
    return 'text-red-400';
  }
</script>

{#if loading}
  <div class="flex items-center justify-center p-6">
    <div class="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full"></div>
  </div>
{:else if error}
  <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
    <p class="text-red-400 text-sm">{error}</p>
  </div>
{:else if metrics}
  <div class="space-y-4">
    <!-- Общее время запуска -->
    <div class="p-4 bg-surface-700 rounded-lg border border-surface-600">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <Clock size={20} class="text-primary" weight="duotone" />
          <h3 class="font-semibold text-sm">Время запуска приложения</h3>
        </div>
        <span class={`font-mono text-lg font-bold ${getStatusColor(metrics.total_duration_ms)}`}>
          {formatDuration(metrics.total_duration_ms)}
        </span>
      </div>

      {#if metrics.total_duration_ms < 10000}
        <p class="text-xs text-surface-300">✅ Отлично! Приложение запускается быстро</p>
      {:else}
        <p class="text-xs text-yellow-400">⚠️ Время запуска превышает 10 секунд</p>
      {/if}
    </div>

    <!-- Использование памяти -->
    <div class="p-4 bg-surface-700 rounded-lg border border-surface-600">
      <div class="flex items-center gap-2 mb-3">
        <Memory size={20} class="text-primary" weight="duotone" />
        <h3 class="font-semibold text-sm">Использование памяти</h3>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <p class="text-xs text-surface-400 mb-1">При запуске</p>
          <p class="font-mono text-sm font-semibold">
            {formatMemory(metrics.memory_at_start_mb)}
          </p>
        </div>
        <div>
          <p class="text-xs text-surface-400 mb-1">После готовности</p>
          <p class="font-mono text-sm font-semibold">
            {formatMemory(metrics.memory_at_ready_mb)}
          </p>
        </div>
      </div>

      <div class="mt-3 pt-3 border-t border-surface-600">
        <p class="text-xs text-surface-400 mb-1">Прирост памяти</p>
        <p class="font-mono text-sm font-semibold text-primary">
          +{formatMemory(metrics.memory_at_ready_mb - metrics.memory_at_start_mb)}
        </p>
      </div>
    </div>

    <!-- Стадии запуска -->
    <div class="p-4 bg-surface-700 rounded-lg border border-surface-600">
      <div class="flex items-center gap-2 mb-3">
        <ChartBar size={20} class="text-primary" weight="duotone" />
        <h3 class="font-semibold text-sm">Стадии инициализации</h3>
      </div>

      <div class="space-y-2">
        {#each metrics.stages as stage}
          <div class="flex items-center justify-between p-2 bg-surface-800 rounded">
            <span class="text-sm text-surface-300 capitalize">
              {stage.name.replace(/_/g, ' ')}
            </span>
            <span class="font-mono text-sm font-medium">
              {formatDuration(stage.duration_ms)}
            </span>
          </div>
        {/each}
      </div>

      <!-- Прогресс-бар для визуализации стадий -->
      <div class="mt-4">
        <div class="h-2 bg-surface-800 rounded-full overflow-hidden flex">
          {#each metrics.stages as stage}
            {@const percentage = (stage.duration_ms / metrics.total_duration_ms) * 100}
            <div
              class="h-full bg-primary transition-all"
              style="width: {percentage}%"
              title="{stage.name}: {formatDuration(stage.duration_ms)}"
            ></div>
          {/each}
        </div>
        <p class="text-xs text-surface-400 mt-2 text-center">
          Распределение времени по стадиям
        </p>
      </div>
    </div>

    <!-- Временная метка -->
    <div class="text-xs text-surface-400 text-center">
      Данные получены: {new Date(metrics.timestamp).toLocaleString('ru-RU')}
    </div>
  </div>
{:else}
  <div class="p-4 bg-surface-700 rounded-lg border border-surface-600">
    <p class="text-sm text-surface-400 text-center">
      Метрики запуска пока недоступны
    </p>
  </div>
{/if}
