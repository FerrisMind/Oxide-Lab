use std::sync::{Arc, Mutex};

use crate::api::commands::threads::apply_rayon_thread_limit;
use crate::core::device::select_device;
use crate::core::performance::StartupTracker;
use crate::core::state::{ModelState, SharedState};
use crate::core::types::DevicePreference;
use crate::models::common::model::ModelBackend;
use tauri::{Emitter, Manager};

#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "authors": env!("CARGO_PKG_AUTHORS"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
    })
}

fn build_shared_state() -> SharedState<Box<dyn ModelBackend + Send>> {
    let initial_device = select_device(Some(DevicePreference::Auto));
    Arc::new(Mutex::new(ModelState::new(initial_device)))
}

fn spawn_startup_tracker(
    app_handle: tauri::AppHandle,
    performance_monitor: Arc<crate::core::performance::PerformanceMonitor>,
) {
    tauri::async_runtime::spawn(async move {
        let mut tracker = StartupTracker::new(performance_monitor).await;

        tracker.stage_completed("tauri_init");
        tracker.stage_completed("plugins_init");
        tracker.stage_completed("state_init");

        let startup_metrics = tracker.finish().await;

        if let Err(e) = app_handle.emit("startup_metrics", &startup_metrics) {
            eprintln!("Failed to emit startup metrics: {e}");
        }

        println!(
            "Application startup completed in {} ms",
            startup_metrics.total_duration_ms
        );
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared = build_shared_state();
    let performance_monitor = {
        let guard = shared.lock().expect("Failed to lock shared state");
        guard.performance_monitor.clone()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(shared.clone())
        .invoke_handler(tauri::generate_handler![
            crate::api::greet,
            get_app_info,
            crate::api::load_model,
            crate::api::unload_model,
            crate::api::cancel_model_loading,
            crate::api::generate_stream,
            crate::api::cancel_generation,
            crate::api::set_device,
            crate::api::is_model_loaded,
            crate::api::get_chat_template,
            crate::api::render_prompt,
            crate::api::get_device_info,
            crate::api::probe_cuda,
            crate::api::get_precision_policy,
            crate::api::set_precision_policy,
            crate::api::get_precision,
            crate::api::set_precision,
            crate::api::get_rayon_thread_limit,
            crate::api::set_rayon_thread_limit,
            crate::api::gguf_list_metadata_keys_from_path,
            crate::api::gguf_list_metadata_keys,
            crate::api::get_experimental_features_enabled,
            crate::api::set_experimental_features_enabled,
            crate::api::performance_api::get_performance_metrics,
            crate::api::performance_api::get_average_duration,
            crate::api::performance_api::get_memory_usage,
            crate::api::performance_api::clear_performance_metrics,
            crate::api::performance_api::get_startup_metrics,
            crate::api::performance_api::get_system_usage,
            crate::api::local_models::parse_gguf_metadata,
            crate::api::local_models::scan_models_folder,
            crate::api::local_models::scan_local_models_folder,
            crate::api::local_models::search_huggingface_gguf,
            crate::api::local_models::download_hf_model_file,
            crate::api::local_models::get_model_readme,
            crate::api::local_models::delete_local_model,
            crate::api::local_models::update_model_manifest,
            crate::api::model_cards::get_model_cards,
            crate::api::model_cards::import_model_cards,
            crate::api::model_cards::reset_model_cards,
            crate::api::model_cards::download_model_card_format,
            crate::api::download_manager::start_model_download,
            crate::api::download_manager::get_downloads_snapshot,
            crate::api::download_manager::pause_download,
            crate::api::download_manager::resume_download,
            crate::api::download_manager::cancel_download,
            crate::api::download_manager::remove_download_entry,
            crate::api::download_manager::clear_download_history,
        ])
        .setup(move |app| {
            let handle = app.handle();
            match ModelState::<Box<dyn ModelBackend + Send>>::load_thread_limit(handle) {
                Ok(limit) => {
                    apply_rayon_thread_limit(limit);
                    if let Ok(mut guard) = shared.lock() {
                        guard.rayon_thread_limit = limit;
                    }
                }
                Err(err) => {
                    eprintln!("Failed to load saved Rayon thread limit: {}", err);
                }
            }
            spawn_startup_tracker(app.handle().clone(), performance_monitor.clone());
            #[cfg(debug_assertions)]
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
