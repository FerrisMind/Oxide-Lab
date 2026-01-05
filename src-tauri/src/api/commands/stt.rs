use tauri::{AppHandle, State};

use crate::core::audio_capture::{AudioCaptureState, resample_linear};
use crate::core::stt_whisper::{DownloadRequest, DownloadResponse, TranscribeRequest};
use crate::core::types::SttSettings;

const TARGET_SAMPLE_RATE: u32 = 16_000;

#[tauri::command]
pub async fn transcribe_audio(app: AppHandle, req: TranscribeRequest) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || crate::core::stt_whisper::transcribe(&app, req))
        .await
        .map_err(|e| format!("STT task failed: {e}"))?
}

#[tauri::command]
pub fn start_voice_recording(
    app: AppHandle,
    state: State<'_, AudioCaptureState>,
) -> Result<(), String> {
    log::info!("Starting voice recording via Tauri command");
    state.start(app)
}

#[tauri::command]
pub fn cancel_voice_recording(state: State<'_, AudioCaptureState>) -> Result<(), String> {
    state.cancel()
}

#[tauri::command]
pub async fn stop_voice_recording_and_transcribe(
    app: AppHandle,
    state: State<'_, AudioCaptureState>,
    language: Option<String>,
) -> Result<String, String> {
    log::info!("Stopping voice recording and transcribing...");
    let (samples, sample_rate) = state.stop()?;
    log::info!("Captured {} samples at {} Hz", samples.len(), sample_rate);
    let samples = if sample_rate == TARGET_SAMPLE_RATE {
        samples
    } else {
        resample_linear(&samples, sample_rate, TARGET_SAMPLE_RATE)
    };
    let req = TranscribeRequest {
        samples,
        sample_rate: TARGET_SAMPLE_RATE,
        language,
    };
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
