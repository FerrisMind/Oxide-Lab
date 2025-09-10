# Model Support Summary

This document summarizes the current state of model support in the project based on the list provided in `arch.md`.

## Models with Native GGUF Support (High Priority - Easy Implementation)

These models are fully supported by Candle for GGUF inference and can be easily integrated:

1. **Llama 2/3/3.1/3.2/3.3** - Fully supported via `quantized-llama` example
2. **CodeLlama** - Supported as part of Llama family
3. **Gemma 2** - Fully supported via `quantized-gemma` example
4. **Qwen 2/3** - Fully supported via `quantized-qwen2` and `quantized-qwen3` examples
5. **Mistral 7B** - Fully supported via `quantized` example
6. **Mixtral** - Fully supported via `quantized` example
7. **Phi-3** - Fully supported via `quantized` example
8. **SmolLM 2** - Fully supported via `quantized` example
9. **Yi** - Supported in candle_transformers

## Models with Native Safetensors Support (Medium Priority - Medium Implementation)

These models are fully supported by Candle for safetensors inference:

1. **Llama 2/3/3.1/3.2/3.3** - Fully supported via `candle_transformers::models::llama`
2. **CodeLlama** - Supported as part of Llama family
3. **Gemma 2** - Fully supported via `candle_transformers::models::gemma`
4. **Qwen 2/3** - Fully supported via `candle_transformers::models::qwen2` and `qwen3`
5. **Mistral 7B** - Fully supported via `candle_transformers::models::mistral`
6. **Mixtral** - Fully supported via `candle_transformers::models::mixtral`
7. **Phi-3** - Fully supported via `candle_transformers::models::phi3`
8. **Yi** - Fully supported via `candle_transformers::models::yi`

## Models Requiring Implementation (Low Priority - High Effort)

These models are not directly supported by Candle and would require custom implementation:

1. **FLUX.1 Kontext-dev** - Not directly supported, would need custom implementation
2. **DeepSeek-V3** - Not directly supported
3. **DeepSeek-R1 variants** - Not directly supported
4. **Llama 4** - Not released yet, no support
5. **Gemma 3n** - Not directly supported
6. **Qwen 3 Coder** - Variant not directly supported
7. **Qwen 2.5 Omni/VL** - Multimodal variants not directly supported
8. **Qwen 2.5 Coder** - Variant not directly supported
9. **QwQ** - Not directly supported
10. **Qwen 2 VL** - Multimodal variant not directly supported
11. **Mistral Small/Magistral/Devstral** - Variants not directly supported
12. **Pixtral** - Partially supported via `candle_transformers::models::pixtral`
13. **Mistral NeMo/Large** - Variants not directly supported
14. **Phi-4/Phi-3.5** - Variants not directly supported
15. **GLM** - Partially supported via `candle_transformers::models::chatglm` and `glm4`
16. **Hunyuan** - Not directly supported
17. **Orpheus** - Not directly supported
18. **LLava** - Partially supported via `candle_transformers::models::llava`
19. **TinyLlama** - Not directly supported
20. **Zephyr-SFT** - Variant not directly supported

## Priority Implementation Order

### Tier 1 (High Priority - Immediate Implementation)

1. Llama 2/3 variants (already supported)
2. Qwen 2/3 variants (already supported)
3. Mistral 7B (already supported)
4. Mixtral (already supported)
5. Gemma 2 (already supported)
6. Phi-3 (already supported)
7. SmolLM 2 (already supported)
8. Yi (already supported)

### Tier 2 (Medium Priority - Soon Implementation)

1. CodeLlama (part of Llama support)
2. Pixtral (multimodal, partial support exists)
3. DeepSeek (partial support exists)

### Tier 3 (Low Priority - Future Implementation)

1. All other models not listed above

## Implementation Details

### Architecture Registry

The architecture registry has been updated to include only models from the user's list that are supported by Candle:

- Llama
- Mistral
- Mixtral
- Gemma
- Qwen3
- Yi
- Phi3
- DeepSeek
- Pixtral
- SmolLM2

All other architectures have been removed from the registry.

### Detection Logic

The detection logic in `src-tauri/src/models/registry.rs` has been updated to:

1. First try to detect from the `general.architecture` field in GGUF files
2. Fall back to heuristic detection based on model names and descriptions

### Tests

Comprehensive tests have been added to verify the detection logic:

- Direct architecture detection from GGUF metadata
- Fallback detection using heuristics
- All tests are passing

## Next Steps

1. Implement model builders for each supported architecture
2. Add support for multimodal models like Pixtral
3. Consider adding support for additional models as needed
