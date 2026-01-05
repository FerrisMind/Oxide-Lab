use crate::core::background_mode::BackgroundModeGuard;
use crate::core::state::{ModelState, SharedState};
use crate::core::types::LoadRequest;
use crate::generate::cancel::{CANCEL_LOADING, cancel_model_loading_cmd};
use crate::log_load;
use crate::log_load_warn;

use std::sync::Arc;

pub fn clone_state_arc(state: &tauri::State<'_, SharedState>) -> SharedState {
    state.inner().clone()
}

fn snapshot_for_loading(
    guard: &ModelState,
) -> (
    candle::Device,
    crate::core::precision::PrecisionPolicy,
    Option<usize>,
    Arc<crate::core::performance::PerformanceMonitor>,
) {
    (
        guard.device.clone(),
        guard.precision_policy.clone(),
        guard.rayon_thread_limit,
        guard.performance_monitor.clone(),
    )
}

#[tauri::command]
pub async fn load_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState>,
    req: LoadRequest,
) -> Result<(), String> {
    CANCEL_LOADING.store(false, std::sync::atomic::Ordering::SeqCst);

    let app_clone = app.clone();
    let state_arc = clone_state_arc(&state);
    // Important: return to the WebView immediately (do not await model loading).
    // If the IPC call stays pending for seconds, WebView2 can show "busy"/ghosting even though
    // the heavy work happens on a background thread.
    tauri::async_runtime::spawn(async move {
        let app_for_blocking = app_clone.clone();
        let join_res = tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
            // Hybrid approach:
            // - keep process priority normal for speed
            // - keep CPU worker threads below-normal (global Rayon pool handler)
            // - do NOT hold the global model-state mutex during heavy loading, so UI can still call backend commands
            //
            // Optional: enable Windows process background mode if you want maximum UI smoothness at the cost of load speed.
            // OXIDE_WIN_BACKGROUND_MODE=1
            let _bg = std::env::var("OXIDE_WIN_BACKGROUND_MODE")
                .ok()
                .is_some_and(|v| v == "1")
                .then(BackgroundModeGuard::new);

            let (device, precision_policy, rayon_thread_limit, performance_monitor) = {
                let guard = match state_arc.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        crate::api::model_loading::emit_load_progress(
                            &app_for_blocking,
                            "error",
                            0,
                            None,
                            true,
                            Some(&e.to_string()),
                        );
                        return Err(e.to_string());
                    }
                };
                snapshot_for_loading(&guard)
            };

            let mut next_state = ModelState::new(device);
            next_state.precision_policy = precision_policy;
            next_state.rayon_thread_limit = rayon_thread_limit;
            next_state.performance_monitor = performance_monitor;

            let res: Result<(), String> = match req {
                LoadRequest::Gguf {
                    model_path,
                    tokenizer_path: _,
                    context_length,
                    device,
                } => crate::api::model_loading::gguf::load_gguf_model(
                    &app_for_blocking,
                    &mut next_state,
                    model_path,
                    context_length,
                    device,
                ),
                LoadRequest::HubGguf {
                    repo_id,
                    revision,
                    filename,
                    context_length,
                    device,
                } => crate::api::model_loading::hub_gguf::load_hub_gguf_model(
                    &app_for_blocking,
                    &mut next_state,
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
                } => crate::api::model_loading::safetensors::load_hub_safetensors_model(
                    &app_for_blocking,
                    &mut next_state,
                    repo_id,
                    revision,
                    context_length,
                    device,
                ),
                LoadRequest::LocalSafetensors {
                    model_path,
                    context_length,
                    device,
                } => crate::api::model_loading::safetensors::load_local_safetensors_model(
                    &app_for_blocking,
                    &mut next_state,
                    model_path,
                    context_length,
                    device,
                ),
            };

            if res.is_ok() {
                match state_arc.lock() {
                    Ok(mut guard) => {
                        *guard = next_state;
                    }
                    Err(e) => {
                        log_load_warn!("failed to commit loaded model state: {}", e);
                    }
                }
            }

            if let Err(ref e) = res {
                crate::api::model_loading::emit_load_progress(
                    &app_for_blocking,
                    "error",
                    0,
                    None,
                    true,
                    Some(e),
                );
            }
            res
        })
        .await;

        if let Err(join_err) = join_res {
            crate::api::model_loading::emit_load_progress(
                &app_clone,
                "error",
                0,
                None,
                true,
                Some(&format!("load_model join error: {}", join_err)),
            );
        }
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_model_loading() -> Result<(), String> {
    cancel_model_loading_cmd()
}

#[tauri::command]
pub async fn unload_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    let app_clone = app.clone();
    let state_arc = clone_state_arc(&state);
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
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
                return Err(e.to_string());
            }
        };
        let device = guard.device.clone();

        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_start",
            0,
            None,
            false,
            None,
        );
        guard.scheduler.unload_model();
        crate::api::model_loading::emit_load_progress(
            &app_clone,
            "unload_model",
            40,
            None,
            false,
            None,
        );
        // guard.gguf_file удалено
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
            Some("complete"),
            true,
            None,
        );
        log_load!("hard reset: freed model/tokenizer and reset state (preserved device)");
        Ok(())
    })
    .await
    .map_err(|e| format!("unload_model join error: {}", e))?
}

#[tauri::command]
pub fn is_model_loaded(state: tauri::State<'_, SharedState>) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.scheduler.has_model() && guard.tokenizer.is_some())
}
