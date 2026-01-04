//! Qwen2/2.5 model backend
//!
//! Обёртка над candle_transformers для интеграции Qwen2/2.5 с нашим API.
//! Архитектура Qwen2/2.5 отличается от Qwen3 отсутствием per-head RMSNorm (q_norm/k_norm).
//!
//! # Структура
//! - `mod.rs` - общий Qwen2Backend и ModelBackend реализация
//! - `gguf.rs` - загрузка из GGUF формата
//! - `safetensors.rs` - загрузка из SafeTensors формата

mod gguf;
mod safetensors;

use candle::{Device, Tensor};
use candle_transformers::models::quantized_qwen2::ModelWeights as QuantizedQwen2;
use candle_transformers::models::qwen2::ModelForCausalLM;

use crate::models::ModelBackend;
use crate::models::api::optimization::{OptimizationConfig, WeightFormat};

/// Qwen2/2.5 бекенд
///
/// Поддерживает как квантизированные (GGUF) так и полные (SafeTensors) модели.
/// Основное отличие от Qwen3: нет per-head RMSNorm в attention.
pub struct Qwen2Backend {
    inner: Qwen2Inner,
    device: Device,
    vocab_size: usize,
    max_seq_len: usize,
    optimization: OptimizationConfig,
}

/// Внутреннее представление модели
enum Qwen2Inner {
    /// Квантизированная модель из GGUF
    Quantized(QuantizedQwen2),
    /// Полная модель из SafeTensors
    Full(ModelForCausalLM),
}

impl Qwen2Backend {
    /// Создаёт квантизированный бекенд (используется из gguf.rs)
    pub(crate) fn new_quantized(
        model: QuantizedQwen2,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
    ) -> Self {
        Self {
            inner: Qwen2Inner::Quantized(model),
            device,
            vocab_size,
            max_seq_len,
            optimization: OptimizationConfig::for_gguf(),
        }
    }

    /// Создаёт полный бекенд (используется из safetensors.rs)
    pub(crate) fn new_full(
        model: ModelForCausalLM,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
        optimization: OptimizationConfig,
    ) -> Self {
        Self {
            inner: Qwen2Inner::Full(model),
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
        matches!(self.inner, Qwen2Inner::Quantized(_))
    }

    /// Возвращает конфигурацию оптимизаций
    pub fn optimization(&self) -> &OptimizationConfig {
        &self.optimization
    }
}

impl ModelBackend for Qwen2Backend {
    fn forward(&mut self, input: &Tensor, pos: usize) -> candle::Result<Tensor> {
        match &mut self.inner {
            // GGUF модель возвращает [batch, vocab_size] - только последний токен
            Qwen2Inner::Quantized(model) => model.forward(input, pos),
            // SafeTensors модель возвращает [batch, seq_len, vocab_size]
            // Извлекаем только последний токен для совместимости с генерацией
            Qwen2Inner::Full(model) => {
                let logits = model.forward(input, pos)?;
                let seq_len = logits.dim(1)?;
                // Берём логиты последнего токена: [batch, vocab_size]
                logits.narrow(1, seq_len - 1, 1)?.squeeze(1)
            }
        }
    }

    fn clear_kv_cache(&mut self) {
        match &mut self.inner {
            Qwen2Inner::Quantized(_model) => {
                // quantized_qwen2::ModelWeights doesn't expose clear_kv_cache() in candle-transformers
                // KV cache is internal to the model layers
                log::debug!("clear_kv_cache called on Qwen2 GGUF (no-op)");
            }
            Qwen2Inner::Full(model) => model.clear_kv_cache(),
        }
    }

    fn model_type(&self) -> &str {
        match self.optimization.weight_format() {
            WeightFormat::Gguf => "qwen2-gguf",
            WeightFormat::SafeTensors => {
                if self.optimization.uses_flash_attn() {
                    "qwen2-flash"
                } else {
                    "qwen2"
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
