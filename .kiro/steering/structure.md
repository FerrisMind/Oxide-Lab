---
inclusion: always
---

# Project Structure

## Root Directory Layout

```
oxide-lab/
├── src/                    # Frontend source (SvelteKit)
├── src-tauri/              # Backend source (Rust/Tauri)
├── static/                 # Static assets
├── models/                 # Model storage and metadata
├── scripts/                # Build and utility scripts
├── example/                # Documentation and examples (excluded from builds)
├── build/                  # Frontend build output
├── node_modules/           # Node dependencies
└── target/                 # Rust build artifacts
```

## Frontend Structure (`src/`)

### Core Files
- `app.html` - HTML template
- `app.css` - Global styles
- `app.d.ts` - TypeScript declarations
- `main.rs` - Rust entry point (symlink/reference)

### Library (`src/lib/`)

#### Components (`src/lib/components/`)
Reusable Svelte components organized by feature area. Components use Svelte 5 runes syntax.

#### Stores (`src/lib/stores/`)
Svelte stores for state management:
- `chat.ts` - Chat state and messages
- `chat-history.ts` - Session management
- `local-models.ts` - Local model scanning and selection
- `remote-models.ts` - Remote model browsing (Hugging Face)
- `model-cards.ts` - Model metadata and filtering
- `inference-metrics.ts` - Performance tracking
- `download-manager.ts` - Download job management
- `experimental-features.svelte.ts` - Feature flags
- `page-tabs.svelte.ts` - Tab navigation state
- `sidebar.ts` - UI state

**Store Patterns:**
- Use `writable()` for mutable state
- Use `derived()` for computed values
- Export stores and helper functions
- Persist important state to localStorage/backend

#### Services (`src/lib/services/`)
Business logic and API communication:
- `local-models.ts` - Local model operations
- `model-cards.ts` - Model card management
- `performance-service.ts` - Performance monitoring
- `huggingface/` - Hugging Face API integration

**Service Patterns:**
- Static class methods for stateless operations
- Singleton instances for stateful services
- Use Tauri `invoke()` for backend communication

#### Types (`src/lib/types/`)
TypeScript type definitions matching Rust backend types.

#### Styles (`src/lib/styles/`)
Shared CSS and styling utilities.

### Routes (`src/routes/`)
SvelteKit pages and layouts. Uses file-based routing in SPA mode.

### Tests (`src/tests/`)
Frontend unit and integration tests using Vitest.

## Backend Structure (`src-tauri/`)

### Core Files
- `Cargo.toml` - Rust dependencies and features
- `tauri.conf.json` - Tauri configuration
- `build.rs` - Build script

### Source (`src-tauri/src/`)
Rust backend code:
- `lib.rs` - Main library entry point
- Command handlers (Tauri commands invoked from frontend)
- ML inference engine (Candle integration)
- Model loading and management
- System monitoring and metrics

### Capabilities (`src-tauri/capabilities/`)
Tauri security permissions and capabilities.

### Resources (`src-tauri/resources/`)
Bundled resources for the application.

### Icons (`src-tauri/icons/`)
Application icons for various platforms.

### Tests (`src-tauri/tests/`)
Rust unit and integration tests.

## Models Directory (`models/`)

```
models/
├── gguf/           # GGUF model files
├── safetensors/    # SafeTensors format (future)
├── metadatas/      # Model metadata files
└── model_cards.json # Model registry
```

## Scripts Directory (`scripts/`)

- `sync-version.cjs` - Synchronize version across configs
- `test-version-sync.cjs` - Validate version consistency
- `check-licenses.js` - License compliance checking
- `auto-version-bump.ps1` - Automated version bumping

## Configuration Files

### Root Level
- `package.json` - Node dependencies and scripts
- `tsconfig.json` - TypeScript configuration
- `vite.config.js` - Vite build configuration
- `svelte.config.js` - SvelteKit configuration
- `vitest.config.ts` - Test configuration
- `.prettierrc.json` - Code formatting rules
- `.eslintrc.json` / `eslint.config.cjs` - Linting rules
- `.gitignore` - Git ignore patterns

### Tauri Specific
- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/tauri.conf.json` - App configuration
- `src-tauri/deny.toml` - Cargo deny configuration

## Important Conventions

### File Naming
- Svelte components: PascalCase (e.g., `ChatMessage.svelte`)
- TypeScript/JavaScript: kebab-case (e.g., `local-models.ts`)
- Stores with runes: `.svelte.ts` extension
- Rust files: snake_case (e.g., `model_loader.rs`)

### Import Paths
- Use `$lib` alias for `src/lib` imports
- Relative imports within same directory
- Type imports use `import type` syntax

### State Management
- Stores in `src/lib/stores/` for shared state
- Component-local state using Svelte runes (`$state`, `$derived`)
- Backend state synchronized via Tauri events

### Backend Communication
- Frontend calls backend via `invoke('command_name', args)`
- Backend emits events via Tauri event system
- Types must match between frontend and backend

### Testing
- Frontend tests: Vitest with Testing Library
- Backend tests: Standard Rust `#[test]` and `#[cfg(test)]`
- Test files colocated with source or in `tests/` directories

## Excluded Directories

- `example/` - Documentation, not part of build
- `node_modules/` - Dependencies
- `target/` - Rust build artifacts
- `build/` - Frontend build output
- `.svelte-kit/` - SvelteKit generated files
