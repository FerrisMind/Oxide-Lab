# Мониторинг производительности в Oxide-Lab

## Обзор

Полностью реализованная система мониторинга производительности для отслеживания:

- ⏱️ Времени загрузки моделей по стадиям
- ⚡ Скорости inference (токены/сек)
- 💾 Использования памяти в реальном времени

## Архитектура

### Backend (Rust)

#### 1. Модуль `core/performance.rs`

**PerformanceMonitor** - главный компонент для отслеживания метрик:

```rust
// Создание монитора (автоматически при инициализации приложения)
let monitor = PerformanceMonitor::new(1000); // Хранит до 1000 метрик
```

**Основные компоненты:**

- `PerformanceMetric` - отдельная метрика операции
- `ModelLoadMetrics` - метрики загрузки модели
- `InferenceMetrics` - метрики inference
- `PerformanceTimer` - таймер для измерений
- `ModelLoadTracker` - трекер загрузки модели
- `InferenceTracker` - трекер inference

#### 2. Интеграция с загрузкой моделей (`api/model_loading/gguf.rs`)

```rust
// Создаём трекер в начале загрузки
let mut tracker = ModelLoadTracker::new(guard.performance_monitor.clone()).await;

// Отмечаем стадии загрузки
tracker.start_stage("device_selection");
tracker.start_stage("file_opening");
tracker.start_stage("read_header");
tracker.start_stage("tokenizer_init");
tracker.start_stage("model_building");

// Финализируем с расчётом всех метрик
let metrics = tracker.finish(model_size_mb).await;

// Отправляем метрики на фронтенд
app.emit("model_load_metrics", &metrics);
```

**Отслеживаемые метрики загрузки:**

- Общее время загрузки
- Время каждой стадии (выбор устройства, открытие файла, чтение заголовка, инициализация токенизатора, построение модели)
- Размер модели в MB
- Использование памяти до/после загрузки
- Δ памяти (изменение)

#### 3. Интеграция с inference (`generate/stream.rs`)

```rust
// Создаём трекер для inference
let mut inference_tracker = InferenceTracker::new(
    effective_context_tokens.len(),
    guard.performance_monitor.clone()
);

// Отмечаем начало prefill
inference_tracker.start_prefill();

// Отмечаем начало generation
inference_tracker.start_generation();

// Увеличиваем счётчик при каждом сгенерированном токене
inference_tracker.increment_generated_tokens();

// Финализируем с расчётом метрик
let metrics = inference_tracker.finish().await;

// Отправляем на фронтенд
app.emit("inference_metrics", &metrics);
```

**Отслеживаемые метрики inference:**

- Количество prompt токенов
- Количество сгенерированных токенов
- Общее время
- Время prefill (обработка prompt)
- Время generation (генерация ответа)
- **Скорость: токены/сек** (основная метрика)
- Скорость prefill (токены/сек)
- Использование памяти

#### 4. API команды (`api/performance_api.rs`)

```rust
// Получить все метрики
#[tauri::command]
pub async fn get_performance_metrics() -> Result<Vec<PerformanceMetric>, String>

// Получить среднюю длительность операции
#[tauri::command]
pub async fn get_average_duration(operation_name: String) -> Result<Option<f64>, String>

// Получить текущее использование памяти
#[tauri::command]
pub async fn get_memory_usage() -> Result<f64, String>

// Очистить все метрики
#[tauri::command]
pub async fn clear_performance_metrics() -> Result<(), String>
```

### Frontend (TypeScript + Svelte)

#### 1. Типы (`lib/types/performance.ts`)

```typescript
interface PerformanceMetric {
  operation_name: string;
  duration_ms: number;
  timestamp: string;
  memory_usage_mb: number;
}

interface ModelLoadMetrics {
  total_duration_ms: number;
  stages: LoadStage[];
  model_size_mb: number;
  memory_delta_mb: number;
}

interface InferenceMetrics {
  prompt_tokens: number;
  generated_tokens: number;
  tokens_per_second: number; // Ключевая метрика!
  total_duration_ms: number;
  memory_usage_mb: number;
}
```

#### 2. Сервис (`lib/services/performance-service.ts`)

```typescript
import { performanceService } from '$lib/services/performance-service';

// Получить метрики
const metrics = await performanceService.getPerformanceMetrics();

// Получить текущую память
const memory = await performanceService.getMemoryUsage();

// Получить сводку
const summary = await performanceService.getPerformanceSummary();

// Подписаться на события
await performanceService.setupEventListeners(
  (modelLoad) => console.log('Model loaded:', modelLoad),
  (inference) => console.log('Inference:', inference.tokens_per_second, 't/s'),
);
```

