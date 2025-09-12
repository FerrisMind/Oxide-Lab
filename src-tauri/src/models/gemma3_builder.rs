//! Gemma 3 model builder (GGUF) на базе официальных примеров candle.
//!
//! Реализует только путь GGUF (квантованные веса). Путь через safetensors
//! для Gemma 3 можно добавить позже при необходимости.

use std::collections::HashMap;
use std::io::{Read, Seek};
use candle::Device;
use candle::{DType};
use candle_nn::VarBuilder;

use crate::models::registry::ArchKind;
use crate::models::common::model::ModelBackend;
use crate::models::gemma3::ModelWeights as Gemma3Gguf;

#[derive(Clone)]
pub struct Gemma3ModelBuilder;

impl Gemma3ModelBuilder {
    pub fn new() -> Self { Self }

    /// Построить модель из GGUF (квантованные веса)
    pub fn from_gguf<R: Read + Seek>(
        &self,
        content: candle::quantized::gguf_file::Content,
        reader: &mut R,
        device: &Device,
        context_length: usize,
        flag: bool,
    ) -> Result<Box<dyn ModelBackend>, String> {
        let model = Gemma3Gguf::from_gguf(content, reader, device, context_length, flag)
            .map_err(|e| format!("Failed to load Gemma3 GGUF model: {}", e))?;
        Ok(Box::new(model))
    }

    /// Построение из VarBuilder (float/safetensors)
    pub fn from_varbuilder(
        &self,
        _vb: VarBuilder,
        _config: &serde_json::Value,
        _device: &Device,
        _dtype: DType,
    ) -> Result<Box<dyn ModelBackend>, String> {
        // Используем реализацию Gemma из candle_transformers
        let vb = _vb;
        let cfg_str = _config.to_string();
        let cfg: candle_transformers::models::gemma::Config = serde_json::from_str(&cfg_str)
            .map_err(|e| format!("Failed to parse Gemma config: {}", e))?;
        let model = candle_transformers::models::gemma::Model::new(false, &cfg, vb)
            .map_err(|e| format!("Failed to build Gemma model: {}", e))?;
        let adapter = crate::models::common::candle_llm::GemmaCandleAdapter::new(model);
        Ok(Box::new(adapter))
    }

    /// Детект архитектуры из GGUF метаданных
    pub fn detect_gguf_arch(&self, metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
        if let Some(arch_value) = metadata.get("general.architecture") {
            if let Ok(arch_str) = arch_value.to_string() {
                let s = arch_str.to_lowercase();
                if s.contains("gemma3") || s == "gemma" || s.contains("gemma") {
                    return Some(ArchKind::Gemma);
                }
            }
        }
        // Фолбэк: эвристика по любым строковым метаданным
        for (_k, v) in metadata.iter() {
            if let Ok(s) = v.to_string() {
                let s = s.to_lowercase();
                if s.contains("gemma3") || s.contains("gemma") {
                    return Some(ArchKind::Gemma);
                }
            }
        }
        None
    }

    /// Детект архитектуры из config.json
    pub fn detect_config_arch(&self, config: &serde_json::Value) -> Option<ArchKind> {
        if let Some(model_type) = config.get("model_type").and_then(|v| v.as_str()) {
            let s = model_type.to_lowercase();
            if s.contains("gemma3") || s == "gemma" || s.contains("gemma") {
                return Some(ArchKind::Gemma);
            }
        }
        if let Some(archs) = config.get("architectures").and_then(|v| v.as_array()) {
            for a in archs {
                if let Some(s) = a.as_str() {
                    let s = s.to_lowercase();
                    if s.contains("gemma3") || s.contains("gemmaforcausallm") || s.contains("gemma") {
                        return Some(ArchKind::Gemma);
                    }
                }
            }
        }
        None
    }

    /// Арх‑метка
    pub fn arch_kind(&self) -> ArchKind { ArchKind::Gemma }
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
        assert_eq!(b.arch_kind(), ArchKind::Gemma);
    }
}
