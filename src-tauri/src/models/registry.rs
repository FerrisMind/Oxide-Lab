//! Model registry - регистрация и автоопределение архитектур моделей

use candle::quantized::gguf_file::Value;
use std::collections::HashMap;

/// Поддерживаемые архитектуры моделей
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchKind {
    Llama,    // Llama 1/2/3/4, Mistral, Mixtral, Yi, DeepSeek, SmolLM, CodeLlama, etc.
    Gemma,    // Gemma 1
    Gemma2,   // Gemma 2
    Gemma3,   // Gemma 3
    Qwen2,    // Qwen 2, Qwen 2.5
    Qwen2Moe, // Qwen 2 MoE
    Qwen3,    // Qwen 3
    Qwen3Moe, // Qwen 3 MoE (30B-A3B)
    Phi,      // Phi 1, 1.5, 2
    Phi3,     // Phi-3, Phi-3.5, Phi-4
    Glm4,     // GLM-4
}

impl ArchKind {
    /// Возвращает человекочитаемое название
    pub fn display_name(&self) -> &'static str {
        match self {
            ArchKind::Llama => "Llama",
            ArchKind::Gemma => "Gemma",
            ArchKind::Gemma2 => "Gemma 2",
            ArchKind::Gemma3 => "Gemma 3",
            ArchKind::Qwen2 => "Qwen 2",
            ArchKind::Qwen2Moe => "Qwen 2 MoE",
            ArchKind::Qwen3 => "Qwen 3",
            ArchKind::Qwen3Moe => "Qwen 3 MoE",
            ArchKind::Phi => "Phi",
            ArchKind::Phi3 => "Phi-3",
            ArchKind::Glm4 => "GLM-4",
        }
    }

    /// Проверяет, поддерживается ли GGUF формат
    pub fn supports_gguf(&self) -> bool {
        matches!(
            self,
            ArchKind::Llama
                | ArchKind::Gemma
                | ArchKind::Gemma2
                | ArchKind::Gemma3
                | ArchKind::Qwen2
                | ArchKind::Qwen3
                | ArchKind::Qwen3Moe
                | ArchKind::Phi3
        )
    }

    /// Проверяет, поддерживается ли SafeTensors формат
    pub fn supports_safetensors(&self) -> bool {
        true // Все архитектуры поддерживают SafeTensors
    }
}

/// Определяет архитектуру из GGUF метаданных
pub fn detect_arch(metadata: &HashMap<String, Value>) -> Option<ArchKind> {
    // Проверяем поле general.architecture
    let arch_str = metadata.get("general.architecture").and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    })?;

    detect_arch_from_string(arch_str)
}

/// Определяет архитектуру из config.json
pub fn detect_arch_from_config(config: &serde_json::Value) -> Option<ArchKind> {
    // Проверяем model_type
    let model_type = config.get("model_type")?.as_str()?;

    detect_arch_from_string(model_type)
}

/// Определяет архитектуру из строки
fn detect_arch_from_string(s: &str) -> Option<ArchKind> {
    let s_lower = s.to_lowercase();

    // Порядок важен - более специфичные первыми
    // Qwen3 MoE: exact matches (safetensors: "qwen3_moe", gguf: "qwen3moe") or pattern matching
    if s_lower == "qwen3_moe"
        || s_lower == "qwen3moe"
        || ((s_lower.contains("qwen3") || s_lower.contains("qwen-3"))
            && (s_lower.contains("moe") || s_lower.contains("a3b")))
    {
        Some(ArchKind::Qwen3Moe)
    } else if s_lower == "qwen3" || s_lower.contains("qwen3") || s_lower.contains("qwen-3") {
        Some(ArchKind::Qwen3)
    } else if s_lower == "qwen2_moe"
        || s_lower == "qwen2moe"
        || ((s_lower.contains("qwen2") || s_lower.contains("qwen-2")) && (s_lower.contains("moe")))
    {
        Some(ArchKind::Qwen2Moe)
    } else if s_lower == "qwen2"
        || s_lower.contains("qwen2")
        || s_lower.contains("qwen-2")
        || s_lower.contains("qwen2.5")
    {
        Some(ArchKind::Qwen2)
    } else if s_lower.contains("gemma3") || s_lower.contains("gemma-3") {
        Some(ArchKind::Gemma3)
    } else if s_lower.contains("gemma2") || s_lower.contains("gemma-2") {
        Some(ArchKind::Gemma2)
    } else if s_lower.contains("gemma") {
        Some(ArchKind::Gemma)
    } else if s_lower.contains("llama") {
        Some(ArchKind::Llama)
    } else if s_lower.contains("phi-3") || s_lower.contains("phi3") {
        Some(ArchKind::Phi3)
    } else if s_lower.contains("phi") {
        Some(ArchKind::Phi)
    } else if s_lower.contains("glm4") || s_lower.contains("glm-4") {
        Some(ArchKind::Glm4)
    } else {
        None
    }
}

