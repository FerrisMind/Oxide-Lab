use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

use crate::api::commands::threads::default_rayon_thread_limit;
use crate::core::audio_capture::AudioCaptureState;
use crate::core::performance::StartupTracker;
use crate::core::rayon_pool::init_global_low_priority_pool;
use crate::core::state::{ModelState, SharedState};
use crate::core::thread_priority::set_current_thread_above_normal;
use crate::log_load_warn;

use tauri_plugin_sql::{Builder, Migration, MigrationKind};

#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "authors": env!("CARGO_PKG_AUTHORS"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
    })
}

fn build_shared_state() -> SharedState {
    Arc::new(Mutex::new(ModelState::new()))
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

    let migrations = vec![
        Migration {
            version: 1,
            description: "create sessions and messages tables",
            sql: "
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    model_path TEXT,
                    repo_id TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);
                
                CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
                    content TEXT NOT NULL DEFAULT '',
                    created_at INTEGER NOT NULL,
                    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            ",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add thinking column to messages",
            sql: "ALTER TABLE messages ADD COLUMN thinking TEXT NOT NULL DEFAULT '';",
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            Builder::default()
                .add_migrations("sqlite:chat_history.db", migrations)
                .build(),
        )
        .manage(shared.clone())
        .manage(AudioCaptureState::new())
        .manage(crate::engines::EngineManager::new())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            // Compatibility commands used by frontend services/pages
            crate::api::get_server_config,
            crate::api::get_experimental_features_enabled,
            crate::api::set_experimental_features_enabled,
            crate::api::get_prefix_cache_info,
            crate::api::set_prefix_cache_enabled,
            crate::api::clear_prefix_cache,
            crate::api::start_voice_recording,
            crate::api::stop_voice_recording_and_transcribe,
            crate::api::cancel_voice_recording,
            crate::api::get_stt_settings,
            crate::api::set_stt_settings,
            crate::api::download_stt_model,
            crate::api::parse_gguf_metadata,
            crate::api::scan_models_folder,
            crate::api::delete_local_model,
            crate::api::update_model_manifest,
            crate::api::search_huggingface_gguf,
            crate::api::get_model_readme,
            crate::api::get_model_cards,
            crate::api::import_model_cards,
            crate::api::reset_model_cards,
            crate::api::download_model_card_format,
            // Threads
            crate::api::get_rayon_thread_limit,
            crate::api::set_rayon_thread_limit,
            // Locale
            crate::api::get_locale,
            crate::api::set_locale,
            // Performance
            crate::api::performance_api::get_performance_metrics,
            crate::api::performance_api::get_average_duration,
            crate::api::performance_api::get_memory_usage,
            crate::api::performance_api::clear_performance_metrics,
            crate::api::performance_api::get_startup_metrics,
            crate::api::performance_api::get_system_usage,
            // Download manager
            crate::api::download_manager::start_model_download,
            crate::api::download_manager::get_downloads_snapshot,
            crate::api::download_manager::pause_download,
            crate::api::download_manager::resume_download,
            crate::api::download_manager::cancel_download,
            crate::api::download_manager::remove_download_entry,
            crate::api::download_manager::clear_download_history,
            // Engine commands (new architecture)
            crate::engines::list_engines,
            crate::engines::start_engine,
            crate::engines::stop_engine,
            crate::engines::generate,
            crate::engines::cancel_generation,
            crate::engines::install_engine_cmd,
            crate::engines::uninstall_engine_cmd,
        ])
        .setup(move |app| {
            // Initialize Engine Manager
            let engine_manager: tauri::State<crate::engines::EngineManager> = app.state();
            tauri::async_runtime::block_on(async {
                if let Err(e) = engine_manager.load_definitions().await {
                    log::error!("Failed to load engine definitions: {}", e);
                }
            });

            let _ = set_current_thread_above_normal();

            let handle = app.handle();
            match ModelState::load_thread_limit(handle) {
                Ok(limit) => {
                    let effective_limit = limit.or_else(|| Some(default_rayon_thread_limit()));
                    if let Some(threads) = effective_limit {
                        match init_global_low_priority_pool(threads) {
                            Ok(true) => {}
                            Ok(false) => log_load_warn!("global rayon pool already initialized"),
                            Err(e) => log_load_warn!("failed to init global rayon pool: {}", e),
                        }
                    }
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
