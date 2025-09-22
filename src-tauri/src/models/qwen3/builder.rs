//! Qwen3 model builder implementation.
//! Provides concrete implementations of the ModelBuilder trait
//! for the Qwen3 architecture, supporting GGUF and safetensors.

use candle::{DType, Device};
use candle_nn::VarBuilder;
use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::models::common::model::ModelBackend;
use crate::models::qwen3::model::ModelWeights as Qwen3Gguf;
use crate::models::registry::ArchKind;

#[derive(Clone)]
pub struct Qwen3ModelBuilder;

impl Qwen3ModelBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build a model from GGUF content
    pub fn from_gguf<R: Read + Seek>(
        &self,
        content: candle::quantized::gguf_file::Content,
        reader: &mut R,
        device: &Device,
        context_length: usize,
        flag: bool,
    ) -> Result<Box<dyn ModelBackend>, String> {
        let model = Qwen3Gguf::from_gguf(content, reader, device, context_length, flag)
            .map_err(|e| format!("Failed to load Qwen3 GGUF model: {}", e))?;
        Ok(Box::new(model))
    }

    /// Build a model from VarBuilder and config
    pub fn from_varbuilder(
        &self,
        vb: VarBuilder,
        config: &serde_json::Value,
        _device: &Device,
        _dtype: DType,
    ) -> Result<Box<dyn ModelBackend>, String> {
        let _ = (vb, config);
        Err("from_varbuilder for Qwen3 is disabled (no adapters)".into())
    }

    /// Detect architecture from GGUF metadata
    pub fn detect_gguf_arch(
        &self,
        metadata: &HashMap<String, candle::quantized::gguf_file::Value>,
    ) -> Option<ArchKind> {
        if let Some(arch_value) = metadata.get("general.architecture") {
            if let Ok(arch_str) = arch_value.to_string() {
                match arch_str.to_lowercase().as_str() {
                    "qwen2" | "qwen3" => return Some(ArchKind::Qwen3),
                    _ => {}
                }
            }
        }
        // Fallback
        for (_k, v) in metadata.iter() {
            if let Ok(s) = v.to_string() {
                let s_lower = s.to_lowercase();
                if s_lower.contains("qwen") {
                    return Some(ArchKind::Qwen3);
                }
            }
        }
        None
    }

    /// Detect architecture from config JSON
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        if let Some(model_type) = config.get("model_type").and_then(|v| v.as_str()) {
            match model_type.to_lowercase().as_str() {
                "qwen2" | "qwen3" => return Some(ArchKind::Qwen3),
                _ => {}
            }
        }
        if let Some(architectures) = config.get("architectures").and_then(|v| v.as_array()) {
            for arch in architectures {
                if let Some(arch_str) = arch.as_str() {
                    match arch_str.to_lowercase().as_str() {
                        "qwen2forcausallm" | "qwen3forcausallm" => return Some(ArchKind::Qwen3),
                        _ => {}
                    }
                }
            }
        }
        None
    }

    pub fn arch_kind(&self) -> ArchKind {
        ArchKind::Qwen3
    }
}

impl Default for Qwen3ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_builder_creation() {
        let builder = Qwen3ModelBuilder::new();
        assert_eq!(builder.arch_kind(), ArchKind::Qwen3);
    }
}
