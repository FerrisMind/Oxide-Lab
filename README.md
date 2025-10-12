[Русский](README-RU.md)

<!-- Project Logo -->
<p align="center">
  <img src=".github/assets/logo.svg" alt="Oxide Lab Logo" width="512" />
</p>

> **Private, powerful and easy-to-use AI chat right on your computer**

![Oxide Lab](https://img.shields.io/badge/Status-Active-brightgreen) ![Platform](https://img.shields.io/badge/Platform-Windows-blue) ![License](https://img.shields.io/badge/License-MIT-yellow) ![Legal](https://img.shields.io/badge/Legal-Compliant-green)

---

## 📚 Table of Contents

- [What is this?](#-what-is-this)
- [Who is this app for?](#-who-is-this-app-for)
- [Key Features](#-key-features)
- [Installation & Setup](#️-installation--setup)
- [How to Start Using](#-how-to-start-using)
- [Interface Features](#-interface-features)
- [Privacy and Security](#-privacy-and-security)
- [Tips and Recommendations](#-tips-and-recommendations)
- [System Requirements and Limitations](#-system-requirements-and-limitations)
- [Support the Project](#-support-the-project)

---

## ✨ What is this?

**Oxide Lab** is a modern desktop application for communicating with AI models that runs completely locally on your computer. No subscriptions, no data sent to the internet — just you and your personal AI assistant.

### 🎯 Who is this app for?

- **AI enthusiasts** — want to experiment with models locally
- **Privacy matters** — your data stays only with you
- **Researchers** — need control over generation parameters
- **Creative minds** — use AI for writing, brainstorming and inspiration

---

## 🚀 Key Features

### 💬 **Smart Chat Interface**

- Modern and intuitive design
- Real-time streaming responses
- Support for text and code formatting

### 🧠 **Thinking Mode**

- Enable the **"Thinking"** feature and watch AI think
- See the analysis process before the final answer
- Higher quality and thoughtful solutions to complex tasks

### ⚙️ **Flexible Settings**

- **Temperature** — control response creativity
- **Top-K, Top-P, Min-P** — fine-tune generation style
- **Repeat Penalty** — avoid repetitions
- **Context Length** — depends on model and device resources

### 🔧 **Easy Setup**

- Support for local Qwen3 models in GGUF format (other models — in plans)
- Intelligent memory management

---

## 🛠️ Installation & Setup

### Prerequisites

Before installing Oxide Lab, ensure you have the following installed:

- **Node.js** (version 18 or higher) - [Download](https://nodejs.org/)
- **Rust** (latest stable) - [Install](https://rustup.rs/)
- **Git** - [Download](https://git-scm.com/)

#### For GPU Acceleration (Optional but Recommended)

- **CUDA 12.0+** for NVIDIA GPUs (Windows/Linux)

### Installation Steps

1. **Clone the repository:**

   ```bash
   git clone https://github.com/FerrisMind/Oxide-Lab.git
   cd Oxide-Lab
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Run in development mode:**

   ```bash
   # For CPU-only mode
   npm run tauri:dev:cpu

   # For CUDA GPU mode (if CUDA is available)
   npm run tauri:dev:cuda
   ```

4. **Build for production:**

   ```bash
   # CPU-only build
   npm run tauri:build:cpu

   # CUDA build
   npm run tauri:build:cuda
   ```

### System Requirements

- **OS:** Windows 10/11, Linux, macOS
- **RAM:** Minimum 4GB, Recommended 8GB+
- **Storage:** 500MB for application + model size
- **GPU:** Optional, but recommended for better performance

### Troubleshooting

- If you encounter build issues, ensure Rust and Node.js are properly installed
- For GPU support, verify CUDA installation
- Check the [Issues](https://github.com/FerrisMind/Oxide-Lab/issues) page for common problems

---

## 📖 How to Start Using

### 1️⃣ **Get the Model**

Download a model in `.gguf` format and `tokenizer.json` file:

- **Recommended models:** Qwen3 4B (and other Qwen3 variants in GGUF)
- **Where to download:** [Hugging Face](https://huggingface.co/collections/Qwen/qwen3-67dd247413f0e2e4f653967f), official model repositories

### 2️⃣ **Load into Application**

1. Open Oxide Lab
2. Click **"Select Model File"** and specify path to `.gguf` file
3. Optionally configure inference parameters
4. Click **"Load"**

### 3️⃣ **Start Chatting**

- Enter your question or request
- Enable **"Thinking"** for deeper responses
- Adjust generation parameters to your taste
- Enjoy conversation with your personal AI!

---

## 🎨 Interface Features

### 📊 **Informative Indicators**

- Model loading progress with detailed stages
- Generation status indicators
- Visual display of AI thinking

<p align="center">
  <img src=".github/assets/screenshots/chat-dark.png" alt="Oxide Lab Chat UI (Dark)" width="720" />
</p>

### ⚡ **Quick Actions**

- Cancel model loading with one click
- Stop generation at any moment
- Quick parameter changes without reloading

---

## 🛡️ Privacy and Security

### 🔒 **100% Local**

- All computations happen on your computer
- No external requests or data sending
- Full control over your information

### 💾 **Data Management**

- Conversations stored only in application session
- Models remain on your disk
- No hidden data collection

---

## 💡 Tips and Recommendations

### 🎯 **For best results:**

- Use thinking mode for complex tasks
- The app already has the best settings built-in based on Qwen3 model manufacturer recommendations. Just enable and use!
- The app also supports changing default settings. Experiment with temperature: 0.7-1.0 for creativity, 0.1-0.3 for accuracy
- Increase context for working with long documents

### ⚡ **Performance optimization:**

- Supports CPU and GPU (CUDA)

### 🎨 **Creative usage:**

- Enable thinking for text analysis and problem solving
- Experiment with high temperature for creative writing
- Use long context for working with large documents

---

## 🖥️ System Requirements and Limitations

### Supported Platforms

- Windows 10/11 — full support
- Linux and macOS — in planning stage (not yet supported)

### Models

- Supported: Qwen3 in GGUF format (mono-architecture)
- Important: compatibility with other models is not yet guaranteed

### Minimum Hardware Requirements

The smallest Qwen3 models (0.6B and 1.7B) work with acceptable speed and quality even on devices with 2-core CPU and 4 GB RAM. The 4B model also works in this application with such devices, but the performance is many times lower and requires more memory, which is difficult to achieve, for example, with LM Studio without significant quality loss.

### Context and Performance

- Effective context length depends on: selected model, available RAM
- Practically achievable context length may be lower than theoretically declared by the model
- The larger the context, the higher the memory requirements and lower the generation speed

---

## 🌟 Support the Project

If Oxide Lab has been useful to you:

- ⭐ Star the project
- 🐛 Report bugs
- 💡 Suggest new features
- 🤝 Share with friends

---

> **Made with ❤️ for the AI enthusiast community**  
> _Freedom, privacy and control over artificial intelligence_