/// Информация о модели из GGUF
#[derive(Debug, Clone)]
pub struct GgufModelInfo {
    pub arch: Option<ArchKind>,
    pub name: Option<String>,
    pub context_length: Option<usize>,
    pub vocab_size: Option<usize>,
    pub hidden_size: Option<usize>,
    pub num_layers: Option<usize>,
    pub num_heads: Option<usize>,
}

impl GgufModelInfo {
    /// Извлекает информацию из GGUF метаданных
    pub fn from_metadata(metadata: &HashMap<String, Value>) -> Self {
        let arch = detect_arch(metadata);

        let name = metadata.get("general.name").and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });

        let context_length = metadata
            .get("llama.context_length")
            .or_else(|| metadata.get("qwen2.context_length"))
            .or_else(|| metadata.get("gemma.context_length"))
            .and_then(|v| match v {
                Value::U32(n) => Some(*n as usize),
                Value::U64(n) => Some(*n as usize),
                _ => None,
            });

        let vocab_size = metadata.get("tokenizer.vocab_size").and_then(|v| match v {
            Value::U32(n) => Some(*n as usize),
            Value::U64(n) => Some(*n as usize),
            _ => None,
        });

        Self {
            arch,
            name,
            context_length,
            vocab_size,
            hidden_size: None,
            num_layers: None,
            num_heads: None,
        }
    }
}

use super::ModelBackend;
use candle::Device;
use candle::quantized::gguf_file::Content;
use std::sync::OnceLock;

/// Фабрика моделей - создаёт модели из различных форматов
pub struct ModelFactory {
    // В будущем здесь будут зарегистрированные билдеры
}

impl ModelFactory {
    pub fn new() -> Self {
        Self {}
    }

    /// Создаёт модель из GGUF
    pub fn build_from_gguf(
        &self,
        arch: ArchKind,
        content: Content,
        file: &mut std::fs::File,
        device: &Device,
        _context_length: usize,
        _use_flash_attn: bool,
    ) -> Result<Box<dyn ModelBackend + Send>, String> {
        match arch {
            ArchKind::Qwen3 => {
                use super::qwen3::Qwen3Backend;
                let model = Qwen3Backend::from_gguf(content, file, device)?;
                Ok(Box::new(model))
            }
            ArchKind::Qwen2 => {
                use super::qwen2::Qwen2Backend;
                let model = Qwen2Backend::from_gguf(content, file, device)?;
                Ok(Box::new(model))
            }
            ArchKind::Qwen3Moe => {
                use super::qwen3_moe::Qwen3MoeBackend;
                // Default to BF16 for MoE GGUF models
                let dtype = if device.is_cuda() || device.is_metal() {
                    candle::DType::BF16
                } else {
                    candle::DType::F32
                };
                let model = Qwen3MoeBackend::from_gguf(content, file, device, dtype)?;
                Ok(Box::new(model))
            }
            // Llama-подобные архитектуры (Llama, Mistral, Mixtral, DeepSeek, Yi, SmolLM2)
            // LlamaVariant определяется автоматически из metadata
            ArchKind::Llama => {
                use super::llama::LlamaBackend;
                let model = LlamaBackend::from_gguf(content, file, device)?;
                Ok(Box::new(model))
            }
            // TODO: Phi3, Gemma
            _ => Err(format!(
                "Model building for {:?} is not yet implemented. \
                 Currently supported: Qwen2, Qwen3, Qwen3Moe, Llama (incl. Mistral, Mixtral, DeepSeek, Yi, SmolLM2)",
                arch
            )),
        }
    }

