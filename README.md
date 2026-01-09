</p>
<p align="left">
  <a href="README.md"><img src="https://img.shields.io/badge/English-5B7CFA" alt="English"></a>
  <a href="README.RU.md"><img src="https://img.shields.io/badge/Русский-232323" alt="Русский"></a>
  <a href="README.PT_BR.md"><img src="https://img.shields.io/badge/Português_BR-232323" alt="Português"></a>
</p>

---

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/logo.svg" alt="Oxide Lab Logo" width="512" height="512">

<p align="center">
  Private AI chat desktop application with local LLM support.<br>
  All inference happens on your machine — no cloud, no data sharing.
</p>

<p align="center">
  <a href="https://github.com/FerrisMind/Oxide-Lab/stargazers"><img src="https://img.shields.io/github/stars/FerrisMind/Oxide-Lab?logo=github" alt="GitHub Stars"></a>
  <a href="https://github.com/tauri-apps/awesome-tauri"><img src="https://img.shields.io/badge/Awesome-Tauri-24C8D8?logo=tauri" alt="Awesome Tauri"></a>
  <a href="https://github.com/TheComputerM/awesome-svelte"><img src="https://img.shields.io/badge/Awesome-Svelte-FF3E00?logo=svelte" alt="Awesome Svelte"></a>
</p>

<h1 align="center"></h1>

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/screenshots/chat-dark.png" alt="Oxide Lab Chat Interface" width="900">
</p>

## 📚 Table of Contents

- [What is this?](#-what-is-this)
- [Key Features](#-key-features)
- [Installation & Setup](#️-installation--setup)
- [How to Start Using](#-how-to-start-using)
- [System Requirements](#️-system-requirements)
- [Supported Models](#-supported-models)
- [Privacy and Security](#️-privacy-and-security)
- [Acknowledgments](#-acknowledgments)
- [License](#-license)

## ✨ What is this?

Oxide Lab is a native desktop application for running large language models locally. Built with Rust and Tauri v2, it provides a fast, private chat interface without requiring internet connectivity or external API services.

## 🚀 Key Features

- 100% local inference — your data never leaves your machine
- Multi-architecture support: Llama, Qwen2, Qwen2.5, Qwen3, Qwen3 MoE, Mistral, Mixtral, DeepSeek, Yi, SmolLM2
- GGUF and SafeTensors model formats
- Hardware acceleration: CPU, CUDA (NVIDIA), Metal (Apple Silicon), Intel MKL, Apple Accelerate
- Streaming text generation
- Multi-language UI: English, Russian, Brazilian Portuguese
- Modern interface built with Svelte 5 and Tailwind CSS

## 🛠️ Installation & Setup

### Prerequisites

- Node.js (for frontend build)
- Rust toolchain (for backend)
- For CUDA: NVIDIA GPU with CUDA toolkit
- For Metal: macOS with Apple Silicon

### Development

```bash
# Install dependencies
npm install

# Run with CPU backend
npm run tauri:dev:cpu

# Run with CUDA backend (NVIDIA GPU)
npm run tauri:dev:cuda

# Platform-aware development
npm run app:dev
```

### Build

```bash
# Build with CPU backend
npm run tauri:build:cpu

# Build with CUDA backend
npm run tauri:build:cuda
```

### Quality Checks

```bash
npm run lint          # ESLint
npm run lint:fix      # ESLint with auto-fix
npm run check         # Svelte type checking
npm run format        # Prettier formatting
npm run test          # Vitest tests
```

### Rust-specific (from src-tauri/)

```bash
cargo clippy          # Linting
cargo test            # Unit tests
cargo audit           # Security audit
```

## 📖 How to Start Using

1. Build or download the application
2. Download a compatible GGUF or SafeTensors model (e.g., from Hugging Face)
3. Launch Oxide Lab
4. Load your model through the interface
5. Start chatting

## 🖥️ System Requirements

- Windows, macOS, or Linux
- Minimum 8 GB RAM (16+ GB recommended for larger models)
- For GPU acceleration:
  - NVIDIA: CUDA-compatible GPU
  - Apple: M1/M2/M3 chip (Metal)
  - Intel: CPU with MKL support

## 🤖 Supported Models

Architectures with full support:
- Llama (1, 2, 3, 4), Mistral, Mixtral, DeepSeek, Yi, SmolLM2, CodeLlama
- Qwen2, Qwen2.5, Qwen2 MoE
- Qwen3, Qwen3 MoE

Formats:
- GGUF (quantized models)
- SafeTensors

## 🛡️ Privacy and Security

- All processing happens locally on your device
- No telemetry or data collection
- No internet connection required for inference
- Content Security Policy (CSP) enforced

## 🙏 Acknowledgments

This project is built on top of excellent open-source work:

- [Candle](https://github.com/huggingface/candle) — ML framework for Rust (HuggingFace)
- [Tauri](https://tauri.app/) — Desktop application framework
- [Svelte](https://svelte.dev/) — Frontend framework
- [Tokenizers](https://github.com/huggingface/tokenizers) — Fast tokenization (HuggingFace)

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for full dependency attribution.

## 📄 License

Apache-2.0 — see [LICENSE](LICENSE)

Copyright (c) 2025 FerrisMind
