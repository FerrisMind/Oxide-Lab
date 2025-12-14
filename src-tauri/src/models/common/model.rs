use candle::Tensor;
use serde_json;

pub trait ModelBackend: Send {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String>;

    /// Очистка KV-кэша модели между запросами (по умолчанию no-op).
    fn clear_kv_cache(&mut self) {}

    /// Optionally apply runtime configuration parsed from GGUF `config.json` metadata.
    /// Default implementation is a no-op. Builders/backends that support applying
    /// configuration should override this method.
    fn apply_config(&mut self, _config: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

/// AnyModel is a wrapper over concrete model implementations (qwen3 / candle models)
pub struct AnyModel {
    inner: Box<dyn ModelBackend + Send>,
}

impl AnyModel {
    pub fn from_qwen3(m: crate::models::qwen3::model::ModelWeights) -> Self {
        // The Qwen3 ModelWeights should implement ModelBackend; we box it
        AnyModel { inner: Box::new(m) }
    }

    // Адаптеры отключены
}

impl ModelBackend for AnyModel {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward_layered(input, position)
    }

    fn clear_kv_cache(&mut self) {
        self.inner.clear_kv_cache();
    }
}
