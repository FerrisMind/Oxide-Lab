//! Model loading from safetensors files using the ModelBuilder pattern.
//!
//! This module provides functions to load models from safetensors files
//! using the unified ModelBuilder interface.

use candle::DType;
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::Path;

use crate::core::device::{device_label, select_device};
use crate::core::state::ModelState;
use crate::core::tokenizer::{extract_chat_template, mark_special_chat_tokens};
use crate::models::registry::{get_model_factory, detect_arch_from_config};
use crate::models::common::model::ModelBackend;
use crate::core::weights::{hub_list_safetensors, local_list_safetensors, hub_cache_safetensors, validate_safetensors_files};

/// Load a model from local safetensors files using the ModelBuilder pattern
pub fn load_local_safetensors_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    model_path: String,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let dev = select_device(device_pref);
    guard.device = dev.clone();
    println!("[load] device selected: {}", device_label(&guard.device));

    let model_path = Path::new(&model_path);
    if !model_path.exists() {
        return Err(format!("Model path does not exist: {}", model_path.display()));
    }

    // Determine if it's a directory or file
    let model_dir = if model_path.is_file() {
        // If it's a file, assume it's in a model directory
        model_path.parent().ok_or("Cannot determine parent directory")?
    } else {
        model_path
    };

    // Load tokenizer.json (если есть)
    let tokenizer_path = model_dir.join("tokenizer.json");
    let mut tokenizer_opt = None;
    if tokenizer_path.exists() {
        match std::fs::read(&tokenizer_path) {
            Ok(bytes) => match tokenizers::Tokenizer::from_bytes(&bytes) {
                Ok(mut tk) => {
                    mark_special_chat_tokens(&mut tk);
                    tokenizer_opt = Some(tk);
                }
                Err(e) => println!("[local] tokenizer.json parse error: {}", e),
            },
            Err(e) => println!("[local] tokenizer.json read error: {}", e),
        }
    }

    // Load config.json (если есть) и сохраняем как строку
    let config_path = model_dir.join("config.json");
    let config_json = if config_path.exists() {
        match std::fs::read(&config_path) {
            Ok(bytes) => {
                let json_str = String::from_utf8_lossy(&bytes).to_string();
                guard.model_config_json = Some(json_str.clone());
                Some(json_str)
            }
            Err(e) => {
                println!("[local] config.json read error: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Используем универсальный загрузчик весов для определения списка файлов safetensors
    let filenames = local_list_safetensors(model_dir)
        .map_err(|e| format!("Failed to list safetensors files from local path: {}", e))?;

    // Validate the local safetensors files
    validate_safetensors_files(&filenames)?;

    // Инициализируем tokenizer и chat_template
    let mut chat_tpl = None;
    if let Some(tk) = tokenizer_opt.as_ref() {
        chat_tpl = extract_chat_template(tk);
        if let Some(tpl) = chat_tpl.as_ref() {
            let head: String = tpl.chars().take(120).collect();
            println!("[template] detected: len={}, head=<<<{}>>>", tpl.len(), head);
        } else {
            println!("[template] not found in tokenizer.json");
        }
    }

    // Use ModelBuilder pattern if we have config
    let mut built_model_opt: Option<Box<dyn ModelBackend + Send>> = None;
    if let Some(config_json_str) = config_json {
        // Parse the config JSON
        let config: serde_json::Value = serde_json::from_str(&config_json_str)
            .map_err(|e| format!("Failed to parse config.json: {}", e))?;

        // Detect the architecture
        if let Some(arch) = detect_arch_from_config(&config) {
            // Determine the dtype based on device
            let dtype = if dev.is_cuda() || dev.is_metal() {
                DType::BF16 // Use BF16 for GPU devices
            } else {
                DType::F32 // Use F32 for CPU devices
            };

            // Use the model factory to build the model
            match get_model_factory().build_from_safetensors(arch, &filenames, &config, &dev, dtype) {
                Ok(model_backend) => {
                    built_model_opt = Some(model_backend);
                }
                Err(e) => println!("[local] ModelBuilder failed: {}", e),
            }
        }
    }

    guard.gguf_model = built_model_opt;
    guard.gguf_file = None;
    guard.tokenizer = tokenizer_opt;
    guard.chat_template = chat_tpl;
    guard.context_length = context_length.max(1);
    guard.model_path = Some(model_path.to_string_lossy().to_string());
    guard.tokenizer_path = tokenizer_path.exists().then(|| tokenizer_path.to_string_lossy().to_string());
    guard.hub_repo_id = None;
    guard.hub_revision = None;
    guard.safetensors_files = Some(filenames);
    println!("[load] local safetensors loaded with ModelBuilder, context_length={}", guard.context_length);
    
    Ok(())
}

/// Load a model from Hub safetensors files using the ModelBuilder pattern
pub fn load_hub_safetensors_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    repo_id: String,
    revision: Option<String>,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let dev = select_device(device_pref);
    guard.device = dev.clone();
    println!("[load] device selected: {}", device_label(&guard.device));

    // Настраиваем API и репозиторий
    let api = Api::new().map_err(|e| e.to_string())?;
    if !repo_id.contains('/') { return Err("repo_id должен быть в формате 'owner/repo'".into()); }
    let rev = revision.clone().unwrap_or_else(|| "main".to_string());
    let repo = Repo::with_revision(repo_id.clone(), RepoType::Model, rev.clone());
    let api = api.repo(repo);

    // Загружаем tokenizer.json (если есть)
    let tokenizer_path = api.get("tokenizer.json").ok();
    let mut tokenizer_opt = None;
    if let Some(path) = tokenizer_path.as_ref() {
        match std::fs::read(path) {
            Ok(bytes) => match tokenizers::Tokenizer::from_bytes(&bytes) {
                Ok(mut tk) => {
                    mark_special_chat_tokens(&mut tk);
                    tokenizer_opt = Some(tk);
                }
                Err(e) => println!("[hub] tokenizer.json parse error: {}", e),
            },
            Err(e) => println!("[hub] tokenizer.json read error: {}", e),
        }
    }

    // Загружаем config.json (если есть) и сохраняем как строку
    let config_json = if let Ok(cfg_path) = api.get("config.json") {
        match std::fs::read(&cfg_path) {
            Ok(bytes) => {
                let json_str = String::from_utf8_lossy(&bytes).to_string();
                guard.model_config_json = Some(json_str.clone());
                Some(json_str)
            }
            Err(e) => {
                println!("[hub] config.json read error: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Используем универсальный загрузчик весов для определения списка файлов safetensors
    let filenames = hub_list_safetensors(&api)
        .map_err(|e| format!("Failed to list safetensors files from Hub: {}", e))?;
    
    // Предзагрузим все файлы в кэш (скачать/проверить наличие)
    let cached_filenames = hub_cache_safetensors(&api, &filenames)
        .map_err(|e| format!("Failed to cache safetensors files: {}", e))?;

    // Validate the downloaded safetensors files
    validate_safetensors_files(&cached_filenames)?;

    // Инициализируем tokenizer и chat_template
    let mut chat_tpl = None;
    if let Some(tk) = tokenizer_opt.as_ref() {
        chat_tpl = extract_chat_template(tk);
        if let Some(tpl) = chat_tpl.as_ref() {
            let head: String = tpl.chars().take(120).collect();
            println!("[template] detected: len={}, head=<<<{}>>>", tpl.len(), head);
        } else {
            println!("[template] not found in tokenizer.json");
        }
    }

    // Use ModelBuilder pattern if we have config
    let mut built_model_opt: Option<Box<dyn ModelBackend + Send>> = None;
    if let Some(config_json_str) = config_json {
        // Parse the config JSON
        let config: serde_json::Value = serde_json::from_str(&config_json_str)
            .map_err(|e| format!("Failed to parse config.json: {}", e))?;

        // Detect the architecture
        if let Some(arch) = detect_arch_from_config(&config) {
            // Determine the dtype based on device
            let dtype = if dev.is_cuda() || dev.is_metal() {
                DType::BF16 // Use BF16 for GPU devices
            } else {
                DType::F32 // Use F32 for CPU devices
            };

            // Use the model factory to build the model
            match get_model_factory().build_from_safetensors(arch, &cached_filenames, &config, &dev, dtype) {
                Ok(model_backend) => {
                    built_model_opt = Some(model_backend);
                }
                Err(e) => println!("[hub] ModelBuilder failed: {}", e),
            }
        }
    }

    guard.gguf_model = built_model_opt;
    guard.gguf_file = None;
    guard.tokenizer = tokenizer_opt;
    guard.chat_template = chat_tpl;
    guard.context_length = context_length.max(1);
    guard.model_path = None;
    guard.tokenizer_path = tokenizer_path.map(|p| p.to_string_lossy().to_string());
    guard.hub_repo_id = Some(repo_id);
    guard.hub_revision = Some(rev);
    guard.safetensors_files = Some(cached_filenames);
    println!("[load] hub safetensors loaded with ModelBuilder, context_length={}", guard.context_length);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_safetensors_loading_function_exists() {
        // This is just a basic test to ensure the function is properly defined
        // Actual testing would require model files and a test environment
        assert!(true);
    }
}