**Утилиты форматирования:**

- `formatDuration(ms)` - "1.5s", "2m 30s"
- `formatMemory(mb)` - "512.5 MB", "2.3 GB"
- `formatSpeed(t/s)` - "45.23 t/s"

#### 3. Компонент (`lib/components/PerformanceMonitor.svelte`)

```svelte
<script>
  import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';
</script>

<PerformanceMonitor />
```

**Функции компонента:**

- 📊 Отображение всех ключевых метрик
- 🔄 Ручное и автоматическое обновление (каждые 2 сек)
- 📈 История inference с расчётом средней скорости
- 🗑️ Очистка метрик
- ⚡ Real-time обновление при получении событий

## События Tauri

### 1. `model_load_metrics`

Отправляется после завершения загрузки модели:

```typescript
listen<ModelLoadMetrics>('model_load_metrics', (event) => {
  console.log('Load time:', event.payload.total_duration_ms, 'ms');
  console.log('Memory delta:', event.payload.memory_delta_mb, 'MB');
});
```

### 2. `inference_metrics`

Отправляется после каждого inference:

```typescript
listen<InferenceMetrics>('inference_metrics', (event) => {
  console.log('Speed:', event.payload.tokens_per_second, 't/s');
  console.log('Tokens:', event.payload.generated_tokens);
});
```

## Использование памяти

Мониторинг использует библиотеку `sysinfo`:

```rust
// Получение текущего использования памяти процессом
let memory_mb = monitor.get_memory_usage_mb().await;

// Автоматическое отслеживание в метриках
// - При загрузке модели: memory_before, memory_after, memory_delta
// - При inference: текущее использование памяти
```

## Примеры логов

### Загрузка модели:

```
[LOAD] Метрики загрузки: total_time=3542ms, memory_delta=1024.56MB,
       stages=["device_selection:12ms", "file_opening:45ms",
               "read_header:234ms", "tokenizer_init:123ms",
               "model_building:3128ms"]
```

### Inference:

```
[INFER] Метрики inference: prompt_tokens=25, generated_tokens=150,
        total_time=3500ms, tokens/sec=42.86, memory=2048.32MB
```

## Интеграция в UI

### Вариант 1: Отдельная страница мониторинга

```svelte
<!-- src/routes/performance/+page.svelte -->
<script>
  import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';
</script>

<PerformanceMonitor />
```

### Вариант 2: Компактная панель в чате

```svelte
<script>
  let summary;

  onMount(async () => {
    summary = await performanceService.getPerformanceSummary();
  });
</script>

{#if summary?.last_inference}
  <div class="perf-badge">
    ⚡ {summary.last_inference.tokens_per_second.toFixed(1)} t/s
  </div>
{/if}
```

### Вариант 3: Встроенные метрики в Sidebar

```svelte
<div class="sidebar-footer">
  <div>💾 {formatMemory(currentMemory)}</div>
  {#if lastInference}
    <div>⚡ {formatSpeed(lastInference.tokens_per_second)}</div>
  {/if}
</div>
```

## Производительность системы мониторинга

- ✅ Minimal overhead: измерения не влияют на производительность inference
- ✅ Async операции: все метрики собираются асинхронно
- ✅ Ограничение памяти: хранится до 1000 метрик (настраивается)
- ✅ Автоматическая очистка истории (последние 100 inference)

## Будущие улучшения

- [ ] Графики производительности (Chart.js/D3.js)
- [ ] Экспорт метрик в CSV/JSON
- [ ] Сравнение разных моделей
- [ ] Мониторинг GPU (температура, utilization)
- [ ] Предупреждения при критичных значениях
- [ ] Профилирование по слоям модели

## Зависимости

**Rust:**

```toml
sysinfo = "0.33"         # Системная информация
chrono = "0.4"           # Временные метки
tokio = { version = "1", features = ["sync", "time"] } # Async
```

**TypeScript:**

```json
"@tauri-apps/api": "^2.0.0"  // Tauri API для invoke и events
```

## Troubleshooting

### Метрики не обновляются

```typescript
// Убедитесь, что слушатели настроены
await performanceService.setupEventListeners();

// Проверьте логи
console.log('Listening for performance events...');
```

### Большое потребление памяти

```rust
// Уменьшите лимит метрик при создании монитора
PerformanceMonitor::new(100) // вместо 1000
```

### Неточные измерения времени

```rust
// Убедитесь, что используется device.synchronize() для GPU
device.synchronize()?;
let duration = timer.elapsed();
```

## Заключение

Система мониторинга производительности полностью интегрирована и готова к использованию. Все ключевые метрики отслеживаются автоматически и доступны как через API, так и через события в реальном времени.
