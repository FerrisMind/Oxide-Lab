use crate::core::state::{ModelState, SharedState};

use tauri::AppHandle;

pub(crate) fn default_rayon_thread_limit() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    cpus.saturating_sub(1).max(1)
}

#[tauri::command]
pub fn get_rayon_thread_limit(app: AppHandle) -> Result<Option<usize>, String> {
    ModelState::load_thread_limit(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_rayon_thread_limit(
    app: AppHandle,
    state: tauri::State<'_, SharedState>,
    limit: Option<usize>,
) -> Result<(), String> {
    // Applied on next startup because Rayon global pool cannot be reconfigured at runtime.
    let normalized = limit.map(|value| value.max(1));
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.rayon_thread_limit = normalized;
    ModelState::save_thread_limit(&app, normalized).map_err(|e| e.to_string())
}
