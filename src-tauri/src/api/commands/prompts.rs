use crate::core::state::SharedState;
use crate::models::ModelBackend;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsgDto {
    pub role: String,
    pub content: String,
}

#[tauri::command]
pub fn get_chat_template(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Option<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.chat_template.clone())
}

#[tauri::command]
pub fn render_prompt(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    messages: Vec<ChatMsgDto>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    crate::api::template::render_prompt(&guard.chat_template, messages)
}
