//! Flash Attention helpers
//!
//! Унифицированный helper для Flash Attention который можно использовать в любых моделях.
//! Автоматически выбирает между Flash Attention (если доступно) и стандартным attention.

use candle::{Result, Tensor};

/// Применяет scaled dot-product attention с автоматическим выбором реализации
///
/// # Arguments
/// * `q` - Query тензор [batch, num_heads, seq_len, head_dim]
/// * `k` - Key тензор [batch, num_heads, seq_len, head_dim]
/// * `v` - Value тензор [batch, num_heads, seq_len, head_dim]
/// * `scale` - Масштабирующий коэффициент (обычно 1/sqrt(head_dim))
/// * `causal` - Использовать causal masking (для autoregressive генерации)
///
/// # Returns
/// Output тензор [batch, num_heads, seq_len, head_dim]
#[allow(unused_variables)]
pub fn scaled_dot_product_attention(
    q: &Tensor,
    k: &Tensor,
    v: &Tensor,
    scale: f32,
    causal: bool,
) -> Result<Tensor> {
    #[cfg(feature = "flash-attn")]
    {
        // Flash Attention требует формат [batch, seq_len, num_heads, head_dim]
        // У нас [batch, num_heads, seq_len, head_dim], нужно transpose
        let q_fa = q.transpose(1, 2)?.contiguous()?;
        let k_fa = k.transpose(1, 2)?.contiguous()?;
        let v_fa = v.transpose(1, 2)?.contiguous()?;

        // Вызываем Flash Attention
        log::debug!("🔥 Using Flash Attention: shape={:?}", q_fa.shape());
        let output = candle_flash_attn::flash_attn(&q_fa, &k_fa, &v_fa, scale, causal)?;

        // Возвращаем обратно в формат [batch, num_heads, seq_len, head_dim]
        output.transpose(1, 2)
    }
    #[cfg(not(feature = "flash-attn"))]
    {
        // Fallback к стандартному attention
        log::debug!("Using standard attention");
        standard_attention(q, k, v, scale, causal)
    }
}

/// Стандартная реализация scaled dot-product attention (fallback)
///
/// Используется когда Flash Attention недоступно
#[allow(dead_code)]
fn standard_attention(
    q: &Tensor,
    k: &Tensor,
    v: &Tensor,
    scale: f32,
    causal: bool,
) -> Result<Tensor> {
    // q, k, v shape: [batch, num_heads, seq_len, head_dim]

    // 1. Q @ K^T
    let scores = q.matmul(&k.transpose(2, 3)?)?;

    // 2. Scale
    let scores = (scores * scale as f64)?;

    // 3. Causal masking (если нужно)
    let scores = if causal {
        let (_, _, seq_len, _) = scores.dims4()?;
        let mask = create_causal_mask(seq_len, scores.device(), scores.dtype())?;
        scores.broadcast_add(&mask)?
    } else {
        scores
    };

    // 4. Softmax
    let probs = candle_nn::ops::softmax_last_dim(&scores)?;

    // 5. @ V
    probs.matmul(v)
}

/// Создаёт causal mask для autoregressive генерации
fn create_causal_mask(
    seq_len: usize,
    device: &candle::Device,
    dtype: candle::DType,
) -> Result<Tensor> {
    let mask: Vec<_> = (0..seq_len)
        .flat_map(|i| (0..seq_len).map(move |j| if j > i { f32::NEG_INFINITY } else { 0f32 }))
        .collect();

    Tensor::from_vec(mask, (seq_len, seq_len), device)?
        .to_dtype(dtype)?
        .unsqueeze(0)?
        .unsqueeze(0) // [1, 1, seq_len, seq_len] для broadcasting
}

/// Проверяет, доступен ли Flash Attention для текущей конфигурации
pub fn is_flash_attention_available() -> bool {
    #[cfg(feature = "flash-attn")]
    {
        candle::utils::cuda_is_available()
    }
    #[cfg(not(feature = "flash-attn"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle::{DType, Device};

    #[test]
    fn test_standard_attention() {
        let device = Device::Cpu;
        let dtype = DType::F32;

        // [batch=1, heads=2, seq=4, dim=8]
        let q = Tensor::randn(0f32, 1f32, (1, 2, 4, 8), &device)
            .unwrap()
            .to_dtype(dtype)
            .unwrap();
        let k = Tensor::randn(0f32, 1f32, (1, 2, 4, 8), &device)
            .unwrap()
            .to_dtype(dtype)
            .unwrap();
        let v = Tensor::randn(0f32, 1f32, (1, 2, 4, 8), &device)
            .unwrap()
            .to_dtype(dtype)
            .unwrap();

        let output = scaled_dot_product_attention(&q, &k, &v, 1.0 / (8f32).sqrt(), true).unwrap();

        assert_eq!(output.dims(), &[1, 2, 4, 8]);
    }
}
