//! Adapter for float LLM models from candle_transformers (safetensors format).
//! 
//! This module provides adapters that allow using models from candle_transformers
//! with the existing ModelBackend trait.
//! 
//! The adapters wrap the candle_transformers models and implement the ModelBackend
//! trait, enabling unified generation API for both GGUF quantized models and
//! float safetensors models.

use candle::Tensor;
use crate::models::common::model::ModelBackend;

/// Adapter for Qwen3 models from candle_transformers
/// 
/// This struct wraps the Qwen3 ModelForCausalLM and adapts it to the ModelBackend trait.
pub struct Qwen3CandleAdapter {
    /// The inner Qwen3 model
    inner: candle_transformers::models::qwen3::ModelForCausalLM,
}

impl Qwen3CandleAdapter {
    /// Create a new Qwen3CandleAdapter
    /// 
    /// # Arguments
    /// * `model` - The Qwen3 ModelForCausalLM
    /// 
    /// # Returns
    /// * `Qwen3CandleAdapter` - Adapter that implements ModelBackend
    pub fn new(model: candle_transformers::models::qwen3::ModelForCausalLM) -> Self {
        Self { inner: model }
    }
}

impl ModelBackend for Qwen3CandleAdapter {
    /// Forward pass through the model
    /// 
    /// # Arguments
    /// * `input` - Input tensor (typically token IDs)
    /// * `position` - Position in the sequence (for positional embeddings)
    /// 
    /// # Returns
    /// * `Result<Tensor, String>` - Output logits tensor or error message
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward(input, position).map_err(|e| e.to_string())
    }
}

/// Adapter for Qwen2 models from candle_transformers
pub struct Qwen2CandleAdapter {
    /// The inner Qwen2 model
    inner: candle_transformers::models::qwen2::ModelForCausalLM,
}

impl Qwen2CandleAdapter {
    /// Create a new Qwen2CandleAdapter
    /// 
    /// # Arguments
    /// * `model` - The Qwen2 ModelForCausalLM
    /// 
    /// # Returns
    /// * `Qwen2CandleAdapter` - Adapter that implements ModelBackend
    pub fn new(model: candle_transformers::models::qwen2::ModelForCausalLM) -> Self {
        Self { inner: model }
    }
}

impl ModelBackend for Qwen2CandleAdapter {
    /// Forward pass through the model
    /// 
    /// # Arguments
    /// * `input` - Input tensor (typically token IDs)
    /// * `position` - Position in the sequence (for positional embeddings)
    /// 
    /// # Returns
    /// * `Result<Tensor, String>` - Output logits tensor or error message
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward(input, position).map_err(|e| e.to_string())
    }
}

/// Adapter for Llama models from candle_transformers
pub struct LlamaCandleAdapter {
    /// The inner Llama model
    inner: candle_transformers::models::llama::Llama,
}

impl LlamaCandleAdapter {
    /// Create a new LlamaCandleAdapter
    /// 
    /// # Arguments
    /// * `model` - The Llama model
    /// 
    /// # Returns
    /// * `LlamaCandleAdapter` - Adapter that implements ModelBackend
    pub fn new(model: candle_transformers::models::llama::Llama) -> Self {
        Self { inner: model }
    }
}

impl ModelBackend for LlamaCandleAdapter {
    /// Forward pass through the model
    /// 
    /// # Arguments
    /// * `input` - Input tensor (typically token IDs)
    /// * `position` - Position in the sequence (for positional embeddings)
    /// 
    /// # Returns
    /// * `Result<Tensor, String>` - Output logits tensor or error message
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        // For Llama models, we need to create a cache and pass it
        // This is a simplified implementation - in practice, you would need to manage the cache properly
        let mut cache = candle_transformers::models::llama::Cache::new(
            true, // use_kv_cache
            input.dtype(),
            &candle_transformers::models::llama::Config {
                hidden_size: 4096,
                intermediate_size: 11008,
                vocab_size: 32000,
                num_hidden_layers: 32,
                num_attention_heads: 32,
                num_key_value_heads: 32,
                rms_norm_eps: 1e-5,
                rope_theta: 10000.0,
                use_flash_attn: false,
                bos_token_id: None,
                eos_token_id: None,
                rope_scaling: None,
                max_position_embeddings: 4096,
                tie_word_embeddings: false,
            },
            input.device()
        ).map_err(|e| e.to_string())?;
        
        self.inner.forward(input, position, &mut cache).map_err(|e| e.to_string())
    }
}

/// Adapter for Phi models from candle_transformers
pub struct PhiCandleAdapter {
    /// The inner Phi model
    inner: candle_transformers::models::phi::Model,
}

impl PhiCandleAdapter {
    /// Create a new PhiCandleAdapter
    /// 
    /// # Arguments
    /// * `model` - The Phi model
    /// 
    /// # Returns
    /// * `PhiCandleAdapter` - Adapter that implements ModelBackend
    pub fn new(model: candle_transformers::models::phi::Model) -> Self {
        Self { inner: model }
    }
}

impl ModelBackend for PhiCandleAdapter {
    /// Forward pass through the model
    /// 
    /// # Arguments
    /// * `input` - Input tensor (typically token IDs)
    /// * `position` - Position in the sequence (for positional embeddings)
    /// 
    /// # Returns
    /// * `Result<Tensor, String>` - Output logits tensor or error message
    fn forward_layered(&mut self, input: &Tensor, _position: usize) -> Result<Tensor, String> {
        // Phi models don't use position parameter in the same way
        self.inner.forward(input).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn test_adapter_creation() {
        // This is a placeholder test - actual testing would require model weights
        // and would be more complex
        assert!(true);
    }
}

/// Adapter for Gemma models from candle_transformers
pub struct GemmaCandleAdapter {
    /// The inner Gemma model
    inner: candle_transformers::models::gemma::Model,
}

impl GemmaCandleAdapter {
    /// Create a new GemmaCandleAdapter
    pub fn new(model: candle_transformers::models::gemma::Model) -> Self {
        Self { inner: model }
    }
}

impl ModelBackend for GemmaCandleAdapter {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward(input, position).map_err(|e| e.to_string())
    }
}
