//! Local safetensors model loader.
//! 
//! This module provides functionality for loading safetensors models from local paths,
//! using the universal weights utilities.

use crate::core::device::{device_label, select_device};
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, extract_chat_template};
use crate::models::common::model::ModelBackend;
// Import our new weights module
use crate::core::weights;
use std::path::Path;

pub fn load_local_safetensors_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    model_path: String,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let dev = select_device(device_pref);
    guard.device = dev;
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
    if config_path.exists() {
        if let Ok(bytes) = std::fs::read(&config_path) {
            guard.model_config_json = Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    // Используем универсальный загрузчик весов для определения списка файлов safetensors
    let safetensors_files = weights::local_list_safetensors(model_dir)?;

    // Validate the local safetensors files
    weights::validate_safetensors_files(&safetensors_files)?;

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

    // Попытаемся построить модель из safetensors используя VarBuilder
    let mut built_model_opt: Option<Box<dyn crate::models::common::model::ModelBackend + Send>> = None;
    
    if config_path.exists() {
        if let Ok(bytes) = std::fs::read(&config_path) {
            // Пробуем определить архитектуру по config.json
            if let Ok(_cfg_str) = String::from_utf8(bytes) {
                // Попробуем собрать candle-модель через VarBuilder используя универсальный подход
                let device = guard.device.clone();
                
                // Используем универсальный VarBuilder с единым подходом к dtype
                let vb = weights::build_varbuilder(&safetensors_files, &device)?;
                
                // Загружаем конфиг в candle_transformers
                if let Ok(cfg_bytes) = std::fs::read(&config_path) {
                    if let Ok(cfg_val) = serde_json::from_slice::<serde_json::Value>(&cfg_bytes) {
                        // Для Qwen3: используем candle_transformers::models::qwen3::ModelForCausalLM
                        if cfg_val.to_string().to_lowercase().contains("qwen") {
                            // Пытаемся десериализовать в структуру config через candle_transformers API
                            let cfg: Result<candle_transformers::models::qwen3::Config, _> = serde_json::from_slice(&cfg_bytes);
                            if let Ok(cfg_parsed) = cfg {
                                match candle_transformers::models::qwen3::ModelForCausalLM::new(&cfg_parsed, vb) {
                                    Ok(m) => {
                                        let any = crate::models::common::model::AnyModel::from_candle_qwen3(m);
                                        built_model_opt = Some(Box::new(any));
                                    }
                                    Err(e) => println!("[local] candle qwen3 model build failed: {}", e),
                                }
                            }
                        }
                    }
                }
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
    guard.safetensors_files = Some(safetensors_files);
    println!("[load] local safetensors loaded, context_length={}", guard.context_length);
    
    Ok(())
}