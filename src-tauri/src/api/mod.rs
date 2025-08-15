use std::fs::File;
use std::path::PathBuf;
// Emitter нужен косвенно из generate, но здесь не используется

use candle::quantized::gguf_file;
use hf_hub::{api::sync::Api, Repo, RepoType};

use crate::core::device::{device_label, select_device};
use crate::core::state::{ModelState, SharedState};
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use serde::{Deserialize, Serialize};
use minijinja::{Environment, Value, context};
use crate::core::types::{DevicePreference, GenerateRequest, LoadRequest};
use crate::generate;
use crate::models::qwen3::ModelWeights as Qwen3Gguf;
use crate::models::registry::{detect_arch, ArchKind};
use crate::models::common::model::{AnyModel, ModelBackend};

#[tauri::command]
pub fn load_model(state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>, req: LoadRequest) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    match req {
        LoadRequest::Gguf { model_path, tokenizer_path: _tokenizer_path, context_length, device } => {
            let dev = select_device(device);
            guard.device = dev;
            println!("[load] device selected: {}", device_label(&guard.device));

            let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
            let content = gguf_file::Content::read(&mut file)
                .map_err(|e| format!("{}", e.with_path(PathBuf::from(model_path.clone()))))?;

            let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
            mark_special_chat_tokens(&mut tokenizer);
            let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));
            match &chat_tpl {
                Some(tpl) => {
                    let head: String = tpl.chars().take(120).collect();
                    println!("[template] detected: len={}, head=<<<{}>>>", tpl.len(), head);
                }
                None => println!("[template] not found in tokenizer.json"),
            }
            let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

            if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
                .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
                .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
            {
                guard.model_config_json = Some(gg);
            }

            let model_any = match arch {
                ArchKind::Qwen3 => {
                    let m = Qwen3Gguf::from_gguf(content, &mut file, &guard.device, context_length, false)
                        .map_err(|e| e.to_string())?;
                    AnyModel::from_qwen3(m)
                }
            };

            guard.gguf_model = Some(Box::new(model_any));
            guard.gguf_file = Some(file);
            guard.tokenizer = Some(tokenizer);
            guard.chat_template = chat_tpl;
            let ctx = if context_length == 0 { 1 } else { context_length };
            guard.context_length = ctx;
            guard.model_path = Some(model_path);
            guard.tokenizer_path = None;
            println!("[load] gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
        }
        LoadRequest::HubGguf { repo_id, revision, filename, context_length, device } => {
            let dev = select_device(device);
            guard.device = dev;
            println!("[load] device selected: {}", device_label(&guard.device));

            // Инициализация HF Hub API c локальным кэшем
            let api = Api::new().map_err(|e| e.to_string())?;
            if !repo_id.contains('/') { return Err("repo_id должен быть в формате 'owner/repo'".into()); }
            let rev = revision.unwrap_or_else(|| "main".to_string());
            let repo = Repo::with_revision(repo_id.clone(), RepoType::Model, rev);
            let api = api.repo(repo);

            // Скачиваем GGUF-файл в кэш и открываем
            let model_path = api
                .get(&filename)
                .map_err(|e| format!("hf_hub get {} failed: {}", filename, e))?;
            println!("[hub] gguf cached at {}", model_path.display());
            let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
            let content = gguf_file::Content::read(&mut file)
                .map_err(|e| format!("{}", e.with_path(model_path.clone())))?;

            // Токенизатор и шаблон чата из GGUF-метаданных
            let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
            mark_special_chat_tokens(&mut tokenizer);
            let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));
            match &chat_tpl {
                Some(tpl) => {
                    let head: String = tpl.chars().take(120).collect();
                    println!("[template] detected: len={}, head=<<<{}>>>", tpl.len(), head);
                }
                None => println!("[template] not found in tokenizer.json"),
            }

            let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

            if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
                .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
                .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
            {
                guard.model_config_json = Some(gg);
            }

            let model_any = match arch {
                ArchKind::Qwen3 => {
                    let m = Qwen3Gguf::from_gguf(content, &mut file, &guard.device, context_length, false)
                        .map_err(|e| e.to_string())?;
                    AnyModel::from_qwen3(m)
                }
            };

            guard.gguf_model = Some(Box::new(model_any));
            guard.gguf_file = Some(file);
            guard.tokenizer = Some(tokenizer);
            guard.chat_template = chat_tpl;
            let ctx = if context_length == 0 { 1 } else { context_length };
            guard.context_length = ctx;
            guard.model_path = Some(model_path.to_string_lossy().to_string());
            guard.tokenizer_path = None;
            println!("[load] hub gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
        }
        LoadRequest::HubSafetensors { repo_id, revision, context_length, device } => {
            let dev = select_device(device);
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
        }
    }

    Ok(())
}

#[tauri::command]
pub fn unload_model(state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let device = guard.device.clone();
    guard.gguf_model = None;
    guard.gguf_file = None;
    guard.tokenizer = None;
    *guard = ModelState::new(device);
    println!("[unload] hard reset: freed model/tokenizer and reset state (preserved device)");
    Ok(())
}

