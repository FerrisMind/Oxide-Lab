use tauri::AppHandle;

use crate::core::stt_whisper::{DownloadRequest, DownloadResponse, TranscribeRequest};
use crate::core::types::SttSettings;

#[tauri::command]
pub async fn transcribe_audio(app: AppHandle, req: TranscribeRequest) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || crate::core::stt_whisper::transcribe(&app, req))
        .await
        .map_err(|e| format!("STT task failed: {e}"))?
}

#[tauri::command]
pub fn get_stt_settings(app: AppHandle) -> Result<SttSettings, String> {
    crate::core::stt_whisper::load_settings(&app)
}

#[tauri::command]
pub fn set_stt_settings(app: AppHandle, settings: SttSettings) -> Result<(), String> {
    crate::core::stt_whisper::save_settings(&app, &settings)
}

#[tauri::command]
pub async fn download_stt_model(
    app: AppHandle,
    req: DownloadRequest,
) -> Result<DownloadResponse, String> {
    crate::core::stt_whisper::download_model(&app, req).await
}
