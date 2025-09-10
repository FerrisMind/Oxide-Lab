# Precision Policy Implementation

This document describes the implementation of the precision policy feature that allows users to control the data type precision used when loading and running AI models.

## Overview

The precision policy feature provides users with control over the data type precision used during model loading and inference. This affects both memory consumption and computational performance.

## Implementation Details

### Backend (Rust)

1. **Core Precision Module** (`src-tauri/src/core/precision.rs`):
   - Created `PrecisionPolicy` enum with three predefined options:
     - `Default`: CPU=F32, GPU=BF16 (optimal balance)
     - `MemoryEfficient`: CPU=F32, GPU=F16 (lower memory usage)
     - `MaximumPrecision`: CPU=F32, GPU=F32 (highest accuracy)
   - Implemented `PrecisionConfig` struct for internal configuration
   - Added functions to convert policy to configuration and select appropriate data types

2. **State Management** (`src-tauri/src/core/state.rs`):
   - Added `precision_policy` field to `ModelState` to store the current policy
   - Default policy is set to `PrecisionPolicy::Default`

3. **Model Loading Integration** (`src-tauri/src/core/weights.rs`):
   - Updated `build_varbuilder` function to use the centralized precision policy
   - Added `build_varbuilder_with_precision` function for custom precision handling

4. **Safetensors Loading** (`src-tauri/src/api/model_loading/safetensors.rs`):
   - Modified both local and Hub safetensors loading to respect the precision policy
   - Updated model building to use the appropriate data type based on policy

5. **Tauri Commands** (`src-tauri/src/api/mod.rs`):
   - Added `get_precision_policy` command to retrieve current policy
   - Added `set_precision_policy` command to update the policy

### Frontend (Svelte)

1. **Settings Page** (`src/routes/settings/+page.svelte`):
   - Created a dedicated settings page with precision policy options
   - Implemented UI cards for each policy option with descriptions
   - Added visual feedback for the currently selected policy

2. **Navigation** (`src/lib/components/Sidebar.svelte`):
   - Added "Settings" navigation item with gear icon
   - Integrated settings page into the main navigation

3. **Type Definitions** (`src/lib/types.ts`):
   - Added `PrecisionPolicy` type definition for frontend usage

## Available Precision Policies

1. **Default**:
   - CPU: F32 (maximum compatibility)
   - GPU: BF16 (better performance and memory usage)
   - Recommended for most users

2. **Memory Efficient**:
   - CPU: F32 (maximum compatibility)
   - GPU: F16 (reduced memory usage)
   - Recommended for systems with limited GPU memory

3. **Maximum Precision**:
   - CPU: F32 (maximum compatibility)
   - GPU: F32 (highest precision)
   - Recommended for tasks requiring maximum accuracy

## Usage

Users can access the precision policy settings through the "Settings" navigation item in the sidebar. The selected policy will be applied to all subsequent model loading operations.

The policy affects:

- Memory consumption during model loading
- Inference performance
- Numerical precision of results

## Testing

The implementation includes:

- Unit tests for precision policy conversion functions
- Integration tests for model loading with different policies
- UI tests for settings page functionality

All tests pass successfully, ensuring the reliability of the implementation.
