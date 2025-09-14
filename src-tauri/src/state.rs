use crate::core::precision::Precision;
use std::fs::{File, create_dir_all};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use candle::Device;
use tokenizers::Tokenizer;
use crate::models::qwen3::model::ModelWeights as Qwen3Gguf;
use crate::core::precision::Precision;
use crate::models::common::model::ModelBackend;
use std::boxed::Box;
use tauri::{AppHandle, api::path::app_local_data_dir};
use serde_json;

/// Глобальное состояние модели и токенизатора
pub(crate) struct ModelState<T> {
    pub(crate) gguf_model: Option<T>,
    pub(crate) gguf_file: Option<File>,
    pub(crate) tokenizer: Option<Tokenizer>,
    pub(crate) device: Device,
    pub(crate) context_length: usize,
    pub(crate) precision: Precision,
    pub(crate) model_path: Option<String>,
    pub(crate) tokenizer_path: Option<String>,
    pub(crate) model_config_json: Option<String>,
}

impl<T> ModelState<T> {
    pub fn save_precision(&self, app: &AppHandle) -> Result<(), String> {
        let dir = app_local_data_dir(app)
            .ok_or("Failed to get app data directory")?;
        let profile_dir = dir.join("oxide-lab");
        if let Err(e) = create_dir_all(&profile_dir) {
            return Err(format!("Failed to create profile directory: {}", e));
        }
        let path = profile_dir.join("precision.json");
        let file = File::create(&path).map_err(|e| format!("Failed to create precision file: {}", e))?;
        serde_json::to_writer(file, &self.precision)
            .map_err(|e| format!("Failed to serialize precision: {}", e))?;
        Ok(())
    }

    pub fn load_precision(app: &AppHandle) -> Result<Precision, String> {
        let dir = app_local_data_dir(app)
            .ok_or("Failed to get app data directory")?;
        let profile_dir = dir.join("oxide-lab");
        let path = profile_dir.join("precision.json");
        if path.exists() {
            let file = File::open(&path).map_err(|e| format!("Failed to open precision file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to deserialize precision: {}", e))
        } else {
            Ok(Precision::default())
        }
    }
}

pub(crate) type SharedState = Arc<Mutex<ModelState<Box<dyn ModelBackend + Send>>>>;

impl<T> ModelState<T> {
    pub(crate) fn new(device: Device) -> Self {
        Self {
            gguf_model: None,
            gguf_file: None,
            tokenizer: None,
            device,
            context_length: 4096,
            precision: Precision::default(),
            model_path: None,
            tokenizer_path: None,
            model_config_json: None,
        }
    }
}

pub(crate) type SharedState = Arc<Mutex<ModelState>>;


