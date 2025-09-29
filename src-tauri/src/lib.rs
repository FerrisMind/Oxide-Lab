// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Use auto-selection for initial device instead of hardcoding CPU
use crate::core::device::select_device;
use crate::core::types::DevicePreference;

pub mod api;
pub mod core;
pub mod generate;
pub mod models;
// модуль `model` удалён, всё перенесено в `models/`
// moved heavy operations to api/
use std::sync::{Arc, Mutex};
// use candle::quantized::gguf_file;
use crate::models::common::model::ModelBackend;
use core::state::{ModelState, SharedState};
// use crate::models::qwen3::ModelWeights as Qwen3Gguf;
// не импортируем типы напрямую здесь, чтобы избежать предупреждений об их неиспользовании

// типы и утилиты перенесены в core/{types,device,tokenizer}.rs

// Команды вынесены в api/mod.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared state хранит боксированную реализацию модели через trait-объект,
    // что позволяет загружать разные архитектуры GGUF под единым интерфейсом.
    // Use auto-selection for initial device instead of hardcoding CPU
    let initial_device = select_device(Some(DevicePreference::Auto));
    let shared: SharedState<Box<dyn ModelBackend + Send>> =
        Arc::new(Mutex::new(ModelState::new(initial_device)));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![
            api::greet,
            api::load_model,
            api::unload_model,
            api::cancel_model_loading,
            api::generate_stream,
            api::cancel_generation,
            api::set_device,
            api::is_model_loaded,
            api::get_chat_template,
            api::render_prompt,
            api::get_device_info,
            api::probe_cuda,
            api::get_precision_policy,
            api::set_precision_policy,
            api::get_precision,
            api::set_precision,
            api::gguf_list_metadata_keys_from_path,
            api::gguf_list_metadata_keys,
            api::performance_api::get_performance_metrics,
            api::performance_api::get_average_duration,
            api::performance_api::get_memory_usage,
            api::performance_api::clear_performance_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
