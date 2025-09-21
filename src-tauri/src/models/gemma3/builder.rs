//! Gemma 3 model builder (GGUF) on top of candle examples.
//! Implements GGUF path; safetensors path can be added later if needed.

use std::collections::HashMap;
use std::io::{Read, Seek};
use candle::Device;
use candle::DType;
use candle_nn::VarBuilder;

use crate::models::registry::ArchKind;
use crate::models::common::model::ModelBackend;
use crate::models::gemma3::model::ModelWeights as Gemma3Gguf;

#[derive(Clone)]
pub struct Gemma3ModelBuilder;

impl Gemma3ModelBuilder {
    pub fn new() -> Self { Self }

    /// Build a model from GGUF (quantized weights)
    pub fn from_gguf<R: Read + Seek>(
        &self,
        content: candle::quantized::gguf_file::Content,
        reader: &mut R,
        device: &Device,
        context_length: usize,
        flag: bool,
    ) -> Result<Box<dyn ModelBackend>, String> {
        // Guard: позволяем GGUF только для архитектуры gemma3
        if let Some(v) = content.metadata.get("general.architecture") {
            if let Ok(s) = v.to_string() {
                let s = s.to_lowercase();
                if !s.contains("gemma3") {
                    return Err("GGUF: обнаружена архитектура, отличная от 'gemma3' (для 'gemma' используйте safetensors/float путь)".to_string());
                }
            }
        }
        let model = Gemma3Gguf::from_gguf(content, reader, device, context_length, flag)
            .map_err(|e| format!("Failed to load Gemma3 GGUF model: {}", e))?;
        Ok(Box::new(model))
    }

    /// Build from VarBuilder and config (float/safetensors)
    pub fn from_varbuilder(
        &self,
        vb: VarBuilder,
        config: &serde_json::Value,
        _device: &Device,
        _dtype: DType,
    ) -> Result<Box<dyn ModelBackend>, String> {
        let _ = (vb, config);
        Err("from_varbuilder for Gemma is disabled (no adapters)".into())
    }

    /// Detect architecture from GGUF metadata
    pub fn detect_gguf_arch(&self, metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
        // Prefer exact gemma3 detection; plain "gemma" maps to Gemma (text-only here)
        if let Some(arch_value) = metadata.get("general.architecture") {
            if let Ok(arch_str) = arch_value.to_string() {
                let s = arch_str.to_lowercase();
                if s.contains("gemma3") { return Some(ArchKind::Gemma3); }
                if s == "gemma" || (s.contains("gemma") && !s.contains("gemma3")) { return Some(ArchKind::Gemma); }
            }
        }
        // Fallback: heuristic over all string metadata
        for (_k, v) in metadata.iter() {
            if let Ok(s) = v.to_string() {
                let s = s.to_lowercase();
                if s.contains("gemma3") { return Some(ArchKind::Gemma3); }
                if s.contains("gemma") { return Some(ArchKind::Gemma); }
            }
        }
        None
    }

    /// Detect architecture from config JSON
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        if let Some(model_type) = config.get("model_type").and_then(|v| v.as_str()) {
            let s = model_type.to_lowercase();
            if s.contains("gemma3") { return Some(ArchKind::Gemma3); }
            if s == "gemma" || (s.contains("gemma") && !s.contains("gemma3")) { return Some(ArchKind::Gemma); }
        }
        if let Some(archs) = config.get("architectures").and_then(|v| v.as_array()) {
            for a in archs {
                if let Some(s) = a.as_str() {
                    let s = s.to_lowercase();
                    if s.contains("gemma3") || s.contains("gemma3forcausallm") { return Some(ArchKind::Gemma3); }
                    if s.contains("gemmaforcausallm") || (s.contains("gemma") && !s.contains("gemma3")) { return Some(ArchKind::Gemma); }
                }
            }
        }
        None
    }

    pub fn arch_kind(&self) -> ArchKind { ArchKind::Gemma3 }
}

impl Default for Gemma3ModelBuilder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_builder_creation() {
        let b = Gemma3ModelBuilder::new();
        assert_eq!(b.arch_kind(), ArchKind::Gemma3);
    }
}
