//! Qwen3 model backend
//!
//! Обёртка над candle_transformers для интеграции Qwen3 с нашим API.
//! Основано на примерах: quantized-qwen3 (GGUF) и qwen (SafeTensors)
//!
//! # Структура
//! - `mod.rs` - общий Qwen3Backend и ModelBackend реализация
//! - `gguf.rs` - загрузка из GGUF формата
//! - `safetensors.rs` - загрузка из SafeTensors формата
//! - `model.rs` - модель с поддержкой flash-attn

mod gguf;
pub mod model;
mod safetensors;

// Re-export types needed by qwen3_moe
pub use model::{Config, Qwen3Attention, Qwen3MLP, Qwen3RotaryEmbedding};

use candle::{Device, Tensor};
use candle_transformers::models::quantized_qwen3::ModelWeights as QuantizedQwen3;

use crate::models::ModelBackend;
use crate::models::api::optimization::{OptimizationConfig, WeightFormat};

// Use our local model with flash-attn support
use model::ModelForCausalLM;

/// Qwen3 бекенд
///
/// Поддерживает как квантизированные (GGUF) так и полные (SafeTensors) модели.
/// SafeTensors модели автоматически используют Flash Attention когда `use_flash_attn = true`.
pub struct Qwen3Backend {
    inner: Qwen3Inner,
    device: Device,
    vocab_size: usize,
    max_seq_len: usize,
    optimization: OptimizationConfig,
}

/// Внутреннее представление модели
enum Qwen3Inner {
    /// Квантизированная модель из GGUF
    Quantized(QuantizedQwen3),
    /// Полная модель из SafeTensors (с опциональным Flash Attention)
    Full(ModelForCausalLM),
}

impl Qwen3Backend {
    /// Создаёт квантизированный бекенд (используется из gguf.rs)
    pub(crate) fn new_quantized(
        model: QuantizedQwen3,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
    ) -> Self {
        Self {
            inner: Qwen3Inner::Quantized(model),
            device,
            vocab_size,
            max_seq_len,
            optimization: OptimizationConfig::for_gguf(),
        }
    }

    /// Создаёт полный бекенд (используется из safetensors.rs)
    /// Flash Attention автоматически включается через config.use_flash_attn
    pub(crate) fn new_full(
        model: ModelForCausalLM,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
        optimization: OptimizationConfig,
    ) -> Self {
        Self {
            inner: Qwen3Inner::Full(model),
            device,
            vocab_size,
            max_seq_len,
            optimization,
        }
    }

    /// Возвращает устройство
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Проверяет, квантизирована ли модель
    pub fn is_quantized(&self) -> bool {
        matches!(self.inner, Qwen3Inner::Quantized(_))
    }

    /// Возвращает конфигурацию оптимизаций
    pub fn optimization(&self) -> &OptimizationConfig {
        &self.optimization
    }
}

impl ModelBackend for Qwen3Backend {
    fn forward(&mut self, input: &Tensor, pos: usize) -> candle::Result<Tensor> {
        match &mut self.inner {
            // GGUF модель возвращает [batch, vocab_size] - только последний токен
            Qwen3Inner::Quantized(model) => model.forward(input, pos),
            // SafeTensors модель возвращает [batch, seq_len, vocab_size]
            // Извлекаем только последний токен для совместимости с генерацией
            Qwen3Inner::Full(model) => {
                let logits = model.forward(input, pos)?;
                let seq_len = logits.dim(1)?;
                // Берём логиты последнего токена: [batch, vocab_size]
                logits.narrow(1, seq_len - 1, 1)?.squeeze(1)
            }
        }
    }

    fn clear_kv_cache(&mut self) {
        match &mut self.inner {
            Qwen3Inner::Quantized(model) => model.clear_kv_cache(),
            Qwen3Inner::Full(model) => model.clear_kv_cache(),
        }
    }

    fn model_type(&self) -> &str {
        match self.optimization.weight_format() {
            WeightFormat::Gguf => "qwen3-gguf",
            WeightFormat::SafeTensors => {
                if self.optimization.uses_flash_attn() {
                    "qwen3-flash"
                } else {
                    "qwen3"
                }
            }
        }
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn max_seq_len(&self) -> usize {
        self.max_seq_len
    }

    fn supports_flash_attn(&self) -> bool {
        self.optimization.uses_flash_attn()
    }
}
