/*!
 * Locale Commands for Tauri
 */

/// Get current locale
#[tauri::command]
pub fn get_locale() -> String {
    "en".to_string()
}

/// Set locale (stub — locale system removed during candle cleanup)
#[tauri::command]
pub fn set_locale(_locale: String) -> Result<(), String> {
    Ok(())
}
