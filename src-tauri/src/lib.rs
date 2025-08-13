mod token_output_stream;
mod state;
mod generate;
mod offload;
mod model;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use candle::{Device};
use candle::quantized::gguf_file;
use serde::{Deserialize, Serialize};
use state::{ModelState, SharedState};
use tokenizers::Tokenizer;
use crate::model::qwen3_offload::ModelWeights as Qwen3Gguf;

fn device_label(d: &Device) -> &'static str {
    match d {
        Device::Cpu => "CPU",
        Device::Cuda(_) => "CUDA",
        Device::Metal(_) => "Metal",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DevicePreference {
    Auto,
    Cpu,
    Cuda { index: usize },
    Metal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "lowercase")]
pub enum LoadRequest {
    // GGUF: путь к .gguf и tokenizer.json
    Gguf {
        model_path: String,
        tokenizer_path: Option<String>,
        context_length: usize,
        device: Option<DevicePreference>,
        fallback_to_cpu_on_oom: Option<bool>,
        n_gpu_layers: Option<usize>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    // Убираем max_new_tokens: длина ответа ограничивается только контекстом
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub min_p: Option<f64>,
    pub repeat_penalty: Option<f32>,
    pub repeat_last_n: usize,
    // Если false, бэкенд использует стандартные дефолты; если true — применяет только переданные параметры
    #[serde(default)]
    pub use_custom_params: bool,
}

#[tauri::command]
fn load_model(state: tauri::State<SharedState>, req: LoadRequest) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    match req {
        LoadRequest::Gguf { model_path, tokenizer_path, context_length, device, fallback_to_cpu_on_oom, n_gpu_layers } => {
            // Выбираем устройство
            let dev = match device.unwrap_or(DevicePreference::Auto) {
                DevicePreference::Auto => Device::cuda_if_available(0).unwrap_or(Device::Cpu),
                DevicePreference::Cpu => Device::Cpu,
                DevicePreference::Cuda { index } => Device::cuda_if_available(index).unwrap_or(Device::Cpu),
                DevicePreference::Metal => Device::Cpu,
            };
            guard.device = dev;
            println!("[load] device selected: {}", device_label(&guard.device));
            if let Some(f) = fallback_to_cpu_on_oom { guard.fallback_to_cpu_on_oom = f; }
            if let Some(k) = n_gpu_layers { guard.n_gpu_layers = k; }
            let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
            let content = gguf_file::Content::read(&mut file)
                .map_err(|e| format!("{}", e.with_path(PathBuf::from(model_path.clone()))))?;
            // Передаём gpu/cpu, чтобы при загрузке модель размесила веса на устройства с автоматическим fallback на CPU при OOM
            let model = Qwen3Gguf::from_gguf(
                content,
                &mut file,
                &guard.device,
                &candle::Device::Cpu,
                context_length,
                false,
            )
                .map_err(|e| e.to_string())?;

            let tp = tokenizer_path.clone();
            let tokenizer = match tp.as_ref() {
                Some(p) => Tokenizer::from_file(p).map_err(|e| e.to_string())?,
                None => return Err("tokenizer_path is required for gguf".into()),
            };

            guard.gguf_model = Some(model);
            guard.gguf_file = Some(file);
            guard.tokenizer = Some(tokenizer);
            let ctx = if context_length == 0 { 1 } else { context_length };
            guard.context_length = ctx;
            guard.model_path = Some(model_path);
            guard.tokenizer_path = tp;
            println!("[load] gguf loaded, context_length={}", guard.context_length);
        }
    }

    Ok(())
}

#[tauri::command]
fn unload_model(state: tauri::State<SharedState>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    // Сохранить базовые настройки перед полной очисткой
    let device = guard.device.clone();
    let fallback = guard.fallback_to_cpu_on_oom;
    let n_gpu_layers = guard.n_gpu_layers;

    // Явно освободить тяжёлые поля
    guard.gguf_model = None; // дроп модели (освобождение VRAM через Drop)
    guard.gguf_file = None;
    guard.tokenizer = None;

    // Полный сброс состояния, чтобы гарантированно освободить любые косвенные держатели
    *guard = ModelState::new(device);
    guard.fallback_to_cpu_on_oom = fallback;
    guard.n_gpu_layers = n_gpu_layers;
    println!("[unload] hard reset: freed model/tokenizer and reset state (preserved device, fallback, n_gpu_layers)");
    Ok(())
}

#[tauri::command]
async fn generate_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState>,
    req: GenerateRequest,
) -> Result<(), String> {
    generate::generate_stream_cmd(app, state, req).await
}

#[tauri::command]
fn cancel_generation() -> Result<(), String> {
    generate::cancel_generation_cmd()
}

#[tauri::command]
fn is_model_loaded(state: tauri::State<SharedState>) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.gguf_model.is_some() && guard.tokenizer.is_some())
}

#[tauri::command]
fn set_device(state: tauri::State<SharedState>, pref: DevicePreference) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let dev = match pref {
        DevicePreference::Auto => Device::cuda_if_available(0).unwrap_or(Device::Cpu),
        DevicePreference::Cpu => Device::Cpu,
        DevicePreference::Cuda { index } => Device::cuda_if_available(index).unwrap_or(Device::Cpu),
        DevicePreference::Metal => Device::Cpu,
    };
    guard.device = dev;
    println!("[device] switched");
    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: SharedState = Arc::new(Mutex::new(ModelState::new(Device::Cpu)));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![greet, load_model, unload_model, generate_stream, cancel_generation, set_device, is_model_loaded])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
