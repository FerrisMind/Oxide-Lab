---
description: Building with hardware acceleration features (CUDA, Metal, MKL, cuDNN)
---

# Hardware Acceleration Build Guide

This workflow describes how to build Oxide-Lab with various hardware acceleration options.

## Available Features

| Feature | Backend | Platform | Description |
|---------|---------|----------|-------------|
| `cuda` | NVIDIA GPU | Windows/Linux | CUDA GPU acceleration via cudarc |
| `cudnn` | NVIDIA GPU | Windows/Linux | cuDNN for optimized convolutions and RNNs |
| `metal` | Apple GPU | macOS | Metal acceleration for Apple Silicon |
| `fast-cpu-mkl` | Intel CPU | Windows/Linux | Intel MKL for BLAS operations |
| `fast-cpu-accelerate` | Apple CPU | macOS | Apple Accelerate framework |
| `flash-attn` | NVIDIA GPU | Windows/Linux | Flash Attention v2 (requires flash-attn-v3) |

## Build Commands

### Standard CPU Build
```bash
cd src-tauri
cargo build --release
```

### NVIDIA GPU (CUDA)
Requires: CUDA Toolkit 12.x installed

```bash
cd src-tauri
cargo build --release --features cuda
```

### NVIDIA GPU with cuDNN
Requires: CUDA Toolkit 12.x + cuDNN 8.x installed

```bash
cd src-tauri
cargo build --release --features "cuda,cudnn"
```

### macOS with Metal (Apple Silicon)
```bash
cd src-tauri
cargo build --release --features metal
```

### macOS with Accelerate (CPU optimized)
```bash
cd src-tauri
cargo build --release --features fast-cpu-accelerate
```

### Intel CPU with MKL
Requires: Intel MKL installed (via oneAPI)

```bash
cd src-tauri
cargo build --release --features fast-cpu-mkl
```

## Profiling Build

For profiling with debug symbols in release mode:
```bash
cd src-tauri
cargo build --profile release-with-debug --features cuda
```

## Tauri Development

For development with hot reload:
```bash
# Standard CPU
cargo tauri dev

# With CUDA
cargo tauri dev --features cuda --release

# With Metal (macOS)
cargo tauri dev --features metal --release
```

## Running Tests

```bash
# CPU only
cargo test

# With CUDA
cargo test --features cuda

# With Metal
cargo test --features metal
```

## Feature Combinations

✅ **Recommended combinations:**
- Windows/Linux with NVIDIA GPU: `cuda` or `cuda,cudnn`
- Windows/Linux with Intel CPU: `fast-cpu-mkl`
- macOS with Apple Silicon: `metal` or `metal,fast-cpu-accelerate`

❌ **Invalid combinations:**
- `cuda` + `metal` (mutually exclusive)
- `fast-cpu-mkl` + `fast-cpu-accelerate` (mutually exclusive)

## Environment Variables

For CUDA builds, ensure these are set:
```bash
# Windows
set CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x

# Linux
export CUDA_HOME=/usr/local/cuda-12
export LD_LIBRARY_PATH=$CUDA_HOME/lib64:$LD_LIBRARY_PATH
```

For MKL builds:
```bash
# Source Intel oneAPI environment
source /opt/intel/oneapi/setvars.sh
```

## Troubleshooting

### CUDA not found
Ensure CUDA Toolkit is installed and `nvcc` is in PATH:
```bash
nvcc --version
```

### cuDNN library not found
Download cuDNN from NVIDIA Developer and install to CUDA directory.

### MKL link errors
Ensure you've sourced the Intel oneAPI environment before building.

### Metal compile errors on macOS
Update Xcode command line tools:
```bash
xcode-select --install
```
