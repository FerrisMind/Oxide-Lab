# Отображение метрик производительности в UI

## Обзор

Метрики производительности автоматически отображаются под каждым ответом модели в чате. Пользователи могут видеть:

- ⚡ **Скорость генерации** (токены/сек) - основная метрика
- 📊 **Количество токенов** (в расширенном режиме)
- ⏱️ **Время генерации** (в расширенном режиме)
- 💾 **Использование памяти** (в расширенном режиме)

## Архитектура отображения

### 1. Компонент метрик

**`src/lib/chat/components/InferenceMetricsDisplay.svelte`**

Компактный компонент для отображения метрик под ответом модели.

```svelte
<InferenceMetricsDisplay {metrics} compact={true} />
```

**Props:**

- `metrics: InferenceMetrics | null` - метрики inference
- `compact: boolean` - компактный режим (только скорость)

**Компактный режим:**

```
⚡ 45.23 t/s
```

**Расширенный режим:**

```
⚡ 45.23 t/s | Токены: 150 | Время: 3.5s | Память: 2.3 GB
```

### 2. Store для метрик

**`src/lib/stores/inference-metrics.ts`**

Хранит метрики для каждого сообщения по индексу:

```typescript
import { inferenceMetricsStore } from '$lib/stores/inference-metrics';

// Добавить метрики для сообщения
inferenceMetricsStore.setMetrics(messageIndex, metrics);

// Получить метрики сообщения
const metrics = inferenceMetricsStore.getMetrics(messageIndex, metricsMap);

// Очистить все метрики
inferenceMetricsStore.clear();
```

### 3. Интеграция в MessageList

**`src/lib/chat/components/MessageList.svelte`**

Автоматически отображает метрики под ответами ассистента:

```svelte
{#if m.role === 'assistant'}
  {@const metrics = metricsMap.get(i)}
  {#if metrics}
    <InferenceMetricsDisplay {metrics} compact={true} />
  {/if}
{/if}
```

### 4. Подписка на события в Chat.svelte

**`src/lib/chat/Chat.svelte`**

При монтировании компонента подписываемся на события метрик:

```typescript
onMount(async () => {
  // ... другой код ...

  // Подписываемся на события метрик производительности
  await performanceService.setupEventListeners(
    undefined, // Не обрабатываем загрузку модели здесь
    (inferenceMetrics: InferenceMetrics) => {
      // Получаем индекс последнего ассистентского сообщения
      const lastAssistantIndex = messages.findLastIndex((m) => m.role === 'assistant');
      if (lastAssistantIndex !== -1) {
        inferenceMetricsStore.setMetrics(lastAssistantIndex, inferenceMetrics);
      }
    },
  );
});

onDestroy(() => {
  performanceService.cleanup();
});
```

## Поток данных

```
Backend (Rust)                 Frontend (TypeScript/Svelte)
──────────────                 ────────────────────────────

generate_stream()
      │
      ├─→ inference_tracker.finish()
      │        │
      │        └─→ monitor.record_metric()
      │
      └─→ app.emit("inference_metrics", metrics)
                   │
                   │   [Tauri Event]
                   │
                   └─→ performanceService.setupEventListeners()
                              │
                              └─→ onInference callback
                                        │
                                        └─→ inferenceMetricsStore.setMetrics()
                                                   │
                                                   │   [Svelte Store Update]
                                                   │
                                                   └─→ MessageList rerenders
                                                              │
                                                              └─→ InferenceMetricsDisplay shows data
                                                                            │
                                                                            └─→ ⚡ 45.23 t/s
```

## Стилизация

Метрики стилизованы в соответствии с темной темой чата:

```css
.inference-metrics {
  margin-top: 0.5rem;
  padding: 0.35rem 0.6rem;
  background: rgba(0, 0, 0, 0.15);
  border-radius: 6px;
  font-size: 0.85rem;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.metric-item.primary {
  color: var(--accent-color, #4a9eff);
  font-weight: 600;
}
```

**Адаптивность:**

- Автоматически подстраивается под ширину экрана
- Flex-обёртка для переноса метрик на новую строку
- Компактный режим для мобильных устройств

