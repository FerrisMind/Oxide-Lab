//! Стратегии семплинга токенов

use candle::{DType, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};

/// Стратегия семплинга
#[derive(Debug, Clone)]
pub enum SamplingStrategy {
    /// Greedy: выбор токена с максимальной вероятностью
    Greedy,

    /// Все токены с заданной температурой
    Temperature(f64),

    /// Top-K sampling
    TopK { k: usize, temperature: f64 },

    /// Top-P (nucleus) sampling
    TopP { p: f64, temperature: f64 },

    /// Top-K затем Top-P
    TopKThenTopP { k: usize, p: f64, temperature: f64 },

    /// Min-P sampling (требует отдельной реализации)
    MinP { min_p: f64, temperature: f64 },
}

impl SamplingStrategy {
    /// Конвертирует в Sampling из candle_transformers
    pub fn to_sampling(&self) -> Sampling {
        match self {
            SamplingStrategy::Greedy => Sampling::ArgMax,
            SamplingStrategy::Temperature(t) => Sampling::All { temperature: *t },
            SamplingStrategy::TopK { k, temperature } => Sampling::TopK {
                k: *k,
                temperature: *temperature,
            },
            SamplingStrategy::TopP { p, temperature } => Sampling::TopP {
                p: *p,
                temperature: *temperature,
            },
            SamplingStrategy::TopKThenTopP { k, p, temperature } => Sampling::TopKThenTopP {
                k: *k,
                p: *p,
                temperature: *temperature,
            },
            // MinP не поддерживается напрямую, используем All
            SamplingStrategy::MinP { temperature, .. } => Sampling::All {
                temperature: *temperature,
            },
        }
    }

    /// Создаёт стратегию из параметров
    pub fn from_params(temperature: f64, top_k: Option<usize>, top_p: Option<f64>) -> Self {
        if temperature <= 0.0 {
            SamplingStrategy::Greedy
        } else {
            match (top_k, top_p) {
                (None, None) => SamplingStrategy::Temperature(temperature),
                (Some(k), None) => SamplingStrategy::TopK { k, temperature },
                (None, Some(p)) => SamplingStrategy::TopP { p, temperature },
                (Some(k), Some(p)) => SamplingStrategy::TopKThenTopP { k, p, temperature },
            }
        }
    }
}

/// Builder для создания LogitsProcessor
pub struct LogitsProcessorBuilder {
    seed: u64,
    temperature: f64,
    top_k: Option<usize>,
    top_p: Option<f64>,
}

impl Default for LogitsProcessorBuilder {
    fn default() -> Self {
        Self {
            seed: 42,
            temperature: 0.7,
            top_k: None,
            top_p: None,
        }
    }
}

impl LogitsProcessorBuilder {
    /// Создаёт новый builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Устанавливает seed
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Устанавливает температуру
    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    /// Устанавливает top_k
    pub fn top_k(mut self, k: usize) -> Self {
        self.top_k = Some(k);
        self
    }

    /// Устанавливает top_p
    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Создаёт LogitsProcessor
    pub fn build(self) -> LogitsProcessor {
        let strategy = SamplingStrategy::from_params(self.temperature, self.top_k, self.top_p);
        LogitsProcessor::from_sampling(self.seed, strategy.to_sampling())
    }
}

/// Min-P фильтр для логитов
pub struct MinPFilter {
    min_p: Option<f32>,
    temperature: f32,
}

impl MinPFilter {
    /// Создаёт новый фильтр
    pub fn new(min_p: Option<f64>, temperature: f64) -> Self {
        Self {
            min_p: min_p.and_then(|v| {
                if (0.0..=1.0).contains(&v) {
                    Some(v as f32)
                } else {
                    None
                }
            }),
            temperature: temperature as f32,
        }
    }

    /// Применяет фильтр к логитам
    ///
    /// Возвращает логиты с отфильтрованными токенами (установленными в -inf)
    pub fn apply(&self, logits: &Tensor) -> super::error::Result<Tensor> {
        let min_p = match self.min_p {
            Some(v) => v,
            None => return logits.to_dtype(DType::F32).map_err(|e| e.into()),
        };

        if self.temperature <= 0.0 {
            return logits.to_dtype(DType::F32).map_err(|e| e.into());
        }

        // Находим максимум
        let max_val = logits.max(0)?;

        // Вычисляем порог
        let threshold_scalar = self.temperature * min_p.ln();
        let threshold = (&max_val + threshold_scalar as f64)?;

        // Broadcast threshold
        let threshold_broadcasted = threshold.broadcast_as(logits.shape())?;

        // Создаём маску
        let mask = logits.ge(&threshold_broadcasted)?;

        // Создаём -inf тензор с правильным dtype
        let neg_inf = Tensor::new(f32::NEG_INFINITY, logits.device())?
            .to_dtype(logits.dtype())?
            .broadcast_as(logits.shape())?;

        // Применяем маску
        let result = mask.where_cond(logits, &neg_inf)?;

        // Конвертируем в F32 для LogitsProcessor
        Ok(result.to_dtype(DType::F32)?)
    }
}

/// Применяет repeat penalty к логитам
pub fn apply_repeat_penalty(
    logits: &Tensor,
    penalty: f32,
    tokens: &[u32],
) -> super::error::Result<Tensor> {
    if tokens.is_empty() || (penalty - 1.0).abs() < f32::EPSILON {
        return Ok(logits.clone());
    }

    candle_transformers::utils::apply_repeat_penalty(logits, penalty, tokens).map_err(|e| e.into())
}
