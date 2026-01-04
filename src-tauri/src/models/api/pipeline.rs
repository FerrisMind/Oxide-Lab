//! Основной пайплайн для генерации текста

use candle::{DType, Device, Tensor};
use candle_transformers::generation::LogitsProcessor;

use super::config::GenerationConfig;
use super::error::{Error, Result};
use super::model::ModelBackend;
use super::sampling::{LogitsProcessorBuilder, MinPFilter, apply_repeat_penalty};
use super::tokenizer::TokenizerWrapper;

/// Пайплайн для генерации текста
pub struct TextGenerationPipeline<M: ModelBackend> {
    model: M,
    tokenizer: TokenizerWrapper,
    config: GenerationConfig,
    device: Device,
    logits_processor: LogitsProcessor,
    minp_filter: MinPFilter,
}

impl<M: ModelBackend> TextGenerationPipeline<M> {
    /// Создаёт новый пайплайн
    pub fn new(
        model: M,
        tokenizer: TokenizerWrapper,
        config: GenerationConfig,
        device: Device,
    ) -> Self {
        let logits_processor = LogitsProcessorBuilder::new()
            .seed(config.seed)
            .temperature(config.temperature)
            .top_k(config.top_k.unwrap_or(40))
            .top_p(config.top_p.unwrap_or(0.9))
            .build();

        let minp_filter = MinPFilter::new(config.min_p, config.temperature);

        Self {
            model,
            tokenizer,
            config,
            device,
            logits_processor,
            minp_filter,
        }
    }

    /// Генерирует текст (блокирующий вызов)
    pub fn generate(&mut self, prompt: &str) -> Result<String> {
        let mut output = String::new();
        self.generate_stream(prompt, |chunk| {
            output.push_str(chunk);
        })?;
        Ok(output)
    }

    /// Генерирует текст с callback для каждого токена
    pub fn generate_stream<F>(&mut self, prompt: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&str),
    {
        // Очищаем KV-кэш
        self.model.clear_kv_cache();

        // Кодируем промпт
        let tokens = self.tokenizer.encode(prompt, true)?;
        let mut all_tokens = tokens.clone();

        // Получаем stop токены
        let stop_ids = self.tokenizer.stop_token_ids();
        if stop_ids.is_empty() {
            return Err(Error::Tokenizer("No EOS token found".into()));
        }

        // Prefill: обрабатываем весь промпт
        let input = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?;
        let logits = self.model.forward(&input, 0)?;
        let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;

        // Семплируем первый токен
        let mut next_token = self.sample_token(&logits, &all_tokens)?;
        all_tokens.push(next_token);

        // Декодируем и отправляем
        if let Ok(text) = self.tokenizer.decode(&[next_token], true)
            && !text.is_empty()
        {
            callback(&text);
        }

        // Генерация
        for idx in 0..self.config.max_new_tokens {
            // Проверяем stop токен
            if stop_ids.contains(&next_token) {
                break;
            }

            // Forward pass для одного токена
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, tokens.len() + idx)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;

            // Семплируем следующий токен
            next_token = self.sample_token(&logits, &all_tokens)?;
            all_tokens.push(next_token);

            // Декодируем и отправляем
            if let Ok(text) = self.tokenizer.decode(&[next_token], true)
                && !text.is_empty()
            {
                callback(&text);
            }
        }

        Ok(())
    }

    /// Семплирует токен из логитов
    fn sample_token(&mut self, logits: &Tensor, all_tokens: &[u32]) -> Result<u32> {
        // Применяем MinP фильтр
        let logits = self.minp_filter.apply(logits)?;

        // Применяем repeat penalty
        let logits = if self.config.repeat_penalty > 1.0 && !all_tokens.is_empty() {
            let start_at = all_tokens.len().saturating_sub(self.config.repeat_last_n);
            let penalty_tokens = &all_tokens[start_at..];
            if !penalty_tokens.is_empty() {
                apply_repeat_penalty(&logits, self.config.repeat_penalty, penalty_tokens)?
            } else {
                logits
            }
        } else {
            logits
        };

        // Семплируем
        let token = self.logits_processor.sample(&logits)?;
        Ok(token)
    }

    /// Возвращает ссылку на модель
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Возвращает мутабельную ссылку на модель
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Возвращает конфигурацию
    pub fn config(&self) -> &GenerationConfig {
        &self.config
    }

    /// Устанавливает новую конфигурацию
    pub fn set_config(&mut self, config: GenerationConfig) {
        self.logits_processor = LogitsProcessorBuilder::new()
            .seed(config.seed)
            .temperature(config.temperature)
            .top_k(config.top_k.unwrap_or(40))
            .top_p(config.top_p.unwrap_or(0.9))
            .build();

        self.minp_filter = MinPFilter::new(config.min_p, config.temperature);
        self.config = config;
    }
}

/// Метрики генерации
#[derive(Debug, Clone, Default)]
pub struct GenerationMetrics {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub prefill_time_ms: u64,
    pub generation_time_ms: u64,
    pub tokens_per_second: f64,
}

impl GenerationMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
