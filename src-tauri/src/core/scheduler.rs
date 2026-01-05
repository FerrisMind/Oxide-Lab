//! Model Scheduler - управление жизненным циклом загруженных моделей
//!
//! Scheduler обеспечивает:
//! - Keep-alive таймеры для автоматической выгрузки неактивных моделей
//! - Управление памятью через ограничение количества загруженных моделей
//! - Переиспользование загруженных моделей между запросами

use std::time::{Duration, Instant};

/// Конфигурация планировщика моделей
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Время бездействия перед автоматической выгрузкой модели
    pub keep_alive: Duration,
    /// Максимальное количество одновременно загруженных моделей
    pub max_loaded_models: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            keep_alive: Duration::from_secs(5 * 60), // 5 минут
            max_loaded_models: 1,
        }
    }
}

impl SchedulerConfig {
    /// Создаёт конфигурацию из переменных окружения (плейсхолдер для будущего)
    pub fn from_env() -> Self {
        // TODO: OXIDE_KEEP_ALIVE, OXIDE_MAX_LOADED_MODELS
        Self::default()
    }

    /// Конвертирует keep_alive из секунд
    pub fn with_keep_alive_secs(mut self, secs: u64) -> Self {
        self.keep_alive = Duration::from_secs(secs);
        self
    }
}

use crate::models::ModelBackend;
use std::fmt;

/// Информация о загруженной модели с таймингом
pub struct LoadedModelEntry {
    /// Сама модель
    pub model: Box<dyn ModelBackend + Send>,
    /// Время последнего использования модели
    pub last_used: Instant,
    /// Идентификатор модели (путь или repo_id)
    pub model_id: String,
}

impl fmt::Debug for LoadedModelEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedModelEntry")
            .field("last_used", &self.last_used)
            .field("model_id", &self.model_id)
            .finish_non_exhaustive()
    }
}

impl LoadedModelEntry {
    pub fn new(model: Box<dyn ModelBackend + Send>, model_id: String) -> Self {
        Self {
            model,
            last_used: Instant::now(),
            model_id,
        }
    }

    /// Обновляет время последнего использования
    pub fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Проверяет, истёк ли keep-alive таймер
    pub fn is_expired(&self, keep_alive: Duration) -> bool {
        self.last_used.elapsed() > keep_alive
    }
}

/// Планировщик моделей
pub struct ModelScheduler {
    /// Активная загруженная модель
    pub active_model: Option<LoadedModelEntry>,
    /// Конфигурация
    pub config: SchedulerConfig,
}

impl ModelScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            active_model: None,
            config,
        }
    }

    /// Загружает новую модель, вытесняя старую
    pub fn load_model(&mut self, model: Box<dyn ModelBackend + Send>, id: String) {
        if let Some(old) = &self.active_model {
            log::info!(
                "ModelScheduler: Unloading previous model '{}' to load '{}'",
                old.model_id,
                id
            );
        } else {
            log::info!("ModelScheduler: Loading new model '{}'", id);
        }
        self.active_model = Some(LoadedModelEntry::new(model, id));
    }

    /// Выгружает текущую модель
    pub fn unload_model(&mut self) {
        if let Some(old) = &self.active_model {
            log::info!("ModelScheduler: Unloading model '{}'", old.model_id);
        }
        self.active_model = None;
    }

    /// Забирает модель для использования (например, для генерации в отдельном потоке).
    /// Вызывающая сторона ОБЯЗАНА вернуть модель через `restore_model`.
    pub fn take_model(&mut self) -> Option<LoadedModelEntry> {
        self.active_model.take()
    }

    /// Возвращает модель после использования
    pub fn restore_model(&mut self, mut entry: LoadedModelEntry) {
        entry.touch();
        self.active_model = Some(entry);
    }

    /// Проверяет таймауты и выгружает просроченные модели
    #[allow(clippy::collapsible_if)]
    pub fn check_expiration(&mut self) {
        if let Some(entry) = &self.active_model {
            if entry.is_expired(self.config.keep_alive) {
                log::info!(
                    "ModelScheduler: Keep-alive expired for '{}' ({:?}), unloading...",
                    entry.model_id,
                    self.config.keep_alive
                );
                self.active_model = None;
            }
        }
    }

    pub fn has_model(&self) -> bool {
        self.active_model.is_some()
    }

    pub fn get_model_id(&self) -> Option<String> {
        self.active_model.as_ref().map(|e| e.model_id.clone())
    }
}
