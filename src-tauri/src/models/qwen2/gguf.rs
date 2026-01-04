//! Qwen2/2.5 GGUF loading
//!
//! Загрузка квантизированных Qwen2/2.5 моделей из GGUF формата.
//! Использует candle_transformers::models::quantized_qwen2.

use candle::Device;
use candle::quantized::gguf_file;
use candle_transformers::models::quantized_qwen2::ModelWeights;
use std::fs::File;

use super::Qwen2Backend;

impl Qwen2Backend {
    /// Создаёт бекенд из GGUF Content
    pub fn from_gguf(
        content: gguf_file::Content,
        file: &mut File,
        device: &Device,
    ) -> Result<Self, String> {
        // Извлекаем метаданные - Qwen2 использует qwen2.* префикс
        let vocab_size = content
            .metadata
            .get("qwen2.vocab_size")
            .or_else(|| content.metadata.get("tokenizer.vocab_size"))
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(151936) as usize;

        let max_seq_len = content
            .metadata
            .get("qwen2.context_length")
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(32768) as usize;

        log::info!(
            "Loading Qwen2/2.5 GGUF: vocab_size={}, max_seq_len={}",
            vocab_size,
            max_seq_len
        );

        // Создаём модель
        let inner = ModelWeights::from_gguf(content, file, device)
            .map_err(|e| format!("Failed to load Qwen2 GGUF model: {}", e))?;

        Ok(Self::new_quantized(
            inner,
            device.clone(),
            vocab_size,
            max_seq_len,
        ))
    }

    /// Создаёт бекенд из пути к GGUF файлу
    pub fn from_gguf_path(path: &std::path::Path, device: &Device) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open GGUF file: {}", e))?;

        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("Failed to read GGUF header: {}", e))?;

        Self::from_gguf(content, &mut file, device)
    }
}
