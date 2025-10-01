//! Local models management API
//!
//! This module provides functionality for scanning, managing and extracting metadata
//! from local model files (GGUF and Safetensors formats).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use candle::quantized::gguf_file;
use regex::Regex;

/// Information about a local model file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    /// Full path to the model file
    pub path: String,
    
    /// Model name (filename without extension)
    pub name: String,
    
    /// Model architecture (e.g., "llama", "gpt2", "bert")
    pub architecture: Option<String>,
    
    /// Number of parameters (e.g., "7B", "13B")
    pub parameters: Option<String>,
    
    /// Model author/publisher
    pub author: Option<String>,
    
    /// Quantization level (e.g., "Q4_K_M", "Q5_K_S", "F16")
    pub quantization: Option<String>,
    
    /// File size in bytes
    pub size_bytes: u64,
    
    /// File format ("gguf" or "safetensors")
    pub format: String,
    
    /// Last modified timestamp (Unix epoch)
    pub last_modified: u64,
}

/// Extract metadata from a GGUF file
fn extract_gguf_metadata(path: &Path) -> Result<LocalModelInfo, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open GGUF file: {}", e))?;
    
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read GGUF file: {}", e))?;
    
    let mut cursor = std::io::Cursor::new(&buf);
    let content = gguf_file::Content::read(&mut cursor)
        .map_err(|e| format!("Failed to parse GGUF file: {}", e))?;
    
    // Extract metadata from GGUF
    let architecture = content.metadata.get("general.architecture")
        .and_then(|v| match v {
            gguf_file::Value::String(s) => Some(s.clone()),
            _ => None,
        });
    
    let name = content.metadata.get("general.name")
        .and_then(|v| match v {
            gguf_file::Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    let author = content.metadata.get("general.author")
        .and_then(|v| match v {
            gguf_file::Value::String(s) => Some(s.clone()),
            _ => None,
        });
    
    // Try to get parameter count from metadata
    let parameters = content.metadata.get("general.parameter_count")
        .and_then(|v| match v {
            gguf_file::Value::U64(n) => Some(format_parameter_count(*n)),
            gguf_file::Value::U32(n) => Some(format_parameter_count(*n as u64)),
            _ => None,
        });
    
    // Extract quantization from filename
    let quantization = extract_quantization_from_filename(
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    );
    
    // Get file metadata
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let size_bytes = metadata.len();
    let last_modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to convert time: {}", e))?
        .as_secs();
    
    Ok(LocalModelInfo {
        path: path.to_string_lossy().to_string(),
        name,
        architecture,
        parameters,
        author,
        quantization,
        size_bytes,
        format: "gguf".to_string(),
        last_modified,
    })
}

/// Extract metadata from a Safetensors file
fn extract_safetensors_metadata(path: &Path) -> Result<LocalModelInfo, String> {
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Try to find config.json in the same directory
    let model_dir = path.parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    
    let config_path = model_dir.join("config.json");
    
    let (architecture, parameters, author) = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(config_str) => {
                match serde_json::from_str::<serde_json::Value>(&config_str) {
                    Ok(config) => {
                        let arch = config.get("architectures")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str())
                            .or_else(|| {
                                config.get("model_type")
                                    .and_then(|v| v.as_str())
                            })
                            .map(|s| s.to_string());
                        
                        let params = config.get("num_parameters")
                            .and_then(|v| v.as_u64())
                            .map(format_parameter_count);
                        
                        let auth = config.get("author")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        (arch, params, auth)
                    }
                    Err(_) => (None, None, None),
                }
            }
            Err(_) => (None, None, None),
        }
    } else {
        (None, None, None)
    };
    
    // Extract quantization from filename
    let quantization = extract_quantization_from_filename(
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    );
    
    // Get file metadata
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    let size_bytes = metadata.len();
    let last_modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to convert time: {}", e))?
        .as_secs();
    
    Ok(LocalModelInfo {
        path: path.to_string_lossy().to_string(),
        name,
        architecture,
        parameters,
        author,
        quantization,
        size_bytes,
        format: "safetensors".to_string(),
        last_modified,
    })
}

/// Extract quantization information from filename
fn extract_quantization_from_filename(filename: &str) -> Option<String> {
    // Regex to match quantization patterns: Q4_K_M, Q5_K_S, Q8_0, F16, F32, etc.
    let re = Regex::new(r"(Q[0-9]_[KMS](?:_[A-Z])?|F16|F32|BF16)").ok()?;
    
    re.captures(filename)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Format parameter count into human-readable string
fn format_parameter_count(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.1}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// Recursively scan a directory for model files
fn scan_directory(dir: &Path, models: &mut Vec<LocalModelInfo>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry
            .map_err(|e| format!("Failed to read directory entry: {}", e))?;
        
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively scan subdirectories
            if let Err(e) = scan_directory(&path, models) {
                eprintln!("Warning: {}", e);
            }
        } else if path.is_file() {
            // Check file extension
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                
                match ext_lower.as_str() {
                    "gguf" => {
                        match extract_gguf_metadata(&path) {
                            Ok(info) => models.push(info),
                            Err(e) => eprintln!("Warning: Failed to extract GGUF metadata from {}: {}", 
                                path.display(), e),
                        }
                    }
                    "safetensors" => {
                        match extract_safetensors_metadata(&path) {
                            Ok(info) => models.push(info),
                            Err(e) => eprintln!("Warning: Failed to extract Safetensors metadata from {}: {}", 
                                path.display(), e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

/// Tauri command: Scan a folder for local models
#[tauri::command]
pub fn scan_local_models_folder(folder_path: String) -> Result<Vec<LocalModelInfo>, String> {
    let path = PathBuf::from(&folder_path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    let mut models = Vec::new();
    scan_directory(&path, &mut models)?;
    
    // Sort models by name
    models.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(models)
}

/// Tauri command: Delete a local model file
#[tauri::command]
pub fn delete_local_model(model_path: String) -> Result<(), String> {
    let path = PathBuf::from(&model_path);
    
    if !path.exists() {
        return Err(format!("File does not exist: {}", model_path));
    }
    
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", model_path));
    }
    
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_parameter_count() {
        assert_eq!(format_parameter_count(1_500_000_000), "1.5B");
        assert_eq!(format_parameter_count(7_000_000_000), "7.0B");
        assert_eq!(format_parameter_count(500_000_000), "500.0M");
        assert_eq!(format_parameter_count(1_500_000), "1.5M");
        assert_eq!(format_parameter_count(500), "500");
    }
    
    #[test]
    fn test_extract_quantization_from_filename() {
        assert_eq!(
            extract_quantization_from_filename("llama-7b-Q4_K_M.gguf"),
            Some("Q4_K_M".to_string())
        );
        assert_eq!(
            extract_quantization_from_filename("model-Q5_K_S.gguf"),
            Some("Q5_K_S".to_string())
        );
        assert_eq!(
            extract_quantization_from_filename("model-F16.safetensors"),
            Some("F16".to_string())
        );
        assert_eq!(
            extract_quantization_from_filename("model.gguf"),
            None
        );
    }
}

