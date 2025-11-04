use crate::core::state::{ModelState, SharedState};
use crate::core::types::LoadRequest;
use crate::generate::cancel::{CANCEL_LOADING, cancel_model_loading_cmd};
use crate::log_load;
use crate::models::common::model::ModelBackend;

pub fn clone_state_arc(
    state: &tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> SharedState<Box<dyn ModelBackend + Send>> {
    state.inner().clone()
}

#[tauri::command]
pub fn load_model(
    app: tauri::AppHandle,
    state: tauri::State<SharedState<Box<dyn ModelBackend + Send>>>,
    req: LoadRequest,
) -> Result<(), String> {
    CANCEL_LOADING.store(false, std::sync::atomic::Ordering::SeqCst);

    let app_clone = app.clone();
    let state_arc = clone_state_arc(&state);
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
                tokenizer_path: _,
                context_length,
                device,
            } => crate::api::model_loading::gguf::load_gguf_model(
                &app_clone,
                &mut guard,
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
            } => crate::api::model_loading::safetensors::load_hub_safetensors_model(
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
            } => crate::api::model_loading::safetensors::load_local_safetensors_model(
                &app_clone,
                &mut guard,
                model_path,
                context_length,
                device,
            ),
        };

        if let Err(ref e) = res {
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
    let state_arc = clone_state_arc(&state);
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
            Some("complete"),
            true,
            None,
        );
        log_load!("hard reset: freed model/tokenizer and reset state (preserved device)");
    });

    Ok(())
}

#[tauri::command]
pub fn is_model_loaded(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.gguf_model.is_some() && guard.tokenizer.is_some())
}
