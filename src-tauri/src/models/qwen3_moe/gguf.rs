//! Qwen3-MoE GGUF loading
//!
//! Загрузка квантизированных Qwen3-MoE моделей из GGUF формата.
//! Основано на примере: src-tauri/src/models/example/quantized-qwen3-moe/main.rs

use candle::quantized::gguf_file;
use candle::{DType, Device};
use std::fs::File;

use super::Qwen3MoeBackend;
use super::quantized_model::GGUFQWenMoE;

impl Qwen3MoeBackend {
    /// Создаёт бекенд из GGUF Content
    /// dtype - тип данных для вычислений (BF16 или F16)
    pub fn from_gguf(
        content: gguf_file::Content,
        file: &mut File,
        device: &Device,
        dtype: DType,
    ) -> Result<Self, String> {
        // Извлекаем метаданные из GGUF
        let vocab_size = content
            .metadata
            .get("qwen3_moe.vocab_size")
            .or_else(|| content.metadata.get("qwen3.vocab_size"))
            .or_else(|| content.metadata.get("tokenizer.vocab_size"))
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(151936) as usize;

        let max_seq_len = content
            .metadata
            .get("qwen3_moe.context_length")
            .or_else(|| content.metadata.get("qwen3.context_length"))
            .and_then(|v| v.to_u32().ok())
            .unwrap_or(40960) as usize;

        log::info!(
            "Loading Qwen3-MoE GGUF: vocab_size={}, max_seq_len={}, dtype={:?}",
            vocab_size,
            max_seq_len,
            dtype
        );

        // Создаём модель из локальной версии с поддержкой clear_kv_cache
        let inner = GGUFQWenMoE::from_gguf(content, file, device, dtype)
            .map_err(|e| format!("Failed to load Qwen3-MoE GGUF model: {}", e))?;

        Ok(Self::new_quantized(
            inner,
            device.clone(),
            vocab_size,
            max_seq_len,
        ))
    }

    /// Создаёт бекенд из пути к GGUF файлу
    pub fn from_gguf_path(
        path: &std::path::Path,
        device: &Device,
        dtype: DType,
    ) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open GGUF file: {}", e))?;

        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("Failed to read GGUF header: {}", e))?;

        Self::from_gguf(content, &mut file, device, dtype)
    }
}
