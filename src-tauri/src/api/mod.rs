use crate::core::state::{ModelState, SharedState};
use crate::core::types::{DevicePreference, GenerateRequest, LoadRequest};
use crate::core::precision::{PrecisionPolicy, Precision};
use crate::generate;
use crate::models::common::model::ModelBackend;
use serde::{Deserialize, Serialize};
use crate::{log_load, log_template};
use crate::generate::cancel::{CANCEL_LOADING, cancel_model_loading_cmd};
use candle::Device;
use std::path::Path;
use candle::quantized::gguf_file;

// Import our new modules
mod model_loading;
use model_loading::{gguf, hub_gguf, safetensors};
mod device;
mod template;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModalitySupportDto { pub text: bool, pub image: bool, pub audio: bool, pub video: bool }

#[tauri::command]
pub fn get_modality_support(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<ModalitySupportDto, String> {
    let mut cfg_json: Option<serde_json::Value> = None;
    if let Ok(guard) = state.lock() {
        // Try config captured during loading first
        if let Some(s) = guard.model_config_json.as_ref() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) { cfg_json = Some(v); }
        }

        // Fallbacks
        if cfg_json.is_none() {
            // GGUF: reopen file and try to extract config.json from metadata
            if let Some(path_str) = guard.model_path.as_ref() {
                let p = Path::new(path_str);
                if p.is_file() && p.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("gguf")).unwrap_or(false) {
                    if let Ok(mut f) = std::fs::File::open(p) {
                        if let Ok(content) = gguf_file::Content::read(&mut f) {
                            if let Some(s) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
                                .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
                                .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
                            {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) { cfg_json = Some(v); }
                            } else {
                                // Heuristic from GGUF metadata keys if no embedded config JSON
                                let mut flags = crate::core::modality::ModalitySupportDto { text: true, image: false, audio: false, video: false };
                                let has_key = |needle: &str| content.metadata.keys().any(|k| k.to_lowercase().contains(needle));
                                // image: any vision/image-related key or known multimodal arch
                                flags.image = has_key("vision") || has_key("image_") || has_key("mm_vision") || has_key("llava") || has_key("onevision");
                                // audio: any audio-related key or arch
                                flags.audio = has_key("audio");
                                // video: explicit video flags or keys
                                flags.video = flags.image && (has_key("video") || has_key("time_instruction") || has_key("faster_video"));
                                // Return early via JSON constructed from flags
                                return Ok(ModalitySupportDto { text: flags.text, image: flags.image, audio: flags.audio, video: flags.video });
                            }
                        }
                    }
                } else if p.is_dir() {
                    // Safetensors local: read config.json from directory
                    let cfg = p.join("config.json");
                    if cfg.exists() {
                        if let Ok(bytes) = std::fs::read(&cfg) {
                            if let Ok(s) = String::from_utf8(bytes) {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) { cfg_json = Some(v); }
                            }
                        }
                    }
                }
            }
        }
    }

    // Derive flags
    let out = if let Some(cfg) = cfg_json {
        crate::core::modality::detect_from_config(&cfg)
    } else {
        // Default to text only if no config is available
        crate::core::modality::ModalitySupportDto { text: true, image: false, audio: false, video: false }
    };
    Ok(ModalitySupportDto { text: out.text, image: out.image, audio: out.audio, video: out.video })
}

#[tauri::command]
pub fn load_model(app: tauri::AppHandle, state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>, req: LoadRequest) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    // Сбрасываем флаг отмены перед новой загрузкой
    CANCEL_LOADING.store(false, std::sync::atomic::Ordering::SeqCst);

    let res = match req {
        LoadRequest::Gguf { model_path, tokenizer_path: _tokenizer_path, context_length, device } => {
            gguf::load_gguf_model(&app, &mut guard, model_path, context_length, device)
        }
        LoadRequest::HubGguf { repo_id, revision, filename, context_length, device } => {
            hub_gguf::load_hub_gguf_model(&app, &mut guard, repo_id, revision, filename, context_length, device)
        }
        LoadRequest::HubSafetensors { repo_id, revision, context_length, device } => {
            safetensors::load_hub_safetensors_model(&app, &mut guard, repo_id, revision, context_length, device)
        }
        LoadRequest::LocalSafetensors { model_path, context_length, device } => {
            safetensors::load_local_safetensors_model(&app, &mut guard, model_path, context_length, device)
        }
    };
    if let Err(ref e) = res {
        // финальный сигнал об ошибке
        crate::api::model_loading::emit_load_progress(&app, "error", 0, None, true, Some(e));
    }
    res
}

