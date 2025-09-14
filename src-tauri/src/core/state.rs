use std::fs::File;
use std::sync::{Arc, Mutex};
use candle::Device;
use tokenizers::Tokenizer;
use crate::core::precision::{PrecisionPolicy, Precision};
use tauri::AppHandle;
use tauri::Manager;
use std::fs::{create_dir_all};
use serde_json;

/// Универсальное состояние для любой модели
pub struct ModelState<M> {
    pub(crate) gguf_model: Option<M>,
    pub(crate) gguf_file: Option<File>,
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
}

impl<M> ModelState<M> {
    pub fn new(device: Device) -> Self {
        Self {
            gguf_model: None,
            gguf_file: None,
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
        }
    }

    pub fn save_precision(&self, app: &AppHandle) -> Result<(), String> {
        let dir = app.path().app_local_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let profile_dir = dir.join("oxide-lab");
        if let Err(e) = create_dir_all(&profile_dir) {
            return Err(format!("Failed to create profile directory: {}", e));
        }
        let path = profile_dir.join("precision.json");
        let file = File::create(&path).map_err(|e| format!("Failed to create precision file: {}", e))?;
        serde_json::to_writer(file, &self.precision_policy)
            .map_err(|e| format!("Failed to serialize precision: {}", e))?;
        Ok(())
    }

    pub fn load_precision(app: &AppHandle) -> Result<Precision, String> {
        let dir = app.path().app_local_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let profile_dir = dir.join("oxide-lab");
        let path = profile_dir.join("precision.json");
        if path.exists() {
            let file = File::open(&path).map_err(|e| format!("Failed to open precision file: {}", e))?;
            let _policy: PrecisionPolicy = serde_json::from_reader(file)
                .map_err(|e| format!("Failed to deserialize precision: {}", e))?;
            // For now, let's just return the default precision since we don't have a direct conversion
            Ok(Precision::default())
        } else {
            Ok(Precision::default())
        }
    }
}

pub type SharedState<M> = Arc<Mutex<ModelState<M>>>;
