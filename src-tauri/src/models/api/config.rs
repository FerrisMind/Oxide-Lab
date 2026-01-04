//! Конфигурация генерации текста
//!
//! Этот модуль содержит структуры для настройки параметров генерации.

use serde::{Deserialize, Serialize};

/// Основная конфигурация для генерации текста
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Температура семплинга (0.0 = greedy, >0 = стохастический)
    pub temperature: f64,

    /// Nucleus sampling (top-p): вероятностный порог
    pub top_p: Option<f64>,

    /// Top-K sampling: количество топовых токенов
    pub top_k: Option<usize>,

    /// Min-P sampling: минимальный порог вероятности
    pub min_p: Option<f64>,

    /// Штраф за повторение токенов (1.0 = нет штрафа)
    pub repeat_penalty: f32,

    /// Размер окна для repeat_penalty
    pub repeat_last_n: usize,

    /// Максимальное количество новых токенов
    pub max_new_tokens: usize,

    /// Seed для RNG
    pub seed: u64,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: Some(0.9),
            top_k: Some(40),
            min_p: Some(0.05),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            max_new_tokens: 2048,
            seed: 42,
        }
    }
}

impl GenerationConfig {
    /// Создаёт конфиг для greedy decoding (детерминированный)
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_p: None,
            top_k: None,
            min_p: None,
            repeat_penalty: 1.0,
            repeat_last_n: 64,
            max_new_tokens: 2048,
            seed: 42,
        }
    }

    /// Создаёт конфиг для креативной генерации
    pub fn creative() -> Self {
        Self {
            temperature: 1.0,
            top_p: Some(0.95),
            top_k: Some(50),
            min_p: Some(0.02),
            repeat_penalty: 1.2,
            repeat_last_n: 128,
            max_new_tokens: 4096,
            seed: 42,
        }
    }

    /// Создаёт конфиг для точных ответов (код, факты)
    pub fn precise() -> Self {
        Self {
            temperature: 0.3,
            top_p: Some(0.8),
            top_k: Some(20),
            min_p: Some(0.1),
            repeat_penalty: 1.05,
            repeat_last_n: 32,
            max_new_tokens: 2048,
            seed: 42,
        }
    }

    /// Builder: устанавливает температуру
    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    /// Builder: устанавливает top_p
    pub fn with_top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Builder: устанавливает top_k
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = Some(k);
        self
    }

    /// Builder: устанавливает min_p
    pub fn with_min_p(mut self, p: f64) -> Self {
        self.min_p = Some(p);
        self
    }

    /// Builder: устанавливает repeat_penalty
    pub fn with_repeat_penalty(mut self, penalty: f32) -> Self {
        self.repeat_penalty = penalty;
        self
    }

    /// Builder: устанавливает max_new_tokens
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_new_tokens = max;
        self
    }

    /// Builder: устанавливает seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

/// Конфигурация загрузки модели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// HuggingFace model ID
    pub model_id: String,

    /// Revision (branch/tag)
    pub revision: String,

    /// Использовать квантизированную версию
    pub quantized: bool,

    /// Путь к локальным весам (опционально)
    pub weight_files: Option<Vec<String>>,

    /// Путь к локальному токенизатору (опционально)
    pub tokenizer_file: Option<String>,

    /// Использовать Flash Attention
    pub use_flash_attn: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            revision: "main".to_string(),
            quantized: false,
            weight_files: None,
            tokenizer_file: None,
            use_flash_attn: false,
        }
    }
}

impl ModelConfig {
    /// Создаёт конфиг для модели по ID
    pub fn from_model_id(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            ..Default::default()
        }
    }

    /// Builder: устанавливает revision
    pub fn with_revision(mut self, rev: impl Into<String>) -> Self {
        self.revision = rev.into();
        self
    }

    /// Builder: включает квантизацию
    pub fn quantized(mut self) -> Self {
        self.quantized = true;
        self
    }

    /// Builder: включает Flash Attention
    pub fn with_flash_attn(mut self) -> Self {
        self.use_flash_attn = true;
        self
    }
}
