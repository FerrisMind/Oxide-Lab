pub mod gguf;
pub mod hub_gguf;
pub mod safetensors;

use serde::Serialize;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize)]
pub struct LoadProgressEvent {
    pub stage: String,
    pub progress: u8,
    pub message: Option<String>,
    pub done: bool,
    pub error: Option<String>,
}

pub fn emit_load_progress(
    app: &tauri::AppHandle,
    stage: &str,
    progress: u8,
    message: Option<&str>,
    done: bool,
    error: Option<&str>,
) {
    let payload = LoadProgressEvent {
        stage: stage.to_string(),
        progress,
        message: message.map(|s| s.to_string()),
        done,
        error: error.map(|s| s.to_string()),
    };
    let _ = app.emit("load_progress", payload);
}