## Примеры использования

### Базовое отображение (автоматическое)

После настройки события автоматически обрабатываются, и метрики появляются под каждым ответом.

```
User: Расскажи про квантовую физику

Assistant: [Длинный ответ про квантовую физику...]

⚡ 42.5 t/s
```

### Расширенный режим

Для отображения всех метрик установите `compact={false}`:

```svelte
<InferenceMetricsDisplay {metrics} compact={false} />
```

Результат:

```
⚡ 42.5 t/s | Токены: 150 | Время: 3.5s | Память: 2.3 GB
```

### Программное получение метрик

```typescript
import { inferenceMetricsStore } from '$lib/stores/inference-metrics';
import { get } from 'svelte/store';

// Получить все метрики
const allMetrics = get(inferenceMetricsStore);

// Получить метрики для конкретного сообщения
const messageMetrics = allMetrics.get(messageIndex);

if (messageMetrics) {
  console.log('Скорость:', messageMetrics.tokens_per_second, 't/s');
  console.log('Токены:', messageMetrics.generated_tokens);
  console.log('Время:', messageMetrics.total_duration_ms, 'ms');
}
```

## Кастомизация

### Изменение стилей

Можно переопределить CSS-переменные:

```css
:global(.inference-metrics) {
  --accent-color: #ff6b6b; /* Красный цвет для скорости */
  --text-secondary: #ccc; /* Цвет дополнительных метрик */
}
```

### Создание собственного компонента

```svelte
<script lang="ts">
  import type { InferenceMetrics } from '$lib/types/performance';
  import { performanceService } from '$lib/services/performance-service';

  let { metrics }: { metrics: InferenceMetrics | null } = $props();
</script>

{#if metrics}
  <div class="custom-metrics">
    <!-- Только скорость, большой размер -->
    <h3>{performanceService.formatSpeed(metrics.tokens_per_second)}</h3>
  </div>
{/if}
```

## Отладка

### Проверка получения событий

```typescript
// В Chat.svelte или любом компоненте
await performanceService.setupEventListeners(undefined, (metrics) => {
  console.log('Received inference metrics:', metrics);
  // Метрики должны появляться после каждого ответа
});
```

### Проверка store

```typescript
import { inferenceMetricsStore } from '$lib/stores/inference-metrics';

inferenceMetricsStore.subscribe((metricsMap) => {
  console.log('Metrics store updated:', metricsMap.size, 'entries');
  metricsMap.forEach((metrics, index) => {
    console.log(`Message ${index}:`, metrics);
  });
});
```

### Если метрики не отображаются

**1. Проверьте backend:**

```rust
// В src-tauri/src/generate/stream.rs должно быть:
app.emit("inference_metrics", &metrics);
```

**2. Проверьте события Tauri:**

```typescript
import { listen } from '@tauri-apps/api/event';

// Временно добавьте прямой слушатель
const unlisten = await listen('inference_metrics', (event) => {
  console.log('Raw event:', event.payload);
});
```

**3. Проверьте индексы сообщений:**

```typescript
const lastAssistantIndex = messages.findLastIndex((m) => m.role === 'assistant');
console.log('Last assistant message index:', lastAssistantIndex);
```

## Доступность

Метрики производительности доступны для всех пользователей:

- ✅ Семантическая разметка
- ✅ Достаточный контраст (цвета соответствуют WCAG AA)
- ✅ Читаемые размеры шрифтов
- ✅ Не мешают чтению основного контента

## Производительность

- **Минимальный overhead**: компонент рендерится только при наличии метрик
- **Эффективное обновление**: используются Svelte 5 руны для оптимальной реактивности
- **Нет утечек памяти**: слушатели очищаются в `onDestroy`
- **Оптимизированные стили**: используются CSS custom properties

## Будущие улучшения

- [ ] Анимированное появление метрик
- [ ] График скорости в реальном времени
- [ ] Сравнение метрик разных ответов
- [ ] Экспорт метрик в CSV
- [ ] Настройки отображения (показывать/скрывать метрики)
- [ ] Tooltip с расширенной информацией при наведении
- [ ] История метрик в отдельной панели
