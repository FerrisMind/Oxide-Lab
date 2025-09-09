use crate::core::state::{ModelState, SharedState};
use crate::core::types::{DevicePreference, GenerateRequest, LoadRequest};
use crate::core::precision::PrecisionPolicy;
use crate::generate;
use crate::models::common::model::ModelBackend;
use serde::{Deserialize, Serialize};

// Import our new modules
mod model_loading;
use model_loading::{gguf, hub_gguf, safetensors};
mod device;
mod template;

#[tauri::command]
pub fn load_model(state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>, req: LoadRequest) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    match req {
        LoadRequest::Gguf { model_path, tokenizer_path: _tokenizer_path, context_length, device } => {
            gguf::load_gguf_model(&mut guard, model_path, context_length, device)
        }
        LoadRequest::HubGguf { repo_id, revision, filename, context_length, device } => {
            hub_gguf::load_hub_gguf_model(&mut guard, repo_id, revision, filename, context_length, device)
        }
        LoadRequest::HubSafetensors { repo_id, revision, context_length, device } => {
            safetensors::load_hub_safetensors_model(&mut guard, repo_id, revision, context_length, device)
        }
        LoadRequest::LocalSafetensors { model_path, context_length, device } => {
            safetensors::load_local_safetensors_model(&mut guard, model_path, context_length, device)
        }
    }
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