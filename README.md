[Русский](README-RU.md)


# 🤖 Oxide Lab - Local AI Assistant

> Private, powerful, and easy-to-use AI chat right on your computer

![Oxide Lab](https://img.shields.io/badge/Status-Active-brightgreen) ![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue) ![License](https://img.shields.io/badge/License-MIT-yellow)

---

## ✨ What is it?

**Oxide Lab** is a modern desktop app for chatting with AI models that runs entirely on your computer. No subscriptions. No cloud. Your data never leaves your device.

### 🎯 Who is it for?

- AI enthusiasts who want to experiment locally
- Privacy-focused users who value full control
- Researchers who need configurable generation
- Creatives who want inspiration and brainstorming help

---

## 🚀 Key Features

### 💬 Smart chat interface
- Modern, intuitive design
- Real-time streaming responses
- Rich text and code formatting
- Persistent conversation history

### 🧠 "Thinking" mode
- Enable the **Thinking** mode to watch the AI reason step-by-step
- See the chain of thought before the final answer
- Better results on complex tasks

### ⚙️ Flexible controls
- Temperature — adjust creativity
- Top-K, Top-P, Min-P — fine-tune response style
- Repeat penalty — reduce repetition
- Context length — from 64 up to 128,000 tokens

### 🔧 Easy setup
- Load any GGUF models
- Automatic device selection (GPU/CPU)
- Smart memory handling
- Adjustable number of GPU layers

---

## 📖 Getting started

### 1️⃣ Get a model
Download a `.gguf` model and a `tokenizer.json` file:
- Recommended models: Qwen-2.5-7B, Qwen-2.5-14B, Qwen-2.5-32B
- Where to get them: Hugging Face or official model repositories

### 2️⃣ Load into the app
1. Open Oxide Lab
2. Click **"Pick model file"** and select your `.gguf`
3. Click **"Pick tokenizer"** and select `tokenizer.json`
4. Optionally set the number of GPU layers
5. Click **"Load"**

### 3️⃣ Start chatting
- Type your question or prompt
- Enable **Thinking** for deeper responses
- Adjust generation parameters to your liking
- Enjoy your private AI assistant!

---

## 🎨 Interface highlights

### 🎯 Smart commands
- Use `/think` at the beginning of a message to enable thinking mode
- Use `/no_think` to disable it for a single request

### 📊 Clear indicators
- Detailed loading progress with stages
- Generation status indicators
- Visualized chain-of-thought block

### ⚡ Quick actions
- Cancel loading with one click
- Stop generation at any moment
- Adjust parameters on the fly

---

## 🛡️ Privacy & Security

### 🔒 100% local
- All computation happens on your machine
- No external requests or cloud processing
- Full control over your data

### 💾 Data handling
- Conversations are stored only in the app session
- Models remain on your disk
- No hidden analytics or tracking

---

## 💡 Tips

### 🎯 For best results
- Enable Thinking mode for complex tasks
- Try temperature 0.7–1.0 for creativity, 0.1–0.3 for precision
- Increase context length for long documents
- Tune GPU layers based on your VRAM budget

### ⚡ Performance
- 8GB+ VRAM: you can offload most layers to GPU
- 4–6GB VRAM: try 50–70% of layers on GPU
- <4GB VRAM: prefer CPU for stability

### 🎨 Creative usage
- Use Thinking mode for analysis and planning
- High temperature for creative writing
- Long context for large documents

---

## 🆘 Need help?

### ❓ FAQ
- App doesn’t start? Check system compatibility
- Slow generation? Reduce GPU layers or model size
- Loading errors? Ensure model and tokenizer are compatible

---

## 🌟 Support the project

If Oxide Lab helps you:
- ⭐ Star the repo
- 🐛 Report issues
- 💡 Suggest features
- 🤝 Share with others

---

> Built with ❤️ for the AI community  
> Freedom, privacy, and control over your AI