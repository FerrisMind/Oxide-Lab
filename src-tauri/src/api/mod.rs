use std::fs::File;
use std::path::PathBuf;
// Emitter нужен косвенно из generate, но здесь не используется

use candle::quantized::gguf_file;

use crate::core::device::{device_label, select_device};
use crate::core::state::{ModelState, SharedState};
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use serde::{Deserialize, Serialize};
use minijinja::{Environment, Value, context};
use crate::core::types::{DevicePreference, GenerateRequest, LoadRequest};
use crate::generate;
use crate::models::qwen3::ModelWeights as Qwen3Gguf;
use crate::models::registry::{detect_arch, ArchKind};

#[tauri::command]
pub fn load_model(state: tauri::State<SharedState<Qwen3Gguf>>, req: LoadRequest) -> Result<(), String> {
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

            let model = match arch {
                ArchKind::Qwen3 => Qwen3Gguf::from_gguf(content, &mut file, &guard.device, context_length, false)
                    .map_err(|e| e.to_string())?,
            };

            guard.gguf_model = Some(model);
            guard.gguf_file = Some(file);
            guard.tokenizer = Some(tokenizer);
            guard.chat_template = chat_tpl;
            let ctx = if context_length == 0 { 1 } else { context_length };
            guard.context_length = ctx;
            guard.model_path = Some(model_path);
            guard.tokenizer_path = None;
            println!("[load] gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn unload_model(state: tauri::State<SharedState<Qwen3Gguf>>) -> Result<(), String> {
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
    state: tauri::State<'_, SharedState<Qwen3Gguf>>,
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
    let msgs_val: Vec<Value> = msgs.iter().map(|m| Value::from_serialize(m)).collect();
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


