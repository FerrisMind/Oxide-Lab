# Extensibility Architecture

Goal: Minimize architecture-dependent code and simplify adding new architectures (GGUF and float/safetensors) through unified abstractions for loading, initialization, tokenization, and generation.

Priority groups: P0 (high impact, low risks/costs) → P1 → P2.

## P0 — Fast and Most Effective

- Auto-device selection CUDA → Metal → CPU
  - Purpose: Automatically select the best available device, as in Candle examples.
  - Impact: More predictable performance without extra configuration; unified default policy.
  - Costs: Low (targeted changes).
  - Location: `src-tauri/src/core/device.rs` (`select_device`), messages in `src-tauri/src/api/device.rs` if needed.
  - Status: IMPLEMENTED ✓ (improved version with correct runtime detection)

- Universal weights loader and VarBuilder (Hub/local)
  - Purpose: Extract common logic into a module: search for `model.safetensors.index.json`, collect shard/file list, build `VarBuilder` with unified `dtype` policy.
  - Impact: Eliminate duplication, single maintenance point, fast path for new architectures.
  - Costs: Low-medium.
  - Location: New module `src-tauri/src/core/weights.rs` with functions: `hub_list_safetensors(...)`, `local_list_safetensors(...)`, `build_varbuilder(...)`, `hub_cache_safetensors(...)`; replace local parsing in `api/model_loading/hub_safetensors.rs` and `api/model_loading/local_safetensors.rs`.
  - Status: IMPLEMENTED ✓ (full implementation with Hub and local path support, unified dtype policy)

- Adapter for float-LLM from candle_transformers (safetensors)
  - Purpose: Allow using `ModelForCausalLM` models (e.g., Qwen3 float) through the existing `ModelBackend` trait.
  - Impact: Unified generation for both GGUF and HF safetensors (without quantization); removes barrier for many architectures from `candle_transformers`.
  - Costs: Low-medium (thin wrapper + `AnyModel::from_candle_qwen3` implementation).
  - Location: New `src-tauri/src/models/common/candle_llm.rs` (implements `ModelBackend` over `ModelForCausalLM`), update `src-tauri/src/models/common/model.rs` (implement `from_candle_qwen3`).
  - Status: IMPLEMENTED ✓ (full implementation with support for various architectures from candle_transformers)