    /// Создаёт модель из SafeTensors
    pub fn build_from_safetensors<P: AsRef<std::path::Path>>(
        &self,
        arch: ArchKind,
        files: &[P],
        config: &serde_json::Value,
        device: &Device,
        dtype: candle::DType,
    ) -> Result<Box<dyn ModelBackend + Send>, String> {
        // Получаем config_path из первого файла (ищем config.json в той же директории)
        let config_path = files
            .first()
            .and_then(|f| f.as_ref().parent())
            .map(|p| p.join("config.json"))
            .ok_or("No files provided")?;

        let filenames: Vec<std::path::PathBuf> =
            files.iter().map(|p| p.as_ref().to_path_buf()).collect();

        let _ = config; // config уже загружен, используем config_path

        match arch {
            ArchKind::Qwen3 => {
                use super::qwen3::Qwen3Backend;
                let model =
                    Qwen3Backend::from_safetensors(&filenames, &config_path, device, dtype)?;
                Ok(Box::new(model))
            }
            ArchKind::Qwen2 => {
                use super::qwen2::Qwen2Backend;
                let model =
                    Qwen2Backend::from_safetensors(&filenames, &config_path, device, dtype)?;
                Ok(Box::new(model))
            }
            ArchKind::Qwen3Moe => {
                use super::qwen3_moe::Qwen3MoeBackend;
                let model =
                    Qwen3MoeBackend::from_safetensors(&filenames, &config_path, device, dtype)?;
                Ok(Box::new(model))
            }
            ArchKind::Qwen2Moe => {
                use super::qwen2_moe::Qwen2MoeBackend;
                let model =
                    Qwen2MoeBackend::from_safetensors(&filenames, &config_path, device, dtype)?;
                Ok(Box::new(model))
            }
            ArchKind::Llama => {
                use super::llama::LlamaBackend;
                let model =
                    LlamaBackend::from_safetensors(&filenames, &config_path, device, dtype)?;
                Ok(Box::new(model))
            }
            _ => Err(format!(
                "SafeTensors model building for {:?} is not yet implemented. \
                 Currently supported: Qwen2, Qwen2Moe, Qwen3, Qwen3Moe, Llama (incl. Mistral, Mixtral, etc.)",
                arch
            )),
        }
    }

    /// Определяет архитектуру из GGUF метаданных
    pub fn detect_gguf_arch(&self, metadata: &HashMap<String, Value>) -> Option<ArchKind> {
        detect_arch(metadata)
    }

    /// Определяет архитектуру из config.json
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        detect_arch_from_config(config)
    }
}

impl Default for ModelFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр ModelFactory
static MODEL_FACTORY: OnceLock<ModelFactory> = OnceLock::new();

/// Получает глобальный экземпляр ModelFactory
pub fn get_model_factory() -> &'static ModelFactory {
    MODEL_FACTORY.get_or_init(ModelFactory::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_arch_from_string() {
        assert_eq!(detect_arch_from_string("llama"), Some(ArchKind::Llama));
        assert_eq!(detect_arch_from_string("Qwen3"), Some(ArchKind::Qwen3));
        assert_eq!(detect_arch_from_string("gemma3"), Some(ArchKind::Gemma3));
        assert_eq!(detect_arch_from_string("mistral"), Some(ArchKind::Llama));
        assert_eq!(detect_arch_from_string("phi-3"), Some(ArchKind::Phi3));
        assert_eq!(detect_arch_from_string("unknown"), None);
    }
}
