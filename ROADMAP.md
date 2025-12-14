# 🗺️ Oxide Lab Roadmap

> **Vision:** We're building the lightest and most accessible desktop AI client. Our goal is to enable running modern LLMs (Llama, Qwen, Mistral) locally and privately, even on devices with 4GB RAM, without sacrificing the performance of powerful GPUs.

This document describes the current development status and our plans for the near future.

## 🌟 Key Principles (Guiding Principles)

- **Edge-First:** We optimize the software to run smoothly on old laptops, while on powerful PCs it works instantly.
- **Pure Rust:** We use Rust (Tauri + Candle) instead of Electron to save your memory (target: <200MB RAM).
- **Privacy by Design:** No telemetry, no hidden requests. Your chat is only on your disk.

---

## 🚧 Current Status: Phase 1 (Foundation)

_Focus on stability of core functionality and support for Qwen/Llama._

- [x] Project initialization on Tauri v2 + Svelte 5
- [x] Candle engine integration (Rust ML framework)
- [x] **Basic chat UI** (streaming responses)
- [x] **GGUF model support** (loading and inference)
- [ ] **Onboarding Wizard**: Simple setup on first launch
- [x] **Qwen 3** architecture support (optimized for small weights)
- [x] Chat history persistence (local SQLite database; currently using localStorage)

## 🔮 Upcoming Plans: Phase 2 (Experience)

_Improving UX and expanding capabilities._

- [x] **Performance Monitor**: Visualization of CPU/GPU usage and tokens/sec
- [ ] **Llama 3.x** and **Mistral** architecture support
- [x] Auto-detection and support for **CUDA** (NVIDIA GPU acceleration)
- [x] In-app model downloader (HuggingFace Integration)
- [x] Generation settings (Temperature, Top-P, System Prompts)
- [x] `.safetensors` format support

## 🚀 Future Plans (Backlog)

_Ideas we want to implement after core stabilization._

- [ ] Versions for **macOS** and **Linux**
- [ ] Theme customization (Dark/Light mode)
- [ ] Chat export (Markdown/JSON)
- [ ] Local **REST API** (OpenAI-compatible) for developers
- [ ] Vision model support (multimodality)

_Note: This roadmap may change based on community feedback._