#[tauri::command]
pub fn cancel_model_loading() -> Result<(), String> {
    cancel_model_loading_cmd()
}

#[tauri::command]
pub fn unload_model(app: tauri::AppHandle, state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let device = guard.device.clone();
    // Эмиссия упрощённого прогресса выгрузки
    crate::api::model_loading::emit_load_progress(&app, "unload_start", 0, None, false, None);
    guard.gguf_model = None;
    crate::api::model_loading::emit_load_progress(&app, "unload_model", 40, None, false, None);
    guard.gguf_file = None;
    guard.tokenizer = None;
    crate::api::model_loading::emit_load_progress(&app, "unload_tokenizer", 70, None, false, None);
    *guard = ModelState::new(device);
    crate::api::model_loading::emit_load_progress(&app, "unload_complete", 100, Some("Выгружено"), true, None);
    log_load!("hard reset: freed model/tokenizer and reset state (preserved device)");
    Ok(())
}

#[tauri::command]
pub async fn generate_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    req: GenerateRequest,
) -> Result<(), String> {
    if let Ok(guard) = state.lock() {
        log_template!("present_at_generate={}", guard.chat_template.as_ref().map(|_| true).unwrap_or(false));
    }
    generate::generate_stream_cmd(app, state, req).await
}

#[tauri::command]
pub fn cancel_generation() -> Result<(), String> {
    generate::cancel_generation_cmd()
}

#[tauri::command]
pub fn get_chat_template(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<Option<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.chat_template.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsgDto { pub role: String, pub content: String }

#[tauri::command]
pub fn render_prompt(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>, messages: Vec<ChatMsgDto>) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    template::render_prompt(&guard.chat_template, messages)
}

#[tauri::command]
pub fn is_model_loaded(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.gguf_model.is_some() && guard.tokenizer.is_some())
}

#[tauri::command]
pub fn set_device(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>, pref: DevicePreference) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    device::set_device(&mut guard, pref)
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
pub fn get_device_info(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<DeviceInfoDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let current = crate::core::device::device_label(&guard.device).to_string();
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

#[tauri::command]
pub fn get_precision_policy(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<PrecisionPolicy, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.precision_policy.clone())
}

#[tauri::command]
pub fn set_precision_policy(state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>, policy: PrecisionPolicy) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.precision_policy = policy;
    Ok(())
}

#[tauri::command]
pub fn get_precision(app: tauri::AppHandle, _state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>) -> Result<Precision, String> {
    let precision = ModelState::<Box<dyn ModelBackend + Send>>::load_precision(&app).map_err(|e| e.to_string())?;
    Ok(precision)
}

#[tauri::command]
pub fn set_precision(app: tauri::AppHandle, state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>, precision_str: String) -> Result<(), String> {
    let precision = match precision_str.as_str() {
        "f16" => Precision::F16,
        "f32" => Precision::F32,
        "int8" => Precision::Int8,
        _ => return Err("Invalid precision: must be 'f16', 'f32', or 'int8'".to_string()),
    };

    // Hardware check: Int8 only on CPU for simplicity
    let guard = state.lock().map_err(|e| e.to_string())?;
    if matches!(precision, Precision::Int8) && !matches!(guard.device, Device::Cpu) {
        return Err("Int8 precision only supported on CPU".to_string());
    }
    drop(guard);

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.precision_policy = match precision {
        Precision::F16 => PrecisionPolicy::MemoryEfficient,
        Precision::F32 => PrecisionPolicy::Default,
        Precision::Int8 => PrecisionPolicy::MemoryEfficient,
    };
    guard.save_precision(&app).map_err(|e| e.to_string())
}
