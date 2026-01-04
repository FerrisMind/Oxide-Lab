//! Backend optimization configuration
//!
//! Конфигурация оптимизаций для бекендов моделей.
//!
//! # Платформо-независимые оптимизации (всегда включены):
//! - KV Cache (кэширование ключей/значений)
//! - RoPE (Rotary Position Embeddings)
//! - RMSNorm (нормализация)
//! - Causal masking
//!
//! # Платформо-зависимые оптимизации:
//! - Flash Attention - автоматически включается для SafeTensors на CUDA (bf16/f16)
//!   Требует: CUDA + feature "flash-attn" + SafeTensors формат

use candle::DType;
use serde::{Deserialize, Serialize};

/// Формат весов модели
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightFormat {
    /// GGUF квантизированный формат
    Gguf,
    /// SafeTensors полная точность
    SafeTensors,
}

/// Конфигурация оптимизаций бекенда
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Flash Attention - автоматически включён для SafeTensors на CUDA
    /// Только для чтения - устанавливается автоматически
    use_flash_attn: bool,

    /// Формат весов модели
    weight_format: WeightFormat,

    /// Тип данных (для SafeTensors)
    dtype: Option<String>,

    /// Использовать KV-кэш (всегда включён)
    use_kv_cache: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            use_flash_attn: false,
            weight_format: WeightFormat::Gguf,
            dtype: None,
            use_kv_cache: true,
        }
    }
}

impl OptimizationConfig {
    /// Создаёт конфигурацию для GGUF модели
    /// Flash Attention НЕ поддерживается для GGUF
    pub fn for_gguf() -> Self {
        Self {
            use_flash_attn: false, // GGUF не поддерживает flash-attn
            weight_format: WeightFormat::Gguf,
            dtype: None,
            use_kv_cache: true,
        }
    }

    /// Создаёт конфигурацию для SafeTensors модели
    /// Flash Attention автоматически включается если:
    /// - feature "flash-attn" скомпилирован
    /// - CUDA доступен
    /// - dtype = bf16 или f16
    pub fn for_safetensors(dtype: DType) -> Self {
        let flash_available = Self::is_flash_attn_available(dtype);

        Self {
            use_flash_attn: flash_available,
            weight_format: WeightFormat::SafeTensors,
            dtype: Some(dtype_to_string(dtype)),
            use_kv_cache: true,
        }
    }

    /// Проверяет, должен ли быть включён Flash Attention
    /// Flash Attention доступен только для SafeTensors + CUDA + bf16/f16
    fn is_flash_attn_available(dtype: DType) -> bool {
        // Flash Attention поддерживает только bf16 и f16
        let dtype_supported = matches!(dtype, DType::BF16 | DType::F16);

        #[cfg(feature = "flash-attn")]
        {
            dtype_supported && candle::utils::cuda_is_available()
        }
        #[cfg(not(feature = "flash-attn"))]
        {
            let _ = dtype_supported;
            false
        }
    }

    /// Возвращает true если Flash Attention включён
    pub fn uses_flash_attn(&self) -> bool {
        self.use_flash_attn
    }

    /// Возвращает формат весов
    pub fn weight_format(&self) -> WeightFormat {
        self.weight_format
    }

    /// Возвращает информацию о SIMD возможностях
    pub fn simd_info() -> SimdCapabilities {
        SimdCapabilities {
            avx: candle::utils::with_avx(),
            neon: candle::utils::with_neon(),
            simd128: candle::utils::with_simd128(),
            f16c: candle::utils::with_f16c(),
        }
    }

    /// Возвращает человекочитаемое описание оптимизаций
    pub fn description(&self) -> String {
        let mut opts = vec!["KV Cache"];

        if self.use_flash_attn {
            opts.push("Flash Attention v2");
        }

        let format = match self.weight_format {
            WeightFormat::Gguf => "GGUF (quantized)",
            WeightFormat::SafeTensors => "SafeTensors",
        };

        format!("{} [{}]", opts.join(", "), format)
    }
}

/// Информация о доступных SIMD возможностях
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimdCapabilities {
    pub avx: bool,
    pub neon: bool,
    pub simd128: bool,
    pub f16c: bool,
}

impl SimdCapabilities {
    /// Возвращает строку с описанием SIMD возможностей
    pub fn description(&self) -> String {
        let mut caps = Vec::new();
        if self.avx {
            caps.push("AVX");
        }
        if self.neon {
            caps.push("NEON");
        }
        if self.simd128 {
            caps.push("SIMD128");
        }
        if self.f16c {
            caps.push("F16C");
        }
        if caps.is_empty() {
            "None".to_string()
        } else {
            caps.join(", ")
        }
    }
}

/// Конвертирует DType в строку
fn dtype_to_string(dtype: DType) -> String {
    match dtype {
        DType::F32 => "f32",
        DType::F16 => "f16",
        DType::BF16 => "bf16",
        DType::F64 => "f64",
        DType::U8 => "u8",
        DType::U32 => "u32",
        DType::I64 => "i64",
        DType::I32 => "i32",
        DType::F8E4M3 => "f8e4m3",
        _ => "unknown",
    }
    .to_string()
}
