//! Llama GGUF loading

use candle::Device;
use candle::quantized::gguf_file;
use candle_transformers::models::quantized_llama::ModelWeights;
use std::fs::File;

use super::LlamaBackend;

impl LlamaBackend {
    /// Создаёт бекенд из GGUF Content
    pub fn from_gguf(
        content: gguf_file::Content,
        file: &mut File,
        device: &Device,
    ) -> Result<Self, String> {
        let vocab_size = content
            .metadata
            .get("llama.vocab_size")
            .or_else(|| content.metadata.get("mistral.vocab_size"))
            .or_else(|| content.metadata.get("tokenizer.vocab_size"))
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(32000) as usize;

        let max_seq_len = content
            .metadata
            .get("llama.context_length")
            .or_else(|| content.metadata.get("mistral.context_length"))
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(4096) as usize;

        let inner = ModelWeights::from_gguf(content, file, device)
            .map_err(|e| format!("Failed to load Llama GGUF model: {}", e))?;

        Ok(Self::new_quantized(
            inner,
            device.clone(),
            vocab_size,
            max_seq_len,
        ))
    }

    pub fn from_gguf_path(path: &std::path::Path, device: &Device) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open GGUF file: {}", e))?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("Failed to read GGUF header: {}", e))?;
        Self::from_gguf(content, &mut file, device)
    }
}
