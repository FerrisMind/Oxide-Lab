use crate::core::precision::{Precision, PrecisionPolicy};
use crate::core::state::{ModelState, SharedState};
use crate::core::types::{DevicePreference, GenerateRequest, LoadRequest};
use crate::generate;
use crate::generate::cancel::{cancel_model_loading_cmd, CANCEL_LOADING};
use crate::models::common::model::ModelBackend;
use crate::{log_load, log_template};
use candle::quantized::gguf_file;
use candle::Device;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use tauri::Manager;

// Import our new modules
mod model_loading;
use model_loading::{gguf, hub_gguf, safetensors};
mod device;
pub mod performance_api;
mod template;
pub use performance_api::{
    clear_performance_metrics, get_average_duration, get_memory_usage, get_performance_metrics,
    get_startup_metrics, get_system_usage,
};
pub mod download_manager;
pub mod local_models;
pub use local_models::{
    delete_local_model, download_hf_model_file, get_model_readme, parse_gguf_metadata,
    scan_local_models_folder, scan_models_folder, search_huggingface_gguf,
};

// Модальная индикация удалена: проект реализует единую обработку вложений независимо от модели.

#[tauri::command]
pub fn load_model(
    app: tauri::AppHandle,
    state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>,
    req: LoadRequest,
) -> Result<(), String> {
    // Сбрасываем флаг отмены перед новой загрузкой
    CANCEL_LOADING.store(false, std::sync::atomic::Ordering::SeqCst);

    // Запускаем тяжёлую загрузку в пуле блокирующих задач, не блокируя IPC/UI
    let app_clone = app.clone();
    let state_arc = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = match state_arc.lock() {
            Ok(g) => g,
            Err(e) => {
                crate::api::model_loading::emit_load_progress(
                    &app_clone,
                    "error",
                    0,
                    None,
                    true,
                    Some(&e.to_string()),
                );
                return;
            }
        };

        let res: Result<(), String> = match req {
            LoadRequest::Gguf {
                model_path,
                tokenizer_path: _tokenizer_path,
                context_length,
                device,
            } => gguf::load_gguf_model(&app_clone, &mut guard, model_path, context_length, device),
            LoadRequest::HubGguf {
                repo_id,
                revision,
                filename,
                context_length,
                device,
            } => hub_gguf::load_hub_gguf_model(
                &app_clone,
                &mut guard,
                repo_id,
                revision,
                filename,
                context_length,
                device,
            ),
            LoadRequest::HubSafetensors {
                repo_id,
                revision,
                context_length,
                device,
            } => safetensors::load_hub_safetensors_model(
                &app_clone,
                &mut guard,
                repo_id,
                revision,
                context_length,
                device,
            ),
            LoadRequest::LocalSafetensors {
                model_path,
                context_length,
                device,
            } => safetensors::load_local_safetensors_model(
                &app_clone,
                &mut guard,
                model_path,
                context_length,
                device,
            ),
        };

        if let Err(ref e) = res {
            // финальный сигнал об ошибке
            crate::api::model_loading::emit_load_progress(
                &app_clone,
                "error",
                0,
                None,
                true,
                Some(e),
            );
        }
    });

    // Немедленно возвращаем управление на фронтенд; прогресс придёт событиями
    Ok(())
}

#[tauri::command]
pub fn cancel_model_loading() -> Result<(), String> {
    cancel_model_loading_cmd()
}

#[tauri::command]
pub fn unload_model(
    app: tauri::AppHandle,
    state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<(), String> {
    let app_clone = app.clone();
    let state_arc = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = match state_arc.lock() {
            Ok(g) => g,
            Err(e) => {
                crate::api::model_loading::emit_load_progress(
                    &app_clone,
                    "error",
                    0,
                    None,
                    true,
                    Some(&e.to_string()),
                );
                return;
            }
        };
        let device = guard.device.clone();
        // Эмиссия упрощённого прогресса выгрузки
        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_start",
            0,
            None,
            false,
            None,
        );
        guard.gguf_model = None;
        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_model",
            40,
            None,
            false,
            None,
        );
        guard.gguf_file = None;
        guard.tokenizer = None;
        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_tokenizer",
            70,
            None,
            false,
            None,
        );
        *guard = ModelState::new(device);
        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_complete",
            100,
            Some("Выгружено"),
            true,
            None,
        );
        log_load!("hard reset: freed model/tokenizer and reset state (preserved device)");
    });
    // Возвращаем управление сразу
    Ok(())
}