#[tauri::command]
pub async fn generate_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    req: GenerateRequest,
) -> Result<(), String> {
    if let Ok(guard) = state.lock() {
        println!("[template] present_at_generate={}", guard.chat_template.as_ref().map(|_| true).unwrap_or(false));
    }
    generate::generate_stream_cmd(app, state, req).await
}

#[tauri::command]
pub fn cancel_generation() -> Result<(), String> {
    generate::cancel_generation_cmd()
}

#[tauri::command]
pub fn get_chat_template(state: tauri::State<SharedState<Qwen3Gguf>>) -> Result<Option<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.chat_template.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsgDto { pub role: String, pub content: String }

#[tauri::command]
pub fn render_prompt(state: tauri::State<SharedState<Qwen3Gguf>>, messages: Vec<ChatMsgDto>) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let tpl = match &guard.chat_template { Some(s) if !s.trim().is_empty() => s.clone(), _ => return Err("chat_template not available".into()) };
    // Лог на вход
    println!("[template] render: msgs={}, tpl_len={}", messages.len(), tpl.len());
    let msgs = messages;
    let mut env = Environment::new();
    env.add_template("tpl", &tpl).map_err(|e| e.to_string())?;
    let tmpl = env.get_template("tpl").map_err(|e| e.to_string())?;
    // minijinja контекст
    let msgs_val: Vec<Value> = msgs.iter().map(Value::from_serialize).collect();
    let rendered = tmpl
        .render(context! { messages => msgs_val, add_generation_prompt => true, tools => Vec::<String>::new() })
        .map_err(|e| e.to_string())?;
    println!("[template] render ok, prefix=<<<{}>>>", rendered.chars().take(120).collect::<String>());
    Ok(rendered)
}

#[tauri::command]
pub fn is_model_loaded(state: tauri::State<SharedState<Qwen3Gguf>>) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.gguf_model.is_some() && guard.tokenizer.is_some())
}

#[tauri::command]
pub fn set_device(state: tauri::State<SharedState<Qwen3Gguf>>, pref: DevicePreference) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    // Явно проверяем запрос CUDA и возвращаем ошибку, если инициализация не удалась
    match pref {
        DevicePreference::Cuda { index } => {
            match candle::Device::cuda_if_available(index) {
                Ok(dev) => {
                    guard.device = dev;
                }
                Err(e) => {
                    return Err(format!("CUDA init failed (index={}): {}", index, e));
                }
            }
        }
        DevicePreference::Cpu => {
            guard.device = candle::Device::Cpu;
        }
        DevicePreference::Auto => {
            guard.device = candle::Device::Cpu;
        }
        DevicePreference::Metal => {
            guard.device = candle::Device::Cpu;
        }
    }
    let label = device_label(&guard.device);
    println!("[device] switched -> {}", label);
    // Если модель загружена — перезагрузим её под выбранное устройство
    if guard.gguf_model.is_some() {
        // Перечитываем с диска по сохранённому пути
        let model_path = match guard.model_path.clone() {
            Some(p) => p,
            None => return Ok(()),
        };
        let ctx_len = guard.context_length.max(1);
        let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("{}", e.with_path(PathBuf::from(model_path.clone()))))?;

        // Токенизатор и шаблон чата
        let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
        mark_special_chat_tokens(&mut tokenizer);
        let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));

        // Архитектура
        let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

        // Создание модели на новом устройстве
        let model = match arch {
            ArchKind::Qwen3 => Qwen3Gguf::from_gguf(content, &mut file, &guard.device, ctx_len, false)
                .map_err(|e| e.to_string())?,
        };

        guard.gguf_model = Some(model);
        guard.gguf_file = Some(file);
        guard.tokenizer = Some(tokenizer);
        guard.chat_template = chat_tpl;
        println!("[device] model reloaded for {}", label);
    }
    Ok(())
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfoDto {
    pub cuda_build: bool,
    pub cuda_available: bool,
    pub current: String,
}

#[tauri::command]
pub fn get_device_info(state: tauri::State<SharedState<Qwen3Gguf>>) -> Result<DeviceInfoDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let current = device_label(&guard.device).to_string();
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    let cuda_available = candle::Device::cuda_if_available(0).is_ok();
    #[cfg(not(feature = "cuda"))]
    let cuda_available = false;
    Ok(DeviceInfoDto { cuda_build, cuda_available, current })
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCudaDto { pub cuda_build: bool, pub ok: bool, pub error: Option<String> }

#[tauri::command]
pub fn probe_cuda() -> Result<ProbeCudaDto, String> {
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    {
        match candle::Device::cuda_if_available(0) {
            Ok(_) => Ok(ProbeCudaDto { cuda_build, ok: true, error: None }),
            Err(e) => Ok(ProbeCudaDto { cuda_build, ok: false, error: Some(e.to_string()) }),
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        Ok(ProbeCudaDto { cuda_build, ok: false, error: Some("built without cuda feature".to_string()) })
    }
}


