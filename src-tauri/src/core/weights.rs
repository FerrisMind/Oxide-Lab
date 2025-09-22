//! Universal model weights loader and VarBuilder utilities.
//!
//! This module provides unified functionality for:
//! - Loading safetensors files from Hugging Face Hub or local paths
//! - Building VarBuilder instances from safetensors files
//! - Handling both sharded and single-file model weights
//! - Unified dtype policy for model loading (delegated to core::precision)
//! - Error handling and validation utilities

use crate::core::precision::{self, PrecisionPolicy};
use crate::{log_hub, log_hub_error, log_local_error, log_validate};
use candle::Device;
use candle_nn::VarBuilder;
use std::collections::HashSet;
use std::path::Path;

/// Lists safetensors files from a Hugging Face Hub repository.
///
/// This function will look for `model.safetensors.index.json` first, and if not found,
/// will try to find a single `model.safetensors` file.
///
/// # Arguments
/// * `api` - The Hugging Face API repository handle
///
/// # Returns
/// * `Ok(Vec<String>)` - List of safetensors filenames
/// * `Err(String)` - Error message if no safetensors files are found
pub fn hub_list_safetensors(api: &hf_hub::api::sync::ApiRepo) -> Result<Vec<String>, String> {
    let mut safetensors_files: Vec<String> = Vec::new();

    // Try to load index file first (sharded model)
    if let Ok(index_path) = api.get("model.safetensors.index.json") {
        match std::fs::read(&index_path) {
            Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(value) => {
                    if let Some(files) = value.get("weight_map").and_then(|m| m.as_object()) {
                        let mut set = HashSet::new();
                        for (_k, v) in files.iter() {
                            if let Some(f) = v.as_str() {
                                set.insert(f.to_string());
                            }
                        }
                        safetensors_files.extend(set);
                    }
                }
                Err(e) => {
                    log_hub_error!("Failed to parse model.safetensors.index.json: {}", e);
                }
            },
            Err(e) => {
                log_hub_error!("Failed to read model.safetensors.index.json: {}", e);
            }
        }
    }

    // If no index file or empty, try single file
    if safetensors_files.is_empty() {
        match api.get("model.safetensors") {
            Ok(_) => {
                safetensors_files.push("model.safetensors".to_string());
            }
            Err(e) => {
                log_hub_error!("model.safetensors not found: {}", e);
            }
        }
    }

    if safetensors_files.is_empty() {
        return Err("No safetensors files found (model.safetensors[.index.json])".into());
    }

    Ok(safetensors_files)
}

/// Lists safetensors files from a local directory.
///
/// This function will look for `model.safetensors.index.json` first, and if not found,
/// will try to find a single `model.safetensors` file.
///
/// # Arguments
/// * `path` - Path to the local directory containing model files
///
/// # Returns
/// * `Ok(Vec<String>)` - List of safetensors file paths
/// * `Err(String)` - Error message if no safetensors files are found
pub fn local_list_safetensors<P: AsRef<Path>>(path: P) -> Result<Vec<String>, String> {
    let path = path.as_ref();

    // Check if path exists
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    // Check if path is a directory
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let mut safetensors_files: Vec<String> = Vec::new();

    // Try to load index file first (sharded model)
    let index_path = path.join("model.safetensors.index.json");
    if index_path.exists() {
        match std::fs::read(&index_path) {
            Ok(bytes) => {
                match serde_json::from_slice::<serde_json::Value>(&bytes) {
                    Ok(value) => {
                        if let Some(files) = value.get("weight_map").and_then(|m| m.as_object()) {
                            let mut set = HashSet::new();
                            for (_k, v) in files.iter() {
                                if let Some(f) = v.as_str() {
                                    // Validate that the file actually exists
                                    let file_path = path.join(f);
                                    if file_path.exists() {
                                        set.insert(f.to_string());
                                    } else {
                                        log_local_error!(
                                            "Referenced safetensors file not found: {}",
                                            file_path.display()
                                        );
                                    }
                                }
                            }
                            safetensors_files.extend(set);
                        }
                    }
                    Err(e) => {
                        log_local_error!("Failed to parse model.safetensors.index.json: {}", e);
                    }
                }
            }
            Err(e) => {
                log_local_error!("Failed to read model.safetensors.index.json: {}", e);
            }
        }
    }

    // If no index file or empty, try single file
    if safetensors_files.is_empty() {
        let single_path = path.join("model.safetensors");
        if single_path.exists() {
            safetensors_files.push("model.safetensors".to_string());
        }
    }

    if safetensors_files.is_empty() {
        return Err("No safetensors files found (model.safetensors[.index.json])".into());
    }

    // Return full paths
    let full_paths: Vec<String> = safetensors_files
        .iter()
        .map(|f| path.join(f).to_string_lossy().to_string())
        .collect();

    Ok(full_paths)
}

