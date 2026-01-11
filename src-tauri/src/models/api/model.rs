//! Trait ModelBackend для унификации всех моделей

use candle::Tensor;

/// Основной trait, который должны реализовывать все модели
pub trait ModelBackend: Send {
    /// Forward pass модели
    ///
    /// # Arguments
    /// * `input` - Входной тензор токенов [batch_size, seq_len]
    /// * `pos` - Позиция в KV-кэше (для инкрементальной генерации)
    ///
    /// # Returns
    /// Логиты [batch_size, seq_len, vocab_size]
    fn forward(&mut self, input: &Tensor, pos: usize) -> candle::Result<Tensor>;

    /// Forward pass с послойной обработкой (для GGUF моделей)
    fn forward_layered(&mut self, input: &Tensor, pos: usize) -> candle::Result<Tensor> {
        // По умолчанию делегируем в обычный forward
        self.forward(input, pos)
    }

    /// Очищает KV-кэш модели
    fn clear_kv_cache(&mut self);

    /// Возвращает тип модели (например "llama", "qwen3", "mistral")
    fn model_type(&self) -> &str;

    /// Возвращает размер словаря
    fn vocab_size(&self) -> usize;

    /// Возвращает максимальную длину контекста
    fn max_seq_len(&self) -> usize {
        4096 // Default
    }

    /// Проверяет, поддерживает ли модель Flash Attention
    fn supports_flash_attn(&self) -> bool {
        false
    }

    /// Возвращает количество параметров модели (приблизительно)
    fn num_parameters(&self) -> Option<usize> {
        None
    }

    /// Применяет конфигурацию из JSON
    fn apply_config(&mut self, _config: &serde_json::Value) -> Result<(), String> {
        // По умолчанию ничего не делаем
        Ok(())
    }

    // ============ Prefix Cache Support ============

    /// Возвращает текущую позицию KV-кэша (количество обработанных токенов)
    ///
    /// Используется для Prefix Cache: после prefill сохраняем эту позицию,
    /// чтобы при повторном запросе с тем же префиксом пропустить prefill.
    fn kv_cache_position(&self) -> usize {
        0 // По умолчанию: prefix cache не поддерживается
    }

    /// Устанавливает позицию KV-кэша для продолжения генерации
    ///
    /// Используется для Prefix Cache: при cache hit устанавливаем позицию
    /// в сохранённое значение и продолжаем генерацию с этой точки.
    ///
    /// # Returns
    /// `true` если модель поддерживает prefix cache, `false` иначе
    fn set_kv_cache_position(&mut self, _pos: usize) -> bool {
        false // По умолчанию: prefix cache не поддерживается
    }

    /// Проверяет, поддерживает ли модель prefix caching
    fn supports_prefix_cache(&self) -> bool {
        false // По умолчанию: не поддерживается
    }

    /// Возвращает эмбеддинги для входного тензора
    ///
    /// # Returns
    /// Тензор эмбеддингов (скрытые состояния последнего слоя)
    fn get_embeddings(&mut self, _input: &Tensor) -> candle::Result<Tensor> {
        candle::bail!("Embeddings not supported for this model type")
    }
}

/// Информация о загруженной модели
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Тип модели
    pub model_type: String,

    /// Размер словаря
    pub vocab_size: usize,

    /// Максимальная длина контекста
    pub max_seq_len: usize,

    /// Количество параметров
    pub num_parameters: Option<usize>,

    /// Квантизированная ли модель
    pub is_quantized: bool,

    /// Тип данных (F32, BF16, etc.)
    pub dtype: String,

    /// Устройство (CPU, CUDA)
    pub device: String,
}

// WeightFormat is defined in optimization module
pub use super::optimization::WeightFormat;

/// Trait для загрузки моделей
pub trait ModelLoader {
    /// Тип модели, которую загружает этот loader
    type Model: ModelBackend;

    /// Загружает модель из SafeTensors
    fn load_safetensors(
        &self,
        files: &[std::path::PathBuf],
        device: &candle::Device,
    ) -> super::error::Result<Self::Model>;

    /// Загружает модель из GGUF
    fn load_gguf(
        &self,
        path: &std::path::Path,
        device: &candle::Device,
    ) -> super::error::Result<Self::Model>;

    /// Возвращает поддерживаемые форматы
    fn supported_formats(&self) -> Vec<WeightFormat> {
        vec![WeightFormat::SafeTensors, WeightFormat::Gguf]
    }
}
