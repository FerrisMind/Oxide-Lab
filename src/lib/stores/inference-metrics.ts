// Store для хранения метрик inference по индексам сообщений
import { writable } from 'svelte/store';
import type { InferenceMetrics } from '$lib/types/performance';

type InferenceMetricsMap = Map<number, InferenceMetrics>;

function createInferenceMetricsStore() {
  const { subscribe, set, update } = writable<InferenceMetricsMap>(new Map());

  return {
    subscribe,
    set,
    update,

    // Добавить метрики для конкретного индекса сообщения
    setMetrics(messageIndex: number, metrics: InferenceMetrics) {
      update((map) => {
        // ВАЖНО: Создаём новый Map для триггера реактивности Svelte!
        const newMap = new Map(map);
        newMap.set(messageIndex, metrics);
        return newMap;
      });
    },

    // Получить метрики для конкретного индекса
    getMetrics(messageIndex: number, currentMap: InferenceMetricsMap): InferenceMetrics | null {
      return currentMap.get(messageIndex) || null;
    },

    // Очистить все метрики
    clear() {
      set(new Map());
    },

    // Удалить метрики для конкретного индекса
    removeMetrics(messageIndex: number) {
      update((map) => {
        map.delete(messageIndex);
        return map;
      });
    },
  };
}

export const inferenceMetricsStore = createInferenceMetricsStore();
