//! Llama SafeTensors loading
//!
//! Загрузка Llama-подобных моделей из SafeTensors формата.

use candle::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Cache, Llama, LlamaConfig};
use std::path::{Path, PathBuf};

use super::LlamaBackend;
use crate::models::api::optimization::OptimizationConfig;
use crate::models::common::is_flash_attention_available;

impl LlamaBackend {
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

        // Десериализуем сразу в LlamaConfig из candle_transformers
        let mut llama_config: LlamaConfig = serde_json::from_slice(&config_data)
            .map_err(|e| format!("Failed to parse config.json: {}", e))?;

        // Heuristic fix for Llama 3 if rope_theta is missing/default
        // Llama 3 uses rope_theta = 500000.0, but parser might default to 10000.0 if field is missing or different
        // We use vocab_size check (128256 for Llama 3 vs 32000 for Llama 2)
        if llama_config.vocab_size >= 128000 && llama_config.rope_theta < 100001.0 {
            log::warn!(
                "Detected Llama 3 (vocab > 128k) with low rope_theta ({:?}). Forcing 500000.0",
                llama_config.rope_theta
            );
            llama_config.rope_theta = 500000.0;
        }

        // Сохраняем vocab_size и max_pos до перемещения в into_config
        let vocab_size = llama_config.vocab_size;
        let max_seq_len = llama_config.max_position_embeddings;

        log::info!(
            "Llama Config Loaded: vocab={}, dim={}, layers={}, heads={}, kv_heads={:?}, rope={:?}, max_seq={}",
            vocab_size,
            llama_config.hidden_size,
            llama_config.num_hidden_layers,
            llama_config.num_attention_heads,
            llama_config.num_key_value_heads,
            llama_config.rope_theta,
            max_seq_len
        );

        // Создаём VarBuilder из SafeTensors
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(filenames, dtype, device)
                .map_err(|e| format!("Failed to load SafeTensors: {}", e))?
        };

        // Создаём конфигурацию оптимизаций
        let optimization = OptimizationConfig::for_safetensors(dtype);

        // Проверяем условия для Flash Attention
        let fa_available = is_flash_attention_available();
        let fa_opt = optimization.uses_flash_attn();
        let use_flash = fa_available && fa_opt && (dtype == DType::BF16 || dtype == DType::F16);

        log::info!(
            "Llama SafeTensors: dtype={:?}, flash_attn available={}, using={}",
            dtype,
            fa_available,
            use_flash
        );

        // Создаём внутренний Config для Llama
        let config = llama_config.into_config(use_flash);

        // Создаём кэш и модель
        let cache = Cache::new(true, dtype, &config, device)
            .map_err(|e| format!("Failed to create cache: {}", e))?;

        let model =
            Llama::load(vb, &config).map_err(|e| format!("Failed to build Llama model: {}", e))?;

        Ok(Self::new_full(
            model,
            cache,
            device.clone(),
            vocab_size,
            max_seq_len,
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

        let mut file_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for filename in weight_map.values() {
            if let Some(name) = filename.as_str() {
                file_set.insert(name.to_string());
            }
        }

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
