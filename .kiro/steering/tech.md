---
inclusion: always
---

# Technology Stack

## Architecture

Oxide Lab is a hybrid desktop application using:
- **Frontend**: SvelteKit 5 (SPA mode) with TypeScript
- **Backend**: Rust with Tauri v2 framework
- **ML Engine**: Candle (Hugging Face's Rust ML framework)

## Core Technologies

### Frontend
- **Framework**: SvelteKit 5 with Svelte 5 runes
- **Build Tool**: Vite 6
- **Language**: TypeScript 5.6
- **UI Components**: Custom components with Phosphor Icons
- **Code Editor**: CodeMirror 6
- **Markdown**: marked with marked-highlight
- **Syntax Highlighting**: Prism.js and highlight.js

### Backend (Rust)
- **Framework**: Tauri 2.x
- **ML Framework**: Candle (candle-core, candle-nn, candle-transformers)
- **Tokenization**: tokenizers crate (v0.21)
- **HTTP Client**: reqwest with rustls-tls
- **Model Hub**: hf-hub for Hugging Face integration
- **Async Runtime**: tokio
- **Logging**: log + env_logger
- **System Monitoring**: sysinfo

### Features
- **GPU Acceleration**: CUDA support (optional, via `cuda` feature flag)
- **Metal Support**: Planned for macOS (via `metal` feature flag)
- **Default**: CPU-only mode with optional CUDA

## Code Style & Formatting

### JavaScript/TypeScript
- **Formatter**: Prettier
  - Single quotes
  - Trailing commas
  - 100 character line width
- **Linter**: ESLint 9 with TypeScript plugin
- **Style**: eslint-plugin-svelte for Svelte components

### Rust
- Standard Rust formatting (rustfmt)
- Edition 2024

## Common Commands

### Development
```bash
# Start dev server (CPU only)
npm run tauri:dev:cpu

# Start dev server (with CUDA)
npm run tauri:dev:cuda

# Frontend only (for UI work)
npm run dev
```

### Building
```bash
# Production build (CPU only)
npm run tauri:build:cpu

# Production build (with CUDA)
npm run tauri:build:cuda

# Frontend build only
npm run build
```

### Code Quality
```bash
# Run linter
npm run lint

# Fix linting issues
npm run lint:fix

# Type checking
npm run check

# Type checking (watch mode)
npm run check:watch

# Format code
npm run format

# Run tests
npm run test
```

### Version Management
```bash
# Sync version across package.json, Cargo.toml, tauri.conf.json
npm run sync-version

# Test version sync
npm run test-version-sync
```

### License Management
```bash
# Check licenses summary
npm run licenses

# Full license report
npm run licenses:full

# Export to CSV
npm run licenses:csv

# Validate licenses
npm run check-licenses
```

## Build Configuration

### Vite
- Dev server port: 1411
- HMR port: 1421
- Ignores `src-tauri` directory in watch mode

### Tauri
- Product name: "Oxide Lab"
- Identifier: com.pc.oxide-lab
- Bundle target: NSIS (Windows installer)
- Custom window decorations (frameless)
- Default window: 1280x720 (min: 640x360)

### SvelteKit
- Adapter: @sveltejs/adapter-static (SPA mode)
- Fallback: index.html
- Preprocessor: vitePreprocess

## Feature Flags (Rust)

- `default = ["cuda"]` - CUDA enabled by default
- `cuda` - Enables NVIDIA GPU acceleration
- `metal` - Enables Apple Metal (macOS, planned)

## Important Notes

- Version must be kept in sync across package.json, Cargo.toml, and tauri.conf.json
- Pre-dev and pre-build hooks automatically run version sync
- CUDA is compiled in by default but runtime falls back to CPU if unavailable
- Frontend runs in SPA mode (no SSR) due to Tauri constraints
- Example folder is excluded from TypeScript compilation
