use crate::core::performance::PerformanceMonitor;
use crate::core::precision::{Precision, PrecisionPolicy};
use crate::core::prefix_cache::{PrefixCache, PrefixCacheConfig};
use crate::core::scheduler::{ModelScheduler, SchedulerConfig};
use candle::Device;
use serde_json;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri::Manager;
use tokenizers::Tokenizer;

/// Универсальное состояние для любой модели
pub struct ModelState {
    pub(crate) scheduler: ModelScheduler,
    // gguf_file удалён, так как модель владеет ресурсами
    pub(crate) tokenizer: Option<Tokenizer>,
    pub(crate) device: Device,
    pub(crate) context_length: usize,
    pub(crate) model_path: Option<String>,
    pub(crate) tokenizer_path: Option<String>,
    pub(crate) model_config_json: Option<String>,
    /// Detected architecture kind
    pub(crate) arch: Option<crate::models::registry::ArchKind>,
    pub(crate) chat_template: Option<String>,
    // HF Hub (safetensors) связанные артефакты
    pub(crate) hub_repo_id: Option<String>,
    pub(crate) hub_revision: Option<String>,
    pub(crate) safetensors_files: Option<Vec<String>>,
    /// Precision policy for model loading
    pub(crate) precision_policy: PrecisionPolicy,
    /// Highest allowed Rayon thread count (None = automatic).
    pub(crate) rayon_thread_limit: Option<usize>,
    /// Performance monitor для отслеживания метрик
    pub(crate) performance_monitor: Arc<PerformanceMonitor>,
    /// Prefix Cache для переиспользования KV-кэшей
    pub(crate) prefix_cache: PrefixCache,
}

impl ModelState {
    pub fn new(device: Device) -> Self {
        Self {
            scheduler: ModelScheduler::new(SchedulerConfig::default()),
            tokenizer: None,
            device,
            context_length: 4096,
            model_path: None,
            tokenizer_path: None,
            model_config_json: None,
            chat_template: None,
            arch: None,
            hub_repo_id: None,
            hub_revision: None,
            safetensors_files: None,
            precision_policy: PrecisionPolicy::Default,
            rayon_thread_limit: None,
            performance_monitor: Arc::new(PerformanceMonitor::new(1000)),
            // Prefix cache включён по умолчанию (32 записи)
            prefix_cache: PrefixCache::new(PrefixCacheConfig::enabled(32)),
        }
    }

    pub fn save_precision(&self, app: &AppHandle) -> Result<(), String> {
        let profile_dir = Self::ensure_profile_dir(app)?;
        let path = profile_dir.join("precision.json");
        let file =
            File::create(&path).map_err(|e| format!("Failed to create precision file: {}", e))?;
        serde_json::to_writer(file, &self.precision_policy)
            .map_err(|e| format!("Failed to serialize precision: {}", e))?;
        Ok(())
    }

    pub fn load_precision(app: &AppHandle) -> Result<Precision, String> {
        let profile_dir = Self::profile_dir(app)?;
        let path = profile_dir.join("precision.json");
        if path.exists() {
            let file =
                File::open(&path).map_err(|e| format!("Failed to open precision file: {}", e))?;
            let _policy: PrecisionPolicy = serde_json::from_reader(file)
                .map_err(|e| format!("Failed to deserialize precision: {}", e))?;
            // For now, let's just return the default precision since we don't have a direct conversion
            Ok(Precision::default())
        } else {
            Ok(Precision::default())
        }
    }

    fn profile_dir(app: &AppHandle) -> Result<PathBuf, String> {
        let dir = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        Ok(dir.join("oxide-lab"))
    }

    fn ensure_profile_dir(app: &AppHandle) -> Result<PathBuf, String> {
        let profile_dir = Self::profile_dir(app)?;
        create_dir_all(&profile_dir)
            .map_err(|e| format!("Failed to create profile directory: {}", e))?;
        Ok(profile_dir)
    }

    pub fn save_thread_limit(app: &AppHandle, limit: Option<usize>) -> Result<(), String> {
        let profile_dir = Self::ensure_profile_dir(app)?;
        let path = profile_dir.join("thread_limit.json");
        let file = File::create(&path)
            .map_err(|e| format!("Failed to create thread limit file: {}", e))?;
        serde_json::to_writer(file, &limit)
            .map_err(|e| format!("Failed to serialize thread limit: {}", e))?;
        Ok(())
    }

    pub fn load_thread_limit(app: &AppHandle) -> Result<Option<usize>, String> {
        let profile_dir = Self::profile_dir(app)?;
        let path = profile_dir.join("thread_limit.json");
        if path.exists() {
            let file = File::open(&path)
                .map_err(|e| format!("Failed to open thread limit file: {}", e))?;
            let limit: Option<usize> = serde_json::from_reader(file)
                .map_err(|e| format!("Failed to deserialize thread limit: {}", e))?;
            Ok(limit)
        } else {
            Ok(None)
        }
    }
}

pub type SharedState = Arc<Mutex<ModelState>>;
