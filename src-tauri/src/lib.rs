pub mod core;
pub mod generate;
pub mod models;
pub mod api;
// модуль `model` удалён, всё перенесено в `models/`
// moved heavy operations to api/
use std::sync::{Arc, Mutex};
use candle::Device;
// use candle::quantized::gguf_file;
use core::state::{ModelState, SharedState};
use crate::models::common::model::ModelBackend;
// use crate::models::qwen3::ModelWeights as Qwen3Gguf;
// не импортируем типы напрямую здесь, чтобы избежать предупреждений об их неиспользовании

// типы и утилиты перенесены в core/{types,device,tokenizer}.rs

// Команды вынесены в api/mod.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared state хранит боксированную реализацию модели через trait-объект,
    // что позволяет загружать разные архитектуры GGUF под единым интерфейсом.
    let shared: SharedState<Box<dyn ModelBackend + Send>> = Arc::new(Mutex::new(ModelState::new(Device::Cpu)));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![
            api::greet,
            api::load_model,
            api::unload_model,
            api::generate_stream,
            api::cancel_generation,
            api::set_device,
            api::is_model_loaded,
            api::get_chat_template,
            api::render_prompt,
            api::get_device_info,
            api::probe_cuda,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
