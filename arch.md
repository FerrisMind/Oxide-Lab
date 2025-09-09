gpt-oss
FLUX.1 Kontext-dev
DeepSeek-V3
DeepSeek-R1-0528-Qwen3-8B
DeepSeek-R1-Distill Llama 3
DeepSeek-R1-Distill Llama 3.3
DeepSeek-R1-Distill Qwen 2.5 
Llama 4 
Llama 3/3.1/3.2/3.3
Llama 2
CodeLlama
Gemma 3n
Gemma 3
Gemma 2
Qwen 3 Coder
Qwen 3
Qwen 2.5 Omni
Qwen 2.5 VL
Qwen 2.5
Qwen 2.5 Coder
QwQ
Qwen 2
Qwen 2 VL
Mistral Small
Magistral
Devstral
Pixtral
Mistral Small
Mistral NeMo
Mistral Large
Mistral 7 B
Mixtral
Phi-4
Phi-3.5
Phi-3
GLM
Hunyuan
Orpheus
LLava
TinyLlama
SmolLM 2
Zephyr-SFT
Yi

# Candle Support Analysis

## Models with Native GGUF Support (High Priority - Easy Implementation)
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
1. **Llama 2/3/3.1/3.2/3.3** - Fully supported via `candle_transformers::models::llama`
2. **CodeLlama** - Supported as part of Llama family
3. **Gemma 2/3** - Fully supported via `candle_transformers::models::gemma` and `gemma2`
4. **Qwen 2/3** - Fully supported via `candle_transformers::models::qwen2` and `qwen3`
5. **Mistral 7B** - Fully supported via `candle_transformers::models::mistral`
6. **Mixtral** - Fully supported via `candle_transformers::models::mixtral`
7. **Phi-3** - Fully supported via `candle_transformers::models::phi3`
8. **Yi** - Fully supported via `candle_transformers::models::yi`
9. **DeepSeek** - Partially supported via `candle_transformers::models::deepseek2`

## Models Requiring Implementation (Low Priority - High Effort)
1. **gpt-oss** - OpenAI model, not directly supported by Candle (requires API access)
2. **FLUX.1 Kontext-dev** - Not directly supported, would need custom implementation
3. **DeepSeek-V3** - Not directly supported
4. **DeepSeek-R1 variants** - Not directly supported
5. **Llama 4** - Not released yet, no support
6. **Gemma 3n** - Not directly supported
7. **Qwen 3 Coder** - Variant not directly supported
8. **Qwen 2.5 Omni/VL** - Multimodal variants not directly supported
9. **Qwen 2.5 Coder** - Variant not directly supported
10. **QwQ** - Not directly supported
11. **Qwen 2 VL** - Multimodal variant not directly supported
12. **Mistral Small/Magistral/Devstral** - Variants not directly supported
13. **Pixtral** - Partially supported via `candle_transformers::models::pixtral`
14. **Mistral NeMo/Large** - Variants not directly supported
15. **Phi-4/Phi-3.5** - Variants not directly supported
16. **GLM** - Partially supported via `candle_transformers::models::chatglm` and `glm4`
17. **Hunyuan** - Not directly supported
18. **Orpheus** - Not directly supported
19. **LLava** - Partially supported via `candle_transformers::models::llava`
20. **TinyLlama** - Not directly supported
21. **Zephyr-SFT** - Variant not directly supported

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
2. Gemma 3 (requires adding support)
3. Qwen 2.5 (similar to Qwen2)
4. Pixtral (multimodal, partial support exists)
5. DeepSeek (partial support exists)

### Tier 3 (Low Priority - Future Implementation)
1. All other models not listed above
2. **gpt-oss** - Note that this is an OpenAI model that would require API access rather than local inference