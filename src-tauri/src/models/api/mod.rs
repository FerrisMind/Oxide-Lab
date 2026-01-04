//! Candle Unified API
//!
//! Единый интерфейс для запуска LLM моделей на базе Candle.
//!
//! # Модули
//! - `config` - Конфигурация генерации
//! - `error` - Типы ошибок
//! - `model` - Trait ModelBackend
//! - `optimization` - Конфигурация оптимизаций
//! - `pipeline` - TextGenerationPipeline
//! - `sampling` - Стратегии семплинга
//! - `tokenizer` - Обёртка над токенизатором
//! - `hub` - Загрузка с HuggingFace Hub

pub mod config;
pub mod error;
pub mod hub;
pub mod model;
pub mod optimization;
pub mod pipeline;
pub mod sampling;
pub mod tokenizer;

pub use config::GenerationConfig;
pub use error::{Error, Result};
pub use hub::HubDownloader;
pub use model::ModelBackend;
pub use optimization::{OptimizationConfig, SimdCapabilities, WeightFormat};
pub use pipeline::TextGenerationPipeline;
pub use sampling::{LogitsProcessorBuilder, SamplingStrategy};
pub use tokenizer::TokenizerWrapper;
