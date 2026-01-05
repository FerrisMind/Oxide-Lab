//! Llama model backend
//!
//! Обёртка над candle_transformers для интеграции Llama с нашим API.
//! Поддерживает семейство Llama: Llama 1/2/3.x, Mistral, DeepSeek-R1, TinyLlama, Yi, SmolLM2.
//!
//! # Структура
//! - `mod.rs` - общий LlamaBackend и ModelBackend реализация
//! - `gguf.rs` - загрузка из GGUF формата
//! - `safetensors.rs` - загрузка из SafeTensors формата

mod gguf;
mod safetensors;

use candle::{Device, Tensor};
use candle_transformers::models::llama::{Cache, Llama};
use candle_transformers::models::quantized_llama::ModelWeights;

use crate::models::ModelBackend;
use crate::models::api::optimization::{OptimizationConfig, WeightFormat};

/// Внутреннее представление модели
enum LlamaInner {
    /// Квантизированная модель из GGUF
    Quantized(ModelWeights),
    /// Полная модель из SafeTensors (с опциональным Flash Attention)
    Full { model: Llama, cache: Cache },
}

/// Llama-подобный бекенд
///
/// Поддерживает как квантизированные (GGUF) так и полные (SafeTensors) модели.
pub struct LlamaBackend {
    inner: LlamaInner,
    device: Device,
    vocab_size: usize,
    max_seq_len: usize,
    optimization: OptimizationConfig,
}

impl LlamaBackend {
    /// Создаёт квантизированный бекенд (используется из gguf.rs)
    pub(crate) fn new_quantized(
        model: ModelWeights,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
    ) -> Self {
        Self {
            inner: LlamaInner::Quantized(model),
            device,
            vocab_size,
            max_seq_len,
            optimization: OptimizationConfig::for_gguf(),
        }
    }

    /// Создаёт полный бекенд (используется из safetensors.rs)
    pub(crate) fn new_full(
        model: Llama,
        cache: Cache,
        device: Device,
        vocab_size: usize,
        max_seq_len: usize,
        optimization: OptimizationConfig,
    ) -> Self {
        Self {
            inner: LlamaInner::Full { model, cache },
            device,
            vocab_size,
            max_seq_len,
            optimization,
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn is_quantized(&self) -> bool {
        matches!(self.inner, LlamaInner::Quantized(_))
    }

    pub fn optimization(&self) -> &OptimizationConfig {
        &self.optimization
    }
}

impl ModelBackend for LlamaBackend {
    fn forward(&mut self, input: &Tensor, pos: usize) -> candle::Result<Tensor> {
        match &mut self.inner {
            LlamaInner::Quantized(model) => model.forward(input, pos),
            LlamaInner::Full { model, cache } => {
                let logits = model.forward(input, pos, cache)?;
                let seq_len = logits.dim(1)?;
                // Берём логиты последнего токена: [batch, vocab_size]
                logits.narrow(1, seq_len - 1, 1)?.squeeze(1)
            }
        }
    }

    fn clear_kv_cache(&mut self) {
        match &mut self.inner {
            LlamaInner::Quantized(_) => {
                // quantized_llama::ModelWeights автоматически сбрасывает kv_cache
                // при index_pos == 0 в forward()
            }
            LlamaInner::Full { .. } => {
                // Для full модели сбрасываем cache
                // Cache имеет внутреннюю структуру, которую можно пересоздать или сбросить,
                // но в candle Llama Cache обычно просто переиспользуется.
                // При новом запросе с index_pos=0 он перезаписывается?
                // TODO: Проверить необходимость явного сброса.
                // Пока оставим пустым, так как context управляется снаружи.
            }
        }
    }

    fn model_type(&self) -> &str {
        match self.optimization.weight_format() {
            WeightFormat::Gguf => "llama-gguf",
            WeightFormat::SafeTensors => {
                if self.optimization.uses_flash_attn() {
                    "llama-flash"
                } else {
                    "llama"
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
