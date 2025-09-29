# Очистка debug кода после тестирования

После успешного тестирования отображения метрик нужно убрать временный debug код.

## Что нужно удалить:

### 1. В `src/lib/chat/components/InferenceMetricsDisplay.svelte`

**Удалить:**

- Все `console.log` в скрипте
- Красный DEBUG блок (строки 21-27)
- Оранжевый else блок (строки 55-59)
- Зелёную подсветку в style (убрать `style="background: #00ff0020; border: 2px solid lime;"`)

**Финальный вид:**

```svelte
<script lang="ts">
  import type { InferenceMetrics } from '$lib/types/performance';
  import { performanceService } from '$lib/services/performance-service';

  export let metrics: InferenceMetrics | null = null;
  export let compact: boolean = true;
</script>

{#if metrics}
  <div class="inference-metrics" class:compact>
    <div class="metrics-row">
      <div class="metric-item primary">
        <span class="icon">⚡</span>
        <span class="value">{performanceService.formatSpeed(metrics.tokens_per_second)}</span>
      </div>

      {#if !compact}
        <div class="metric-item">
          <span class="label">Токены:</span>
          <span class="value">{metrics.generated_tokens}</span>
        </div>

        <div class="metric-item">
          <span class="label">Время:</span>
          <span class="value">{performanceService.formatDuration(metrics.total_duration_ms)}</span>
        </div>

        <div class="metric-item">
          <span class="label">Память:</span>
          <span class="value">{performanceService.formatMemory(metrics.memory_usage_mb)}</span>
        </div>
      {/if}
    </div>
  </div>
{/if}
```

### 2. В `src/lib/chat/components/MessageList.svelte`

**Удалить:**

- Все `console.log` в $effect
- Debug блок "No metrics for message X" (строки 72-76)

**Финальный вид:**

```svelte
<script lang="ts">
  import type { ChatMessage } from "$lib/chat/types";
  import { registerAssistantBubble } from "$lib/chat/stream_render";
  import ChatPlaceholder from "./ChatPlaceholder.svelte";
  import InferenceMetricsDisplay from "./InferenceMetricsDisplay.svelte";
  import { inferenceMetricsStore } from "$lib/stores/inference-metrics";

  let {
    messages = $bindable([]),
    messagesEl = $bindable(null),
    showModelNotice = false
  }: {
    messages?: ChatMessage[];
    messagesEl?: HTMLDivElement | null;
    showModelNotice?: boolean;
  } = $props();

  const baseBackground = '#2d2d2d';
  let placeholderOnly = $derived(showModelNotice && messages.length === 0);

  let metricsMap = $state(new Map());

  $effect(() => {
    const unsubscribe = inferenceMetricsStore.subscribe(value => {
      metricsMap = value;
    });
    return unsubscribe;
  });
</script>

<!-- ... rest of template ... -->

{#if m.role === 'assistant'}
  {@const metrics = metricsMap.get(i)}
  {#if metrics}
    <InferenceMetricsDisplay {metrics} compact={true} />
  {/if}
{/if}
```

### 3. В `src/lib/chat/Chat.svelte`

**Удалить** все `console.log`:

```typescript
await performanceService.setupEventListeners(undefined, (inferenceMetrics: InferenceMetrics) => {
  setTimeout(() => {
    const lastAssistantIndex = messages.findLastIndex((m) => m.role === 'assistant');
    if (lastAssistantIndex !== -1) {
      inferenceMetricsStore.setMetrics(lastAssistantIndex, inferenceMetrics);
    }
  }, 150);
});
```

### 4. В `src/lib/services/performance-service.ts`

**Удалить** console.log из setupEventListeners:

```typescript
const inferenceListener = await listen<InferenceMetrics>('inference_metrics', (event) => {
  this.lastInferenceMetrics = event.payload;
  this.inferenceHistory.push(event.payload);

  if (this.inferenceHistory.length > 100) {
    this.inferenceHistory.shift();
  }

  onInference?.(event.payload);
});
```

### 5. В `src/lib/stores/inference-metrics.ts`

**Удалить** все console.log из setMetrics:

```typescript
setMetrics(messageIndex: number, metrics: InferenceMetrics) {
  update((map) => {
    const newMap = new Map(map);
    newMap.set(messageIndex, metrics);
    return newMap;
  });
},
```

## Команда для финальной проверки:

```bash
npm run check
npm run build
```

Если всё компилируется без ошибок и метрики отображаются - готово! 🎉