- Architecture registry extension and detection
  - Purpose: Recognize architectures from the supported model list (Llama, Mistral, Mixtral, Gemma, Qwen, Yi, Phi3, DeepSeek, Pixtral, SmolLM2) by `config.json`/GGUF metadata.
  - Impact: Centralized architecture determination → single entry point for model builders.
  - Costs: Low.
  - Location: `src-tauri/src/models/registry.rs` — add heuristics (by fields like `model_type`, `architectures`, `general.architecture`, etc.).
  - Status: IMPLEMENTED ✓ (full implementation with support for user's model list, unsupported architectures removed)

## P1 — System Universality Improvements

- Introduction of model trait-builder (`ModelBuilder`) and factory
  - Purpose: Unify model creation from GGUF and from VarBuilder+config (`from_gguf(...)`, `from_varbuilder(...)`).
  - Impact: Minimize architectural code to local adapters; simplify connecting new architectures.
  - Costs: Medium (introducing trait layer and registering builders by `ArchKind`).
  - Location: New `src-tauri/src/models/common/builder.rs`; registration in `src-tauri/src/models/registry.rs`; adapters in `src-tauri/src/models/<arch>.rs`.
  - Status: IMPLEMENTED ✓ (full implementation with unified interface for GGUF and safetensors models, factory and architecture detection)

- Prompt builder based on chat template
  - Purpose: Uniformly form prompts from messages and template (`tokenizer.json`/metadata).
  - Impact: Same chat behavior across different models; reduce copy-paste.
  - Costs: Low-medium.
  - Location: New `src-tauri/src/core/prompt.rs`; usage in `src-tauri/src/generate/stream.rs` before tokenization.
  - Status: IMPLEMENTED ✓ (full implementation with Jinja template support and fallback formatting, integrated into generation pipeline)

- Unification of generation parameters (SamplingOptions)
  - Purpose: Collect `temperature/top_k/top_p/min_p/seed/repeat_penalty/repeat_last_n` into one type with defaults and unified logging.
  - Impact: Stable behavior and easy option passing to all generation modes.
  - Costs: Low.
  - Location: New `src-tauri/src/core/config.rs` (or `generate/options.rs`); update `generate/sampling.rs`, `generate/stream.rs`.
  - Status: IMPLEMENTED ✓ (full implementation with unified SamplingOptions type, preset configurations and integration into generation pipeline)

- Centralization of dtype/precision policy
  - Purpose: Unified policy F16/BF16/F32 (GPU→BF16/F16, CPU→F32) with optional override capability through UI.
  - Impact: Consistent precision and memory consumption; fewer discrepancies between loaders; user control over precision.
  - Costs: Low.
  - Location: New `src-tauri/src/core/precision.rs`; use in `core/weights.rs` and during model initialization; UI in `src/routes/settings`.
  - Status: IMPLEMENTED ✓ (full implementation with centralized precision policy, default configurations, customization capability, and UI integration)

- Uniform logging and errors
  - Purpose: Unify prefixes (`[load]`, `[infer]`, `[hub]`, `[template]`) and error representation.
  - Impact: Improved maintenance and debugging; clear traces for all architectures.
  - Costs: Low.
  - Location: (Optional) `src-tauri/src/core/error.rs`/`log.rs`; apply in loaders and generation.

## P2 — Reliability and Growth Readiness

- Minimal smoke tests for key components
  - Purpose: Check `tokenizer_from_gguf_metadata`, EOS extraction, prompt builder, `ModelBackend` interface (prefill/decode steps on small inputs).
  - Impact: Reduces regressions when adding architectures; speeds up review.
  - Costs: Medium (test stubs/mini-models).
  - Location: `src-tauri/tests` or unit tests next to code.

- Common preprocessors/postprocessors for multimodal models
  - Purpose: Lay foundation for CV/Audio (normalization, resize, mel-spectrograms) following `candle_examples` pattern.
  - Impact: Readiness for future architectures (CLIP, Whisper, etc.).
  - Costs: Medium; optional.
  - Location: `src-tauri/src/core/vision.rs`, `src-tauri/src/core/audio.rs` (as needed).
  - Status: IMPLEMENTED ✓ (full implementation with support for image and audio preprocessing)
  - Implementation:
    - `src-tauri/src/core/vision.rs` module:
      - Image preprocessing utilities (normalization, resizing, tensor conversion)
      - Support for various image formats (JPEG, PNG, BMP)
      - Integration with computer vision models (ViT, CLIP, etc.)
      - Feature extraction from images
    - `src-tauri/src/core/audio.rs` module:
      - Audio preprocessing utilities (amplitude normalization, resampling)
      - Conversion to mel-spectrograms for speech recognition models
      - Support for various audio formats (WAV, MP3)
      - Integration with audio processing models (Whisper, EnCodec, etc.)
    - Common interfaces:
      - Traits for unified multimodal data processing
      - Support for streaming processing of large files
      - Integration with existing weight loading and generation modules

- Extension of `ModelBackend` interface if needed
  - Purpose: Explicitly express support for kv-cache/positions, special tokens, context constraints.
  - Impact: Less fragile integration of different implementations; simplification of optimizations.
  - Costs: Low-medium (as needed).
  - Location: `src-tauri/src/models/common/model.rs` (trait and adapter extension).

## Recommended Implementation Order (by Priority)

1. P0: auto-device; `core/weights.rs` module; float-LLM adapter; architecture detection extension.
2. P1: `ModelBuilder`+factory; prompt builder; `SamplingOptions`; `precision` policy; unified logging.
3. P2: smoke tests; multimodal utilities; `ModelBackend` evolution as needed.

## Expected Result

- Adding a new architecture comes down to: (a) add detection in `registry`, (b) implement adapter/builder in `models/<arch>.rs` (GGUF and/or VarBuilder), (c) configure prompt-builder if needed. Everything else (weight loading, dtype policy, device, generation, tokenization, logs) is universal and unchanged.
