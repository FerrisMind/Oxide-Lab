# Universal Weight Loader and VarBuilder

This document explains how to use the universal weight loader and VarBuilder functionality implemented in the [src-tauri/src/core/weights.rs](file:///D:/GitHub/Oxide-Lab/src-tauri/src/core/weights.rs) module.

## Overview

The universal weight loader provides a unified interface for loading model weights from both Hugging Face Hub and local storage. It handles:

1. Discovery of safetensors files (both sharded and single-file models)
2. Downloading files from Hugging Face Hub
3. Building VarBuilder instances with a unified dtype policy
4. Validation of safetensors files

## Key Functions

### `hub_list_safetensors`

Lists safetensors files from a Hugging Face Hub repository.

```rust
use hf_hub::api::sync::ApiRepo;

fn hub_list_safetensors(api: &ApiRepo) -> Result<Vec<String>, String>
```

This function looks for `model.safetensors.index.json` first (for sharded models), and falls back to `model.safetensors` for single-file models.

### `local_list_safetensors`

Lists safetensors files from a local directory.

```rust
use std::path::Path;

fn local_list_safetensors<P: AsRef<Path>>(path: P) -> Result<Vec<String>, String>
```

Similar to `hub_list_safetensors`, but works with local file paths.

### `build_varbuilder`

Builds a VarBuilder from safetensors files with a unified dtype policy.

```rust
use candle::{Device, DType};
use candle_nn::VarBuilder;

fn build_varbuilder(safetensors_paths: &[String], device: &Device) -> Result<VarBuilder<'static>, String>
```

The function applies a unified dtype policy:
- CPU: F32 for compatibility
- CUDA/Metal: BF16 for better performance

### `hub_cache_safetensors`

Downloads safetensors files from Hugging Face Hub to local cache.

```rust
use hf_hub::api::sync::ApiRepo;

fn hub_cache_safetensors(api: &ApiRepo, safetensors_files: &[String]) -> Result<Vec<String>, String>
```

### `validate_safetensors_files`

Validates that all required safetensors files exist and are readable.

```rust
fn validate_safetensors_files(safetensors_paths: &[String]) -> Result<(), String>
```

## Usage Examples

### Loading from Hugging Face Hub

```rust
use hf_hub::{api::sync::Api, Repo, RepoType};
use llm_chat_lib::core::weights;
use candle::Device;

// Setup API and repository
let api = Api::new().map_err(|e| e.to_string())?;
let repo = api.repo(Repo::new("owner/repo".to_string(), RepoType::Model));
  
// List safetensors files
let safetensors_files = weights::hub_list_safetensors(&repo)?;
  
// Download files to cache
let cached_paths = weights::hub_cache_safetensors(&repo, &safetensors_files)?;
  
// Validate files
weights::validate_safetensors_files(&cached_paths)?;
  
// Build VarBuilder
let device = Device::Cpu;
let vb = weights::build_varbuilder(&cached_paths, &device)?;
```

### Loading from Local Storage

```rust
use llm_chat_lib::core::weights;
use candle::Device;
use std::path::Path;

// List safetensors files
let model_dir = Path::new("/path/to/model");
let safetensors_files = weights::local_list_safetensors(model_dir)?;
  
// Validate files
weights::validate_safetensors_files(&safetensors_files)?;
  
// Build VarBuilder
let device = Device::Cpu;
let vb = weights::build_varbuilder(&safetensors_files, &device)?;
```

## Integration with Model Loading

The weight loader is integrated with both Hub and local model loading in:
- [src-tauri/src/api/model_loading/hub_safetensors.rs](file:///D:/GitHub/Oxide-Lab/src-tauri/src/api/model_loading/hub_safetensors.rs)
- [src-tauri/src/api/model_loading/local_safetensors.rs](file:///D:/GitHub/Oxide-Lab/src-tauri/src/api/model_loading/local_safetensors.rs)

These modules use the universal weight loader to handle the common logic of discovering, downloading, and building VarBuilder instances from safetensors files.

## Benefits

1. **Unified Interface**: Same API for both Hub and local loading
2. **Automatic Sharding Support**: Handles both sharded and single-file models automatically
3. **Consistent DType Policy**: Applies the same dtype policy across all model loading
4. **Error Handling**: Comprehensive error handling and validation
5. **Extensibility**: Easy to extend for new model formats or loading mechanisms

## Extending for New Architectures

To add support for new model architectures:

1. Use the existing weight loading functions
2. Implement a model adapter that implements the `ModelBackend` trait
3. Add architecture detection logic in the model registry
4. Use the unified VarBuilder to initialize your model

Example for a new architecture:

```rust
// In your model implementation
use llm_chat_lib::core::weights;

// List and load weights
let safetensors_files = weights::local_list_safetensors(model_dir)?;
weights::validate_safetensors_files(&safetensors_files)?;
let vb = weights::build_varbuilder(&safetensors_files, &device)?;

// Initialize your model with the VarBuilder
let model = MyNewModel::new(&config, vb)?;
```