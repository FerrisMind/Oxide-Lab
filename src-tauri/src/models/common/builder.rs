//! Model builder trait and factory implementation.
//!
//! This module provides a unified interface for building models from different sources:
//! - GGUF files
//! - Safetensors files with config
//!
//! The builder pattern allows for consistent model creation regardless of the source format.

use candle::{DType, Device};
use candle_nn::VarBuilder;
use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::models::common::model::ModelBackend;
use crate::models::registry::ArchKind;

/// Result type for model building operations
pub type BuildResult<T> = Result<T, String>;

/// Since we can't make traits with generic methods into trait objects,
/// we'll use an enum-based approach instead
pub enum ModelBuilder {
    Qwen3(crate::models::qwen3::builder::Qwen3ModelBuilder),
    Gemma3(crate::models::gemma3::builder::Gemma3ModelBuilder),
    // Add other model builders here as they are implemented
}

impl ModelBuilder {
    /// Build a model from GGUF content
    pub fn from_gguf<R: Read + Seek>(
        &self,
        content: candle::quantized::gguf_file::Content,
        reader: &mut R,
        device: &Device,
        context_length: usize,
        flag: bool,
    ) -> BuildResult<Box<dyn ModelBackend>> {
        match self {
            ModelBuilder::Qwen3(builder) => {
                builder.from_gguf(content, reader, device, context_length, flag)
            }
            ModelBuilder::Gemma3(builder) => {
                builder.from_gguf(content, reader, device, context_length, flag)
            }
        }
    }

    /// Build a model from VarBuilder and config
    pub fn from_varbuilder(
        &self,
        vb: VarBuilder,
        config: &serde_json::Value,
        device: &Device,
        dtype: DType,
    ) -> BuildResult<Box<dyn ModelBackend>> {
        match self {
            ModelBuilder::Qwen3(builder) => builder.from_varbuilder(vb, config, device, dtype),
            ModelBuilder::Gemma3(builder) => builder.from_varbuilder(vb, config, device, dtype),
        }
    }

    /// Detect architecture from GGUF metadata
    pub fn detect_gguf_arch(
        &self,
        metadata: &HashMap<String, candle::quantized::gguf_file::Value>,
    ) -> Option<ArchKind> {
        match self {
            ModelBuilder::Qwen3(builder) => builder.detect_gguf_arch(metadata),
            ModelBuilder::Gemma3(builder) => builder.detect_gguf_arch(metadata),
        }
    }

    /// Detect architecture from config JSON
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        match self {
            ModelBuilder::Qwen3(builder) => builder.detect_config_arch(config),
            ModelBuilder::Gemma3(builder) => builder.detect_config_arch(config),
        }
    }

    /// Get the architecture kind this builder supports
    pub fn arch_kind(&self) -> ArchKind {
        match self {
            ModelBuilder::Qwen3(builder) => builder.arch_kind(),
            ModelBuilder::Gemma3(builder) => builder.arch_kind(),
        }
    }
}

/// Model factory that manages model builders for different architectures
pub struct ModelFactory {
    builders: HashMap<ArchKind, ModelBuilder>,
}

impl ModelFactory {
    /// Create a new model factory
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }

    /// Register a builder for an architecture
    pub fn register_builder(&mut self, builder: ModelBuilder) {
        let arch_kind = builder.arch_kind();
        self.builders.insert(arch_kind, builder);
    }

    /// Register an explicit alias mapping: use the given builder to handle a specified `arch`.
    /// Useful when the same loader implementation supports multiple closely related architectures
    /// (e.g., Gemma and Gemma3) but we want distinct ArchKind values for policy decisions.
    pub fn register_builder_for_arch(&mut self, arch: ArchKind, builder: ModelBuilder) {
        self.builders.insert(arch, builder);
    }

    /// Build a model from GGUF content
    pub fn build_from_gguf<R: Read + Seek>(
        &self,
        arch: ArchKind,
        content: candle::quantized::gguf_file::Content,
        reader: &mut R,
        device: &Device,
        context_length: usize,
        flag: bool,
    ) -> BuildResult<Box<dyn ModelBackend>> {
        self.builders
            .get(&arch)
            .ok_or_else(|| format!("No builder registered for architecture {:?}", arch))?
            .from_gguf(content, reader, device, context_length, flag)
    }

    /// Build a model from safetensors files and config
    pub fn build_from_safetensors(
        &self,
        arch: ArchKind,
        filenames: &[String],
        config: &serde_json::Value,
        device: &Device,
        dtype: DType,
    ) -> BuildResult<Box<dyn ModelBackend>> {
        let vb = crate::core::weights::build_varbuilder(filenames, device)
            .map_err(|e| format!("Failed to build VarBuilder: {}", e))?;

        self.builders
            .get(&arch)
            .ok_or_else(|| format!("No builder registered for architecture {:?}", arch))?
            .from_varbuilder(vb, config, device, dtype)
    }

    /// Detect architecture from GGUF metadata
    pub fn detect_gguf_arch(
        &self,
        metadata: &HashMap<String, candle::quantized::gguf_file::Value>,
    ) -> Option<ArchKind> {
        // Try each builder's detection method
        for builder in self.builders.values() {
            if let Some(arch) = builder.detect_gguf_arch(metadata) {
                return Some(arch);
            }
        }
        None
    }

    /// Detect architecture from config JSON
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        // Try each builder's detection method
        for builder in self.builders.values() {
            if let Some(arch) = builder.detect_config_arch(config) {
                return Some(arch);
            }
        }
        None
    }
}

impl Default for ModelFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = ModelFactory::new();
        assert_eq!(factory.builders.len(), 0);
    }
}
