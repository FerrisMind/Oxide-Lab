use std::collections::HashMap;
use std::sync::OnceLock;
use crate::models::common::builder::ModelFactory;
use crate::models::qwen3::builder::Qwen3ModelBuilder;
use crate::models::gemma3::builder::Gemma3ModelBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArchKind {
    // Models from the user's list that are supported by Candle
    Llama,      // Covers Llama 2, 3, 3.1, 3.2, 3.3, 4 and CodeLlama
    Mistral,    // Covers Mistral 7B, Mistral Small, Mistral NeMo, Mistral Large
    Mixtral,
    Gemma,      // Gemma 2 (text-only in this app)
    Gemma3,     // Gemma 3 (multimodal: text + image)
    Qwen3,      // Covers Qwen 2, 2.5, 3, 3 Coder
    Yi,
    Phi3,       // Covers Phi-3, Phi-3.5
    DeepSeek,   // Covers DeepSeek-R1 variants
    Pixtral,
    SmolLM2,    // SmolLM 2
    // Removed architectures not in the user's list:
    // Falcon, OLMo, StarCoder2, Arctic, Cohere, CommandR, DBRX, 
    // Granite, GraniteMoE, InternLM2, Jais, JinaBert, JinaReranker, JinaEmbeddings, Minicpm, Mpt
}

/// Global model factory instance
static MODEL_FACTORY: OnceLock<ModelFactory> = OnceLock::new();

/// Get the global model factory instance
pub fn get_model_factory() -> &'static ModelFactory {
    MODEL_FACTORY.get_or_init(|| {
        let mut factory = ModelFactory::new();
        
        // Register Qwen3 builder
        factory.register_builder(crate::models::common::builder::ModelBuilder::Qwen3(Qwen3ModelBuilder::new()));
        // Register Gemma3 builder
        factory.register_builder(crate::models::common::builder::ModelBuilder::Gemma3(Gemma3ModelBuilder::new()));
        // Alias: use Gemma3 builder for Gemma (text-only policy applies elsewhere)
        factory.register_builder_for_arch(
            ArchKind::Gemma,
            crate::models::common::builder::ModelBuilder::Gemma3(Gemma3ModelBuilder::new()),
        );
        
        // TODO: Register other builders as they are implemented
        // factory.register_builder(crate::models::common::builder::ModelBuilder::Llama(LlamaModelBuilder::new()));
        // factory.register_builder(crate::models::common::builder::ModelBuilder::Mistral(MistralModelBuilder::new()));
        // etc.
        
        factory
    })
}

pub fn detect_arch(metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
    // Use the model factory to detect the architecture
    get_model_factory().detect_gguf_arch(metadata)
}

/// Detect architecture from config JSON
pub fn detect_arch_from_config(config: &serde_json::Value) -> Option<ArchKind> {
    // Use the model factory to detect the architecture
    get_model_factory().detect_config_arch(config)
}
