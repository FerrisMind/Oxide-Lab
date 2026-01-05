use crate::core::state::SharedState;
use crate::core::types::GenerateRequest;
use crate::generate;
use crate::log_template;

#[tauri::command]
pub async fn generate_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState>,
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
