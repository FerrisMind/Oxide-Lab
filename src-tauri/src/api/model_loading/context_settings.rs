use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextSource {
    Auto,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContextSettings {
    pub size: usize,
    pub source: ContextSource,
    pub last_autotune: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextSettingsStore {
    // Map model_id (or path) -> settings
    pub models: HashMap<String, ModelContextSettings>,
}

pub struct ContextSettingsManager {
    config_path: PathBuf,
    store: Mutex<ContextSettingsStore>,
}

impl ContextSettingsManager {
    pub fn new(app: &AppHandle) -> Self {
        let config_dir = app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let config_path = config_dir.join("context_settings.json");

        let store = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => ContextSettingsStore::default(),
            }
        } else {
            ContextSettingsStore::default()
        };

        Self {
            config_path,
            store: Mutex::new(store),
        }
    }

    pub fn get_settings(&self, model_id: &str) -> Option<ModelContextSettings> {
        let store = self.store.lock().unwrap();
        store.models.get(model_id).cloned()
    }

    pub fn save_settings(
        &self,
        model_id: &str,
        settings: ModelContextSettings,
    ) -> Result<(), String> {
        let mut store = self.store.lock().unwrap();
        store.models.insert(model_id.to_string(), settings);

        // Persist to disk
        let json = serde_json::to_string_pretty(&*store).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
