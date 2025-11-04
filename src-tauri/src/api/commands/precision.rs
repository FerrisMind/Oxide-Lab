use crate::core::precision::{Precision, PrecisionPolicy};
use crate::core::state::{ModelState, SharedState};
use crate::models::common::model::ModelBackend;

#[tauri::command]
pub fn get_precision_policy(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<PrecisionPolicy, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.precision_policy.clone())
}

#[tauri::command]
pub fn set_precision_policy(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    policy: PrecisionPolicy,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.precision_policy = policy;
    Ok(())
}

#[tauri::command]
pub fn get_precision(
    app: tauri::AppHandle,
    _state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Precision, String> {
    let precision = ModelState::<Box<dyn ModelBackend + Send>>::load_precision(&app)
        .map_err(|e| e.to_string())?;
    Ok(precision)
}

#[tauri::command]
pub fn set_precision(
    app: tauri::AppHandle,
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
    precision_str: String,
) -> Result<(), String> {
    let precision = match precision_str.as_str() {
        "f16" => Precision::F16,
        "f32" => Precision::F32,
        "int8" => Precision::Int8,
        _ => return Err("Invalid precision: must be 'f16', 'f32', or 'int8'".to_string()),
    };

    let guard = state.lock().map_err(|e| e.to_string())?;
    if matches!(precision, Precision::Int8) && !matches!(guard.device, candle::Device::Cpu) {
        return Err("Int8 precision only supported on CPU".to_string());
    }
    drop(guard);

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.precision_policy = match precision {
        Precision::F16 => PrecisionPolicy::MemoryEfficient,
        Precision::F32 => PrecisionPolicy::Default,
        Precision::Int8 => PrecisionPolicy::MemoryEfficient,
    };
    guard.save_precision(&app).map_err(|e| e.to_string())
}
