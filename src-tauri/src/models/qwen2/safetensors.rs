//! Qwen2/2.5 SafeTensors loading
//!
//! Загрузка Qwen2/2.5 моделей из SafeTensors формата.
//! Использует candle_transformers::models::qwen2.

use candle::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config, ModelForCausalLM};
use std::path::{Path, PathBuf};

use super::Qwen2Backend;
use crate::models::api::optimization::OptimizationConfig;

impl Qwen2Backend {
    /// Создаёт бекенд из SafeTensors файлов
    pub fn from_safetensors(
        filenames: &[PathBuf],
        config_path: &Path,
        device: &Device,
        dtype: DType,
    ) -> Result<Self, String> {
        // Загружаем конфигурацию
        let config_data =
            std::fs::read(config_path).map_err(|e| format!("Failed to read config.json: {}", e))?;
        let config: Config = serde_json::from_slice(&config_data)
            .map_err(|e| format!("Failed to parse config.json: {}", e))?;

        // Создаём VarBuilder из SafeTensors
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(filenames, dtype, device)
                .map_err(|e| format!("Failed to load SafeTensors: {}", e))?
        };

        // Создаём конфигурацию оптимизаций
        let optimization = OptimizationConfig::for_safetensors(dtype);

        log::info!(
            "Loading Qwen2/2.5 SafeTensors: vocab_size={}, max_position_embeddings={}",
            config.vocab_size,
            config.max_position_embeddings
        );

        // Создаём модель
        let inner = ModelForCausalLM::new(&config, vb)
            .map_err(|e| format!("Failed to build Qwen2 model: {}", e))?;

        Ok(Self::new_full(
            inner,
            device.clone(),
            config.vocab_size,
            config.max_position_embeddings,
            optimization,
        ))
    }

    /// Создаёт бекенд из директории модели
    pub fn from_safetensors_dir(
        model_dir: &Path,
        device: &Device,
        dtype: DType,
    ) -> Result<Self, String> {
        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            return Err("config.json not found in model directory".to_string());
        }

        // Определяем файлы весов
        let filenames = Self::find_weight_files(model_dir)?;

        Self::from_safetensors(&filenames, &config_path, device, dtype)
    }

    /// Находит файлы весов в директории
    fn find_weight_files(model_dir: &Path) -> Result<Vec<PathBuf>, String> {
        // Проверяем model.safetensors.index.json
        let index_path = model_dir.join("model.safetensors.index.json");
        if index_path.exists() {
            return Self::load_indexed_files(model_dir, &index_path);
        }

        // Проверяем единственный model.safetensors
        let single_file = model_dir.join("model.safetensors");
        if single_file.exists() {
            return Ok(vec![single_file]);
        }

        Err("No model.safetensors or model.safetensors.index.json found".to_string())
    }

    /// Загружает список файлов из index.json
    fn load_indexed_files(model_dir: &Path, index_path: &Path) -> Result<Vec<PathBuf>, String> {
        let content = std::fs::read_to_string(index_path)
            .map_err(|e| format!("Failed to read index: {}", e))?;

        let index: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse index: {}", e))?;

        let weight_map = index
            .get("weight_map")
            .and_then(|v| v.as_object())
            .ok_or("weight_map not found in index")?;

        // Собираем уникальные имена файлов
        let mut file_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for filename in weight_map.values() {
            if let Some(name) = filename.as_str() {
                file_set.insert(name.to_string());
            }
        }

        // Конвертируем в пути и проверяем существование
        let files: Vec<PathBuf> = file_set
            .into_iter()
            .map(|name| model_dir.join(name))
            .filter(|path| path.exists())
            .collect();

        if files.is_empty() {
            return Err("No SafeTensors files found from index".to_string());
        }

        Ok(files)
    }
}
