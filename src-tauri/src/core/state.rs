use crate::core::performance::PerformanceMonitor;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri::Manager;

/// Application state — lightweight after EngineManager migration.
/// Retains only settings persistence and performance monitoring.
pub struct ModelState {
    /// Highest allowed Rayon thread count (None = automatic).
    pub(crate) rayon_thread_limit: Option<usize>,
    /// Performance monitor for tracking metrics
    pub(crate) performance_monitor: Arc<PerformanceMonitor>,
}

impl Default for ModelState {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelState {
    pub fn new() -> Self {
        Self {
            rayon_thread_limit: None,
            performance_monitor: Arc::new(PerformanceMonitor::new(1000)),
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
