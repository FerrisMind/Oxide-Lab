// Сервис для работы с метриками производительности
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  PerformanceMetric,
  ModelLoadMetrics,
  InferenceMetrics,
  PerformanceSummary,
  StartupMetrics,
  SystemUsage,
} from '$lib/types/performance';

export class PerformanceService {
  private listeners: UnlistenFn[] = [];
  private lastModelLoadMetrics: ModelLoadMetrics | null = null;
  private lastInferenceMetrics: InferenceMetrics | null = null;
  private inferenceHistory: InferenceMetrics[] = [];
  private startupMetrics: StartupMetrics | null = null;

  /**
   * Получить все метрики производительности
   */
  async getPerformanceMetrics(): Promise<PerformanceMetric[]> {
    try {
      return await invoke<PerformanceMetric[]>('get_performance_metrics');
    } catch (error) {
      console.error('Failed to get performance metrics:', error);
      throw error;
    }
  }

  /**
   * Получить среднюю длительность операции
   */
  async getAverageDuration(operationName: string): Promise<number | null> {
    try {
      return await invoke<number | null>('get_average_duration', { operationName });
    } catch (error) {
      console.error(`Failed to get average duration for ${operationName}:`, error);
      throw error;
    }
  }

  /**
   * Получить текущее использование памяти
   */
  async getMemoryUsage(): Promise<number> {
    try {
      return await invoke<number>('get_memory_usage');
    } catch (error) {
      console.error('Failed to get memory usage:', error);
      throw error;
    }
  }

  /**
   * Получить метрики запуска приложения
   */
  async getStartupMetrics(): Promise<StartupMetrics | null> {
    try {
      const metrics = await invoke<StartupMetrics | null>('get_startup_metrics');
      if (metrics) {
        this.startupMetrics = metrics;
      }
      return metrics;
    } catch (error) {
      console.error('Failed to get startup metrics:', error);
      throw error;
    }
  }

  /**
   * Очистить все метрики производительности
   */
  async clearMetrics(): Promise<void> {
    try {
      await invoke('clear_performance_metrics');
      this.lastModelLoadMetrics = null;
      this.lastInferenceMetrics = null;
      this.inferenceHistory = [];
      // Не очищаем startup metrics, так как они задаются один раз
    } catch (error) {
      console.error('Failed to clear metrics:', error);
      throw error;
    }
  }

  /**
   * Получить сводку производительности
   */
  async getPerformanceSummary(): Promise<PerformanceSummary> {
    const currentMemory = await this.getMemoryUsage();

    // Загружаем startup metrics если еще не загружены
    if (!this.startupMetrics) {
      await this.getStartupMetrics();
    }

    const averageTokensPerSecond =
      this.inferenceHistory.length > 0
        ? this.inferenceHistory.reduce((sum, m) => sum + m.tokens_per_second, 0) /
          this.inferenceHistory.length
        : 0;

    const totalGeneratedTokens = this.inferenceHistory.reduce(
      (sum, m) => sum + m.generated_tokens,
      0,
    );

    return {
      current_memory_mb: currentMemory,
      last_model_load: this.lastModelLoadMetrics || undefined,
      last_inference: this.lastInferenceMetrics || undefined,
      startup: this.startupMetrics || undefined,
      average_tokens_per_second: averageTokensPerSecond,
      total_generated_tokens: totalGeneratedTokens,
    };
  }

  /**
   * Подписаться на события метрик
   */
  async setupEventListeners(
    onModelLoad?: (metrics: ModelLoadMetrics) => void,
    onInference?: (metrics: InferenceMetrics) => void,
    onStartup?: (metrics: StartupMetrics) => void,
  ): Promise<void> {
    // Слушаем метрики загрузки модели
    const modelLoadListener = await listen<ModelLoadMetrics>('model_load_metrics', (event) => {
      console.log('Model load metrics:', event.payload);
      this.lastModelLoadMetrics = event.payload;
      onModelLoad?.(event.payload);
    });

    // Слушаем метрики inference
    const inferenceListener = await listen<InferenceMetrics>('inference_metrics', (event) => {
      this.lastInferenceMetrics = event.payload;
      this.inferenceHistory.push(event.payload);

      // Ограничиваем историю 100 последними записями
      if (this.inferenceHistory.length > 100) {
        this.inferenceHistory.shift();
      }

      onInference?.(event.payload);
    });

    // Слушаем метрики запуска
    const startupListener = await listen<StartupMetrics>('startup_metrics', (event) => {
      console.log('✅ Startup metrics received:', event.payload);
      this.startupMetrics = event.payload;
      onStartup?.(event.payload);
    });

    this.listeners = [modelLoadListener, inferenceListener, startupListener];
  }

  /**
   * Отписаться от всех событий
   */
  cleanup(): void {
    this.listeners.forEach((unlisten) => unlisten());
    this.listeners = [];
  }

  /**
   * Форматировать длительность в человекочитаемый вид
   */
  formatDuration(ms: number): string {
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    } else if (ms < 60000) {
      return `${(ms / 1000).toFixed(2)}s`;
    } else {
      const minutes = Math.floor(ms / 60000);
      const seconds = ((ms % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  }

  /**
   * Форматировать размер памяти
   */
  formatMemory(mb: number): string {
    if (mb < 1024) {
      return `${mb.toFixed(2)} MB`;
    } else {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
  }

  /**
   * Форматировать скорость (токены/сек)
   */
  formatSpeed(tokensPerSecond: number): string {
    return `${tokensPerSecond.toFixed(2)} t/s`;
  }

  /**
   * Получить историю inference метрик
   */
  getInferenceHistory(): InferenceMetrics[] {
    return [...this.inferenceHistory];
  }

  /**
   * Получить последние метрики загрузки модели
   */
  getLastModelLoadMetrics(): ModelLoadMetrics | null {
    return this.lastModelLoadMetrics;
  }

  /**
   * Получить последние метрики inference
   */
  getLastInferenceMetrics(): InferenceMetrics | null {
    return this.lastInferenceMetrics;
  }

  /**
   * Получить метрики запуска (кэшированные)
   */
  getCachedStartupMetrics(): StartupMetrics | null {
    return this.startupMetrics;
  }

  /**
   * Получить текущее использование системных ресурсов (CPU, GPU, память)
   */
  async getSystemUsage(): Promise<SystemUsage> {
    try {
      return await invoke<SystemUsage>('get_system_usage');
    } catch (error) {
      console.error('Failed to get system usage:', error);
      throw error;
    }
  }
}

// Singleton instance
export const performanceService = new PerformanceService();