#[tauri::command]
pub async fn generate_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    req: GenerateRequest,
) -> Result<(), String> {
    if let Ok(guard) = state.lock() {
        log_template!(
            "present_at_generate={}",
            guard.chat_template.as_ref().map(|_| true).unwrap_or(false)
        );
    }
    generate::generate_stream_cmd(app, state, req).await
}

#[tauri::command]
pub fn cancel_generation() -> Result<(), String> {
    generate::cancel_generation_cmd()
}

#[tauri::command]
pub fn get_chat_template(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Option<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.chat_template.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsgDto {
    pub role: String,
    pub content: String,
}

#[tauri::command]
pub fn render_prompt(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    messages: Vec<ChatMsgDto>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    template::render_prompt(&guard.chat_template, messages)
}

#[tauri::command]
pub fn is_model_loaded(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.gguf_model.is_some() && guard.tokenizer.is_some())
}

#[tauri::command]
pub fn set_device(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    pref: DevicePreference,
) -> Result<(), String> {
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
pub fn get_device_info(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<DeviceInfoDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let current = crate::core::device::device_label(&guard.device).to_string();
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    let cuda_available = candle::Device::cuda_if_available(0).is_ok();
    #[cfg(not(feature = "cuda"))]
    let cuda_available = false;
    Ok(DeviceInfoDto {
        cuda_build,
        cuda_available,
        current,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCudaDto {
    pub cuda_build: bool,
    pub ok: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn probe_cuda() -> Result<ProbeCudaDto, String> {
    let cuda_build = cfg!(feature = "cuda");
    #[cfg(feature = "cuda")]
    {
        match candle::Device::cuda_if_available(0) {
            Ok(_) => Ok(ProbeCudaDto {
                cuda_build,
                ok: true,
                error: None,
            }),
            Err(e) => Ok(ProbeCudaDto {
                cuda_build,
                ok: false,
                error: Some(e.to_string()),
            }),
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        Ok(ProbeCudaDto {
            cuda_build,
            ok: false,
            error: Some("built without cuda feature".to_string()),
        })
    }
}

#[tauri::command]
pub fn get_precision_policy(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<PrecisionPolicy, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.precision_policy.clone())
}

#[tauri::command]
pub fn set_precision_policy(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    policy: PrecisionPolicy,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.precision_policy = policy;
    Ok(())
}

#[tauri::command]
pub fn get_precision(
    app: tauri::AppHandle,
    _state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Precision, String> {
    let precision = ModelState::<Box<dyn ModelBackend + Send>>::load_precision(&app)
        .map_err(|e| e.to_string())?;
    Ok(precision)
}

#[tauri::command]
pub fn set_precision(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    precision_str: String,
) -> Result<(), String> {
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

// --- GGUF metadata helpers ---

#[tauri::command]
pub fn gguf_list_metadata_keys_from_path(path: String) -> Result<Vec<String>, String> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err(format!("Not a file: {}", path));
    }
    if !p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
    {
        return Err("Path is not a .gguf file".to_string());
    }
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut f).map_err(|e| e.to_string())?;
    let mut keys: Vec<String> = content.metadata.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}

#[tauri::command]
pub fn gguf_list_metadata_keys(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Vec<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let path_str = guard
        .model_path
        .as_ref()
        .ok_or_else(|| "No model loaded".to_string())?;
    let p = Path::new(path_str);
    if !p.is_file() {
        return Err(format!("Not a file: {}", path_str));
    }
    if !p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
    {
        return Err("Loaded model is not a .gguf file".to_string());
    }
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut f).map_err(|e| e.to_string())?;
    let mut keys: Vec<String> = content.metadata.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}

// --- Experimental features flag ---

#[tauri::command]
pub fn get_experimental_features_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    let profile_dir = dir.join("oxide-lab");
    let path = profile_dir.join("experimental_features.json");

    if !path.exists() {
        return Ok(false);
    }

    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open experimental features file: {}", e))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read experimental features file: {}", e))?;
    let enabled: bool = serde_json::from_str(&buf)
        .map_err(|e| format!("Failed to parse experimental features file: {}", e))?;
    Ok(enabled)
}

#[tauri::command]
pub fn set_experimental_features_enabled(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    let profile_dir = dir.join("oxide-lab");
    create_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to create profile directory: {}", e))?;
    let path = profile_dir.join("experimental_features.json");

    let mut file = File::create(&path)
        .map_err(|e| format!("Failed to create experimental features file: {}", e))?;
    let data = serde_json::to_string(&enabled)
        .map_err(|e| format!("Failed to serialize experimental features: {}", e))?;
    file.write_all(data.as_bytes())
        .map_err(|e| format!("Failed to write experimental features file: {}", e))?;
    Ok(())
}
