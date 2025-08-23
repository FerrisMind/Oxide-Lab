use hf_hub::{api::sync::Api, Repo, RepoType};
use crate::core::device::{device_label, select_device};
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, extract_chat_template};
use crate::models::common::model::ModelBackend;

pub fn load_hub_safetensors_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    repo_id: String,
    revision: Option<String>,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let dev = select_device(device_pref);
    guard.device = dev;
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
    if let Ok(cfg_path) = api.get("config.json") {
        if let Ok(bytes) = std::fs::read(cfg_path) {
            guard.model_config_json = Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    // Определяем список файлов весов safetensors: index.json или одиночный файл
    let mut safetensors_files: Vec<String> = Vec::new();
    // Сценарий 1: есть индекс
    if let Ok(index_path) = api.get("model.safetensors.index.json") {
        if let Ok(bytes) = std::fs::read(index_path) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if let Some(files) = value.get("weight_map").and_then(|m| m.as_object()) {
                    let mut set = std::collections::BTreeSet::new();
                    for (_k, v) in files.iter() {
                        if let Some(f) = v.as_str() { set.insert(f.to_string()); }
                    }
                    safetensors_files.extend(set);
                }
            }
        }
    }
    // Сценарий 2: одиночный файл по умолчанию
    if safetensors_files.is_empty() {
        // Попробуем наиболее частое имя
        if api.get("model.safetensors").is_ok() {
            safetensors_files.push("model.safetensors".to_string());
        }
    }
    if safetensors_files.is_empty() {
        return Err("В репозитории не найдены веса safetensors (model.safetensors[.index.json])".into());
    }
    // Предзагрузим все файлы в кэш (скачать/проверить наличие)
    let mut cached_weight_paths: Vec<String> = Vec::with_capacity(safetensors_files.len());
    for fname in safetensors_files.iter() {
        let p = api.get(fname).map_err(|e| format!("hf_hub get {} failed: {}", fname, e))?;
        println!("[hub] safetensors cached: {}", p.display());
        cached_weight_paths.push(p.to_string_lossy().to_string());
    }

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

    // Попытаемся построить модель из safetensors используя VarBuilder как в candle-examples
    // Если не удалось — оставляем состояние в режиме «prepared» (весa закешированы)
    let mut built_model_opt: Option<Box<dyn crate::models::common::model::ModelBackend + Send>> = None;
    if let Ok(cfg_path) = api.get("config.json") {
        if let Ok(bytes) = std::fs::read(cfg_path) {
            // Пробуем определить архитектуру по config.json
            if let Ok(cfg_str) = String::from_utf8(bytes) {
                // Примитивная эвристика: искать qwen в config
                if cfg_str.to_lowercase().contains("qwen") {
                    // Попробуем собрать candle-модель через VarBuilder
                    let device = guard.device.clone();
                    let dtype = if device.is_cuda() || device.is_metal() { candle::DType::BF16 } else { candle::DType::F32 };
                    // Подготовим VarBuilder из закешированных файлов
                    let vb_res = unsafe { candle_nn::VarBuilder::from_mmaped_safetensors(&cached_weight_paths.iter().map(std::path::PathBuf::from).collect::<Vec<_>>(), dtype, &device) };
                    if let Ok(vb) = vb_res {
                        // Загружаем конфиг в candle_transformers
                        if let Ok(cfg_json_path) = api.get("config.json") {
                            if let Ok(cfg_bytes) = std::fs::read(cfg_json_path) {
                                if let Ok(cfg_val) = serde_json::from_slice::<serde_json::Value>(&cfg_bytes) {
                                    // Для Qwen3: используем candle_transformers::models::qwen3::ModelForCausalLM
                                    if cfg_val.to_string().to_lowercase().contains("qwen") {
                                        // Пример из candle-examples: Model3::new(&config, vb)? — здесь нужно конкретно распарсить Config
                                        // Пытаемся десериализовать в структуру config через candle_transformers API
                                        let cfg: Result<candle_transformers::models::qwen3::Config, _> = serde_json::from_slice(&cfg_bytes);
                                        if let Ok(cfg_parsed) = cfg {
                                            match candle_transformers::models::qwen3::ModelForCausalLM::new(&cfg_parsed, vb) {
                                                Ok(m) => {
                                                    let any = crate::models::common::model::AnyModel::from_candle_qwen3(m);
                                                    built_model_opt = Some(Box::new(any));
                                                }
                                                Err(e) => println!("[hub] candle qwen3 model build failed: {}", e),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        println!("[hub] VarBuilder from safetensors failed: {:?}", vb_res.err());
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
    guard.model_path = None;
    guard.tokenizer_path = tokenizer_path.map(|p| p.to_string_lossy().to_string());
    guard.hub_repo_id = Some(repo_id);
    guard.hub_revision = Some(rev);
    guard.safetensors_files = Some(cached_weight_paths);
    println!("[load] hub safetensors prepared (weights cached, tokenizer/config loaded), context_length={}", guard.context_length);
    
    Ok(())
}