# Отладка отображения метрик в UI

## Проблема

Метрики производительности видны в консоли разработчика, но не отображаются в UI под ответами модели.

## Добавленное логирование

Я добавил детальное логирование на каждом этапе потока данных:

### 1. Backend → Frontend (Tauri Events)

**В `performance-service.ts`:**

```
[PerformanceService] Received inference_metrics event: {...}
[PerformanceService] Calling onInference callback
```

### 2. Chat Component (получение события)

**В `Chat.svelte`:**

```
[Chat] Received inference metrics: {...}
[Chat] Current messages: [...]
[Chat] Last assistant index: X
[Chat] Setting metrics for message X
```

### 3. Store (сохранение метрик)

**В `inference-metrics.ts`:**

```
[InferenceMetricsStore] setMetrics called: {messageIndex: X, metrics: {...}}
[InferenceMetricsStore] Current map size before: 0
[InferenceMetricsStore] Current map size after: 1
[InferenceMetricsStore] Map contents: [[X, {...}]]
```

### 4. MessageList Component (рендеринг)

**В `MessageList.svelte`:**

```
[MessageList] Metrics store updated, size: 1
[MessageList] Metrics map: Map { 0 => {...} }
[MessageList] Messages count: 2
[MessageList] Metrics map size: 1
```

### 5. InferenceMetricsDisplay Component

**В `InferenceMetricsDisplay.svelte`:**

```
[InferenceMetricsDisplay] Mounted with metrics: {...}
[InferenceMetricsDisplay] Metrics updated: {...}
```

## Как использовать логирование

### Шаг 1: Запустите приложение

```bash
npm run tauri dev
```

### Шаг 2: Откройте DevTools

- Нажмите `F12` или `Ctrl+Shift+I`
- Перейдите на вкладку "Console"

### Шаг 3: Загрузите модель и отправьте сообщение

1. Загрузите GGUF модель
2. Отправьте любое сообщение
3. Дождитесь ответа модели

### Шаг 4: Проверьте логи

Ищите последовательность логов в консоли:

#### ✅ Ожидаемая последовательность (если всё работает):

```
1. [PerformanceService] Received inference_metrics event: {...}
2. [PerformanceService] Calling onInference callback
3. [Chat] Received inference metrics: {...}
4. [Chat] Current messages: [...]
5. [Chat] Last assistant index: 1 (или другой индекс)
6. [Chat] Setting metrics for message 1
7. [InferenceMetricsStore] setMetrics called: {...}
8. [InferenceMetricsStore] Current map size after: 1
9. [MessageList] Metrics store updated, size: 1
10. [InferenceMetricsDisplay] Mounted with metrics: {...}
```

После этого под ответом модели должна появиться метрика: `⚡ XX.X t/s`

#### ❌ Диагностика проблем:

**Проблема 1: События не приходят**

Если вы НЕ видите лог:

```
[PerformanceService] Received inference_metrics event
```

**Решение:**

- Проверьте backend: `src-tauri/src/generate/stream.rs`
- Убедитесь, что есть строка: `app.emit("inference_metrics", &metrics)?;`
- Проверьте логи Rust в терминале: должны быть `[INFER]` сообщения

**Проблема 2: События приходят, но индекс не найден**

Если вы видите:

```
[Chat] Last assistant index: -1
[Chat] No assistant message found to attach metrics
```

**Решение:**

- Метрики приходят ДО того, как сообщение добавлено в `messages`
- Возможно timing issue - событие метрик приходит раньше, чем сообщение добавляется

**Проблема 3: Store обновляется, но UI не рендерит**

Если вы видите:

```
[InferenceMetricsStore] Current map size after: 1
[MessageList] Metrics store updated, size: 1
```

Но НЕ видите:

```
[InferenceMetricsDisplay] Mounted with metrics
```

**Решение:**

- Возможно проблема с реактивностью Svelte
- Map не триггерит обновление - нужно создавать новый Map

## Возможные исправления

### Исправление 1: Проблема с Map реактивностью

Svelte может не отслеживать изменения в Map. Исправим store:

```typescript
// В inference-metrics.ts
setMetrics(messageIndex: number, metrics: InferenceMetrics) {
  update((map) => {
    const newMap = new Map(map); // Создаём новый Map!
    newMap.set(messageIndex, metrics);
    return newMap;
  });
},
```

### Исправление 2: Timing issue (метрики приходят раньше сообщения)

Добавим задержку или проверку:

```typescript
// В Chat.svelte
(inferenceMetrics: InferenceMetrics) => {
  // Даём время сообщению добавиться
  setTimeout(() => {
    const lastAssistantIndex = messages.findLastIndex((m) => m.role === 'assistant');
    if (lastAssistantIndex !== -1) {
      inferenceMetricsStore.setMetrics(lastAssistantIndex, inferenceMetrics);
    }
  }, 100);
};
```

### Исправление 3: Явное обновление компонента

В `MessageList.svelte`:

```typescript
let metricsMap = $state(new Map());

$effect(() => {
  const unsubscribe = inferenceMetricsStore.subscribe((value) => {
    // Создаём новую ссылку для Svelte реактивности
    metricsMap = new Map(value);
  });
  return unsubscribe;
});
```

## Временный visual debug

В UI сейчас добавлен временный debug блок. Под каждым ответом ассистента вы увидите:

- Если метрики ЕСТЬ: `⚡ XX.X t/s`
- Если метрик НЕТ: `[Debug] No metrics for message X`

Это поможет понять, доходят ли метрики до компонента.

## Следующие шаги

1. **Запустите приложение** и проверьте логи
2. **Отправьте сообщение** и дождитесь ответа
3. **Сфотографируйте консоль** с логами
4. **Проверьте UI** - видите ли вы `[Debug]` сообщение или метрики

Если вы видите `[Debug] No metrics for message X`, то проблема в том, что:

- События либо не приходят
- Либо приходят с неправильным индексом
- Либо Map не обновляется реактивно

Логи в консоли покажут, на каком именно этапе происходит сбой.
