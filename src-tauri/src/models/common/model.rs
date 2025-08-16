use candle::Tensor;

pub trait ModelBackend: Send {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String>;
}

/// AnyModel is a wrapper over concrete model implementations (qwen3 / candle models)
pub struct AnyModel {
    inner: Box<dyn ModelBackend + Send>,
}

impl AnyModel {
    pub fn from_qwen3(m: crate::models::qwen3::ModelWeights) -> Self {
        // The Qwen3 ModelWeights should implement ModelBackend; we box it
        AnyModel { inner: Box::new(m) }
    }

    pub fn from_candle_qwen3(_m: candle_transformers::models::qwen3::ModelForCausalLM) -> Self {
        // For now, we do not have a direct candle wrapper; implement later if needed
        unimplemented!("from_candle_qwen3 is not implemented yet")
    }
}

impl ModelBackend for AnyModel {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward_layered(input, position)
    }
}


