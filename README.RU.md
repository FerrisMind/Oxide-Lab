</p>
<p align="left">
  <a href="README.md"><img src="https://img.shields.io/badge/English-232323" alt="English"></a>
  <a href="README.RU.md"><img src="https://img.shields.io/badge/Русский-D65C5C" alt="Русский"></a>
  <a href="README.PT_BR.md"><img src="https://img.shields.io/badge/Português_BR-232323" alt="Português"></a>
</p>

---

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/logo.svg" alt="Oxide Lab Logo" width="512" height="512">

<p align="center">
  Приватное десктопное приложение для AI-чата с поддержкой локальных LLM.<br>
  Весь инференс происходит на вашем компьютере — без облака, без передачи данных.
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

## 📚 Содержание

- [Что это?](#-что-это)
- [Демо](#-демо)
- [Ключевые возможности](#-ключевые-возможности)
- [Установка и настройка](#️-установка-и-настройка)
- [Как начать использовать](#-как-начать-использовать)
- [Системные требования](#️-системные-требования)
- [Поддерживаемые модели](#-поддерживаемые-модели)
- [Приватность и безопасность](#️-приватность-и-безопасность)
- [Благодарности](#-благодарности)
- [Лицензия](#-лицензия)

## ✨ Что это?

Oxide Lab — нативное десктопное приложение для запуска больших языковых моделей локально. Построено на Rust и Tauri v2, обеспечивает быстрый приватный чат-интерфейс без необходимости подключения к интернету или внешним API-сервисам.

## 🎬 Демо

<p align="center">
  <video src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/screenshots/dem1.mp4" width="600" controls></video>
</p>

<p align="center">
  <video src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/screenshots/dem2.mp4" width="600" controls></video>
</p>

<p align="center">
  <video src="https://raw.githubusercontent.com/FerrisMind/Oxide-Lab/main/.github/assets/screenshots/dem3.mp4" width="600" controls></video>
</p>

## 🚀 Ключевые возможности

- 100% локальный инференс — ваши данные никогда не покидают компьютер
- Поддержка множества архитектур: Llama, Qwen2, Qwen2.5, Qwen3, Qwen3 MoE, Mistral, Mixtral, DeepSeek, Yi, SmolLM2
- Форматы моделей GGUF и SafeTensors
- Аппаратное ускорение: CPU, CUDA (NVIDIA), Metal (Apple Silicon), Intel MKL, Apple Accelerate
- Потоковая генерация текста
- Многоязычный интерфейс: английский, русский, бразильский португальский
- Современный интерфейс на Svelte 5 и Tailwind CSS

## 🛠️ Установка и настройка

### Требования

- Node.js (для сборки фронтенда)
- Rust toolchain (для бэкенда)
- Для CUDA: видеокарта NVIDIA с CUDA toolkit
- Для Metal: macOS с Apple Silicon

### Разработка

```bash
# Установка зависимостей
npm install

# Запуск с CPU бэкендом
npm run tauri:dev:cpu

# Запуск с CUDA бэкендом (NVIDIA GPU)
npm run tauri:dev:cuda

# Платформо-зависимая разработка
npm run app:dev
```

### Сборка

```bash
# Сборка с CPU бэкендом
npm run tauri:build:cpu

# Сборка с CUDA бэкендом
npm run tauri:build:cuda
```

### Проверка качества

```bash
npm run lint          # ESLint
npm run lint:fix      # ESLint с автоисправлением
npm run check         # Проверка типов Svelte
npm run format        # Форматирование Prettier
npm run test          # Тесты Vitest
```

### Rust-специфичные команды (из src-tauri/)

```bash
cargo clippy          # Линтинг
cargo test            # Юнит-тесты
cargo audit           # Аудит безопасности
```

## 📖 Как начать использовать

1. Соберите или скачайте приложение
2. Скачайте совместимую модель в формате GGUF или SafeTensors (например, с Hugging Face)
3. Запустите Oxide Lab
4. Загрузите модель через интерфейс
5. Начните общение

## 🖥️ Системные требования

- Windows, macOS или Linux
- Минимум 8 ГБ RAM (рекомендуется 16+ ГБ для больших моделей)
- Для GPU-ускорения:
  - NVIDIA: GPU с поддержкой CUDA
  - Apple: чип M1/M2/M3 (Metal)
  - Intel: CPU с поддержкой MKL

## 🤖 Поддерживаемые модели

Архитектуры с полной поддержкой:
- Llama (1, 2, 3, 4), Mistral, Mixtral, DeepSeek, Yi, SmolLM2, CodeLlama
- Qwen2, Qwen2.5, Qwen2 MoE
- Qwen3, Qwen3 MoE

Форматы:
- GGUF (квантованные модели)
- SafeTensors

## 🛡️ Приватность и безопасность

- Вся обработка происходит локально на вашем устройстве
- Никакой телеметрии или сбора данных
- Интернет-соединение не требуется для инференса
- Применяется Content Security Policy (CSP)

## 🙏 Благодарности

Проект построен на отличных open-source решениях:

- [Candle](https://github.com/huggingface/candle) — ML-фреймворк для Rust (HuggingFace)
- [Tauri](https://tauri.app/) — фреймворк для десктопных приложений
- [Svelte](https://svelte.dev/) — фронтенд-фреймворк
- [Tokenizers](https://github.com/huggingface/tokenizers) — быстрая токенизация (HuggingFace)

Полный список зависимостей см. в [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## 📄 Лицензия

Apache-2.0 — см. [LICENSE](LICENSE)

Copyright (c) 2025 FerrisMind
