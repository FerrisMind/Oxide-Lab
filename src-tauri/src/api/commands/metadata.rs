use crate::core::state::SharedState;
use crate::models::ModelBackend;
use candle::quantized::gguf_file;
use std::path::Path;

#[tauri::command]
pub fn gguf_list_metadata_keys_from_path(path: String) -> Result<Vec<String>, String> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err(format!("Not a file: {path}"));
    }
    if !p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
    {
        return Err("Path is not a .gguf file".to_string());
    }
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut f).map_err(|e| e.to_string())?;
    let mut keys: Vec<String> = content.metadata.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}

#[tauri::command]
pub fn gguf_list_metadata_keys(
    state: tauri::State<'_, SharedState<Box<dyn ModelBackend + Send>>>,
) -> Result<Vec<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let path_str = guard
        .model_path
        .as_ref()
        .ok_or_else(|| "No model loaded".to_string())?;
    let p = Path::new(path_str);
    if !p.is_file() {
        return Err(format!("Not a file: {path_str}"));
    }
    if !p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
    {
        return Err("Loaded model is not a .gguf file".to_string());
    }
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut f).map_err(|e| e.to_string())?;
    let mut keys: Vec<String> = content.metadata.keys().cloned().collect();
    keys.sort();
    Ok(keys)
}
