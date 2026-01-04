//! Типы ошибок для Candle Unified API

use std::fmt;

/// Основной тип ошибки
#[derive(Debug)]
pub enum Error {
    /// Ошибка Candle (тензоры, устройства)
    Candle(String),

    /// Ошибка токенизатора
    Tokenizer(String),

    /// Ошибка загрузки модели
    ModelLoad(String),

    /// Ошибка конфигурации
    Config(String),

    /// Ошибка HuggingFace Hub
    Hub(String),

    /// Ошибка ввода/вывода
    Io(std::io::Error),

    /// Ошибка сериализации
    Serde(String),

    /// Модель не загружена
    ModelNotLoaded,

    /// Токенизатор не загружен
    TokenizerNotLoaded,

    /// Неподдерживаемый формат модели
    UnsupportedFormat(String),

    /// Генерация отменена
    Cancelled,

    /// Другая ошибка
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Candle(msg) => write!(f, "Candle error: {}", msg),
            Error::Tokenizer(msg) => write!(f, "Tokenizer error: {}", msg),
            Error::ModelLoad(msg) => write!(f, "Model load error: {}", msg),
            Error::Config(msg) => write!(f, "Config error: {}", msg),
            Error::Hub(msg) => write!(f, "Hub error: {}", msg),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Serde(msg) => write!(f, "Serialization error: {}", msg),
            Error::ModelNotLoaded => write!(f, "Model is not loaded"),
            Error::TokenizerNotLoaded => write!(f, "Tokenizer is not loaded"),
            Error::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
            Error::Cancelled => write!(f, "Generation cancelled"),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<candle::Error> for Error {
    fn from(e: candle::Error) -> Self {
        Error::Candle(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e.to_string())
    }
}

impl From<hf_hub::api::sync::ApiError> for Error {
    fn from(e: hf_hub::api::sync::ApiError) -> Self {
        Error::Hub(e.to_string())
    }
}

impl From<tokenizers::Error> for Error {
    fn from(e: tokenizers::Error) -> Self {
        Error::Tokenizer(e.to_string())
    }
}

/// Алиас для Result с нашим типом ошибки
pub type Result<T> = std::result::Result<T, Error>;
