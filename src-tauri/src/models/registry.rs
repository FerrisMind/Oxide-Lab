use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ArchKind {
    // Models from the user's list that are supported by Candle
    Llama,      // Covers Llama 2, 3, 3.1, 3.2, 3.3, 4 and CodeLlama
    Mistral,    // Covers Mistral 7B, Mistral Small, Mistral NeMo, Mistral Large
    Mixtral,
    Gemma,      // Covers Gemma 2, Gemma 3
    Qwen3,      // Covers Qwen 2, 2.5, 3, 3 Coder
    Yi,
    Phi3,       // Covers Phi-3, Phi-3.5
    DeepSeek,   // Covers DeepSeek-R1 variants
    Pixtral,
    SmolLM2,    // SmolLM 2
    // Removed architectures not in the user's list:
    // Falcon, OLMo, Phi, Gemma2, StarCoder2, Arctic, Cohere, CommandR, DBRX, 
    // Granite, GraniteMoE, InternLM2, Jais, JinaBert, JinaReranker, JinaEmbeddings, Minicpm, Mpt
}

pub fn detect_arch(metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
    // First, try to detect from the general.architecture field (standard GGUF approach)
    if let Some(arch_value) = metadata.get("general.architecture") {
        if let Ok(arch_str) = arch_value.to_string() {
            return match arch_str.to_lowercase().as_str() {
                "llama" | "llama2" | "llama3" => Some(ArchKind::Llama),
                "mistral" | "mistralai" => Some(ArchKind::Mistral),
                "mixtral" => Some(ArchKind::Mixtral),
                "gemma" => Some(ArchKind::Gemma),
                "qwen2" | "qwen3" => Some(ArchKind::Qwen3),
                "yi" => Some(ArchKind::Yi),
                "phi3" => Some(ArchKind::Phi3),
                "deepseek" => Some(ArchKind::DeepSeek),
                "pixtral" => Some(ArchKind::Pixtral),
                "smollm2" => Some(ArchKind::SmolLM2),
                _ => None,
            };
        }
    }
    
    // Fallback: try to detect from model-specific fields or heuristics
    for (_k, v) in metadata.iter() {
        if let Ok(s) = v.to_string() {
            let s_lower = s.to_lowercase();
            if s_lower.contains("llama") {
                return Some(ArchKind::Llama);
            }
            if s_lower.contains("mistral") {
                return Some(ArchKind::Mistral);
            }
            if s_lower.contains("mixtral") {
                return Some(ArchKind::Mixtral);
            }
            if s_lower.contains("gemma") {
                return Some(ArchKind::Gemma);
            }
            if s_lower.contains("qwen") {
                return Some(ArchKind::Qwen3);
            }
            if s_lower.contains("yi") {
                return Some(ArchKind::Yi);
            }
            if s_lower.contains("phi3") {
                return Some(ArchKind::Phi3);
            }
            if s_lower.contains("deepseek") {
                return Some(ArchKind::DeepSeek);
            }
            if s_lower.contains("pixtral") {
                return Some(ArchKind::Pixtral);
            }
            if s_lower.contains("smollm2") {
                return Some(ArchKind::SmolLM2);
            }
        }
    }
    
    None
}
