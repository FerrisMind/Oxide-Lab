//! Qwen3 model builder implementation.
//!
//! This module provides concrete implementations of the ModelBuilder trait
//! for the Qwen3 architecture, supporting both GGUF and safetensors formats.

use std::collections::HashMap;
use std::io::{Read, Seek};
use candle::{Device, DType};
use candle_nn::VarBuilder;

use crate::models::registry::ArchKind;
use crate::models::common::model::ModelBackend;
use crate::models::qwen3::ModelWeights as Qwen3Gguf;
use crate::models::common::candle_llm::Qwen3CandleAdapter;

/// Qwen3 model builder implementation
#[derive(Clone)]
pub struct Qwen3ModelBuilder;

impl Qwen3ModelBuilder {
    /// Create a new Qwen3 model builder
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
        // Build the model from GGUF content
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
        // Parse the config to create the model configuration
        let config_str = config.to_string();
        let qwen_config: candle_transformers::models::qwen3::Config = 
            serde_json::from_str(&config_str)
                .map_err(|e| format!("Failed to parse Qwen3 config: {}", e))?;
        
        // Build the model from the VarBuilder and config
        let model = candle_transformers::models::qwen3::ModelForCausalLM::new(&qwen_config, vb)
            .map_err(|e| format!("Failed to load Qwen3 model: {}", e))?;
        
        // Wrap in our adapter
        let adapter = Qwen3CandleAdapter::new(model);
        
        Ok(Box::new(adapter))
    }
    
    /// Detect architecture from GGUF metadata
    pub fn detect_gguf_arch(&self, metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
        // First, try to detect from the general.architecture field (standard GGUF approach)
        if let Some(arch_value) = metadata.get("general.architecture") {
            if let Ok(arch_str) = arch_value.to_string() {
                match arch_str.to_lowercase().as_str() {
                    "qwen2" | "qwen3" => return Some(ArchKind::Qwen3),
                    _ => {}
                }
            }
        }
        
        // Fallback: try to detect from model-specific fields or heuristics
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
        // Try to detect from model_type field
        if let Some(model_type) = config.get("model_type") {
            if let Some(model_type_str) = model_type.as_str() {
                match model_type_str.to_lowercase().as_str() {
                    "qwen2" | "qwen3" => return Some(ArchKind::Qwen3),
                    _ => {}
                }
            }
        }
        
        // Try to detect from architectures field
        if let Some(architectures) = config.get("architectures") {
            if let Some(arch_array) = architectures.as_array() {
                for arch in arch_array {
                    if let Some(arch_str) = arch.as_str() {
                        match arch_str.to_lowercase().as_str() {
                            "qwen2forcausallm" | "qwen3forcausallm" => return Some(ArchKind::Qwen3),
                            _ => {}
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Get the architecture kind this builder supports
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