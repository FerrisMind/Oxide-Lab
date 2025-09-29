// Модуль для мониторинга производительности
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use sysinfo::{System, Pid};

/// Метрики производительности для одной операции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub operation_name: String,
    pub duration_ms: u64,
    pub timestamp: String,
    pub memory_usage_mb: f64,
    pub additional_data: Option<serde_json::Value>,
}

/// Метрики загрузки модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadMetrics {
    pub total_duration_ms: u64,
    pub stages: Vec<LoadStage>,
    pub model_size_mb: f64,
    pub memory_before_mb: f64,
    pub memory_after_mb: f64,
    pub memory_delta_mb: f64,
}

/// Стадия загрузки модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStage {
    pub name: String,
    pub duration_ms: u64,
}

/// Метрики inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_duration_ms: u64,
    pub prefill_duration_ms: u64,
    pub generation_duration_ms: u64,
    pub tokens_per_second: f64,
    pub prefill_tokens_per_second: f64,
    pub memory_usage_mb: f64,
    pub timestamp: String,
}

/// Монитор производительности
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<PerformanceMetric>>>,
    max_entries: usize,
    system: Arc<RwLock<System>>,
}

impl PerformanceMonitor {
    pub fn new(max_entries: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            max_entries,
            system: Arc::new(RwLock::new(system)),
        }
    }

    /// Записать метрику
    pub async fn record_metric(&self, metric: PerformanceMetric) {
        let mut metrics = self.metrics.write().await;

        // Ограничиваем количество записей
        if metrics.len() >= self.max_entries {
            metrics.remove(0);
        }

        metrics.push(metric);
    }

    /// Получить все метрики
    pub async fn get_metrics(&self) -> Vec<PerformanceMetric> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Получить среднюю длительность операции
    pub async fn get_average_duration(&self, operation_name: &str) -> Option<f64> {
        let metrics = self.metrics.read().await;
        let operation_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.operation_name == operation_name)
            .collect();

        if operation_metrics.is_empty() {
            return None;
        }

        let total: u64 = operation_metrics.iter().map(|m| m.duration_ms).sum();
        Some(total as f64 / operation_metrics.len() as f64)
    }

    /// Получить текущее использование памяти процессом
    pub async fn get_memory_usage_mb(&self) -> f64 {
        let mut system = self.system.write().await;
        system.refresh_all();

        let pid = Pid::from_u32(std::process::id());
        if let Some(process) = system.process(pid) {
            // Память в байтах
            let memory_bytes = process.memory();
            // Конвертируем в мегабайты
            (memory_bytes as f64) / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }

    /// Очистить все метрики
    pub async fn clear_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

/// Таймер для измерения производительности
pub struct PerformanceTimer {
    start: Instant,
    operation_name: String,
    monitor: Option<Arc<PerformanceMonitor>>,
}

impl PerformanceTimer {
    /// Создать новый таймер
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation_name: operation_name.into(),
            monitor: None,
        }
    }

    /// Создать таймер с мониторингом
    pub fn with_monitor(
        operation_name: impl Into<String>,
        monitor: Arc<PerformanceMonitor>,
    ) -> Self {
        Self {
            start: Instant::now(),
            operation_name: operation_name.into(),
            monitor: Some(monitor),
        }
    }

    /// Получить длительность с момента создания таймера
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Получить длительность в миллисекундах
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Завершить измерение и записать метрику
    pub async fn finish(self) -> u64 {
        let duration_ms = self.elapsed_ms();

        if let Some(monitor) = self.monitor {
            let memory_usage = monitor.get_memory_usage_mb().await;
            let metric = PerformanceMetric {
                operation_name: self.operation_name,
                duration_ms,
                timestamp: chrono::Utc::now().to_rfc3339(),
                memory_usage_mb: memory_usage,
                additional_data: None,
            };

            monitor.record_metric(metric).await;
        }

        duration_ms
    }

    /// Завершить измерение с дополнительными данными
    pub async fn finish_with_data(self, data: serde_json::Value) -> u64 {
        let duration_ms = self.elapsed_ms();

        if let Some(monitor) = self.monitor {
            let memory_usage = monitor.get_memory_usage_mb().await;
            let metric = PerformanceMetric {
                operation_name: self.operation_name,
                duration_ms,
                timestamp: chrono::Utc::now().to_rfc3339(),
                memory_usage_mb: memory_usage,
                additional_data: Some(data),
            };

            monitor.record_metric(metric).await;
        }

        duration_ms
    }
}