///   Builds a VarBuilder from safetensors files with unified dtype policy.
///
/// This function handles loading multiple safetensors files and applies a consistent
/// dtype conversion policy across all tensors. It's designed to work with models
/// that may have tensors saved in different precisions (f32, f16, bf16, etc.)
/// but need to be loaded with a unified dtype for inference or training.
///
/// # Arguments
///
/// * `paths` - Vector of paths to safetensors files to load
/// * `device` - Target device for the tensors (CPU, CUDA, etc.)
/// * `policy` - Precision policy to apply to all loaded tensors
///
/// # Returns
///
/// * `Ok(VarBuilder)` - Successfully created VarBuilder with all tensors loaded and converted
/// * `Err(String)` - Error message if VarBuilder creation fails
pub fn build_varbuilder(
    safetensors_paths: &[String],
    device: &Device,
) -> Result<VarBuilder<'static>, String> {
    build_varbuilder_with_precision(safetensors_paths, device, None)
}
///
/// This function applies a unified dtype policy based on the device and precision policy.
///
/// # Arguments
/// * `safetensors_paths` - List of paths to safetensors files
/// * `device` - Target device for the model
/// * `precision_policy` - Precision policy to use (default if None)
///
/// # Returns
/// * `Ok(VarBuilder<'static>)` - Configured VarBuilder instance
/// * `Err(String)` - Error message if VarBuilder creation fails
pub fn build_varbuilder_with_precision(
    safetensors_paths: &[String],
    device: &Device,
    precision_policy: Option<&PrecisionPolicy>,
) -> Result<VarBuilder<'static>, String> {
    // Validate that paths are provided
    if safetensors_paths.is_empty() {
        return Err("No safetensors paths provided".into());
    }

    // Validate that all files exist
    for path in safetensors_paths {
        if !std::path::Path::new(path).exists() {
            return Err(format!("Safetensors file not found: {}", path));
        }
    }

    // Unified dtype policy (delegated to precision module)
    let dtype = match precision_policy {
        Some(policy) => precision::select_dtype_by_policy(device, policy),
        None => precision::select_dtype_default(device),
    };

    // Convert to PathBuf
    let paths: Vec<std::path::PathBuf> = safetensors_paths
        .iter()
        .map(std::path::PathBuf::from)
        .collect();

    // Create VarBuilder from safetensors files
    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&paths, dtype, device)
            .map_err(|e| format!("Failed to create VarBuilder: {}", e))?
    };

    Ok(vb)
}

/// Downloads safetensors files from Hugging Face Hub to local cache.
///
/// This function ensures all required safetensors files are downloaded and available
/// in the local cache before model loading.
///
/// # Arguments
/// * `api` - The Hugging Face API repository handle
/// * `safetensors_files` - List of safetensors filenames to download
///
/// # Returns
/// * `Ok(Vec<String>)` - List of cached file paths
/// * `Err(String)` - Error message if download fails
pub fn hub_cache_safetensors(
    api: &hf_hub::api::sync::ApiRepo,
    safetensors_files: &[String],
) -> Result<Vec<String>, String> {
    let mut cached_paths: Vec<String> = Vec::with_capacity(safetensors_files.len());

    for fname in safetensors_files {
        match api.get(fname) {
            Ok(path) => {
                log_hub!("safetensors cached: {}", path.display());
                cached_paths.push(path.to_string_lossy().to_string());
            }
            Err(e) => {
                return Err(format!("Failed to download {}: {}", fname, e));
            }
        }
    }

    Ok(cached_paths)
}

/// Validates that all required safetensors files exist and are readable.
///
/// # Arguments
/// * `safetensors_paths` - List of paths to safetensors files to validate
///
/// # Returns
/// * `Ok(())` - All files are valid
/// * `Err(String)` - Error message if validation fails
pub fn validate_safetensors_files(safetensors_paths: &[String]) -> Result<(), String> {
    if safetensors_paths.is_empty() {
        return Err("No safetensors paths provided for validation".into());
    }

    for path in safetensors_paths {
        let path_buf = std::path::Path::new(path);

        // Check if file exists
        if !path_buf.exists() {
            return Err(format!("Safetensors file does not exist: {}", path));
        }

        // Check if it's a file
        if !path_buf.is_file() {
            return Err(format!("Path is not a file: {}", path));
        }

        // Try to open and read a small portion to verify it's a valid safetensors file
        match std::fs::File::open(path_buf) {
            Ok(_) => {
                // File can be opened, which is a basic validation
                log_validate!("Safetensors file is accessible: {}", path);
            }
            Err(e) => {
                return Err(format!("Cannot access safetensors file {}: {}", path, e));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_list_safetensors_empty() {
        // Test with a directory that doesn't exist or has no safetensors files
        let result = local_list_safetensors("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_safetensors_files_empty() {
        // Test validation with empty list
        let result = validate_safetensors_files(&[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("No safetensors paths provided"));
    }
}