/// Трекер загрузки модели
pub struct ModelLoadTracker {
    start: Instant,
    stages: Vec<(String, Instant)>,
    memory_before_mb: f64,
    monitor: Arc<PerformanceMonitor>,
}

impl ModelLoadTracker {
    /// Создать новый трекер загрузки
    pub async fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        let memory_before = monitor.get_memory_usage_mb().await;

        Self {
            start: Instant::now(),
            stages: Vec::new(),
            memory_before_mb: memory_before,
            monitor,
        }
    }

    /// Начать новую стадию загрузки
    pub fn start_stage(&mut self, stage_name: impl Into<String>) {
        self.stages.push((stage_name.into(), Instant::now()));
    }

    /// Завершить трекинг и вернуть метрики
    pub async fn finish(self, model_size_mb: f64) -> ModelLoadMetrics {
        let total_duration_ms = self.start.elapsed().as_millis() as u64;
        let memory_after_mb = self.monitor.get_memory_usage_mb().await;
        let memory_delta_mb = memory_after_mb - self.memory_before_mb;

        let mut load_stages = Vec::new();
        let mut prev_time = self.start;

        for (stage_name, _stage_start) in self.stages {
            let stage_duration = prev_time.elapsed().as_millis() as u64;
            load_stages.push(LoadStage {
                name: stage_name,
                duration_ms: stage_duration,
            });
            prev_time = Instant::now();
        }

        ModelLoadMetrics {
            total_duration_ms,
            stages: load_stages,
            model_size_mb,
            memory_before_mb: self.memory_before_mb,
            memory_after_mb,
            memory_delta_mb,
        }
    }
}

/// Трекер inference
pub struct InferenceTracker {
    start: Instant,
    prefill_start: Option<Instant>,
    generation_start: Option<Instant>,
    prompt_tokens: usize,
    generated_tokens: usize,
    monitor: Arc<PerformanceMonitor>,
}

impl InferenceTracker {
    /// Создать новый трекер inference
    pub fn new(prompt_tokens: usize, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            prefill_start: None,
            generation_start: None,
            prompt_tokens,
            generated_tokens: 0,
            monitor,
        }
    }

    /// Отметить начало prefill
    pub fn start_prefill(&mut self) {
        self.prefill_start = Some(Instant::now());
    }

    /// Отметить начало generation
    pub fn start_generation(&mut self) {
        self.generation_start = Some(Instant::now());
    }

    /// Увеличить счётчик сгенерированных токенов
    pub fn increment_generated_tokens(&mut self) {
        self.generated_tokens += 1;
    }

    /// Завершить трекинг и вернуть метрики
    pub async fn finish(self) -> InferenceMetrics {
        let total_duration_ms = self.start.elapsed().as_millis() as u64;

        let prefill_duration_ms = if let Some(prefill_start) = self.prefill_start {
            if let Some(generation_start) = self.generation_start {
                (generation_start - prefill_start).as_millis() as u64
            } else {
                0
            }
        } else {
            0
        };

        let generation_duration_ms = if let Some(generation_start) = self.generation_start {
            generation_start.elapsed().as_millis() as u64
        } else {
            total_duration_ms
        };

        let tokens_per_second = if generation_duration_ms > 0 {
            (self.generated_tokens as f64) / (generation_duration_ms as f64 / 1000.0)
        } else {
            0.0
        };

        let prefill_tokens_per_second = if prefill_duration_ms > 0 {
            (self.prompt_tokens as f64) / (prefill_duration_ms as f64 / 1000.0)
        } else {
            0.0
        };

        let memory_usage_mb = self.monitor.get_memory_usage_mb().await;

        InferenceMetrics {
            prompt_tokens: self.prompt_tokens,
            generated_tokens: self.generated_tokens,
            total_duration_ms,
            prefill_duration_ms,
            generation_duration_ms,
            tokens_per_second,
            prefill_tokens_per_second,
            memory_usage_mb,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Макрос для измерения производительности блока кода
#[macro_export]
macro_rules! measure_performance {
    ($monitor:expr, $operation_name:expr, $block:expr) => {{
        let timer = $crate::core::performance::PerformanceTimer::with_monitor(
            $operation_name,
            $monitor.clone(),
        );
        let result = $block;
        timer.finish().await;
        result
    }};
}
