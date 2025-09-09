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

    pub fn from_candle_qwen3(m: candle_transformers::models::qwen3::ModelForCausalLM) -> Self {
        // Wrap the candle model to implement ModelBackend
        let wrapper = crate::models::common::candle_llm::Qwen3CandleAdapter::new(m);
        AnyModel { inner: Box::new(wrapper) }
    }
    
    /// Create AnyModel from any Qwen2 model from candle_transformers
    pub fn from_candle_qwen2(m: candle_transformers::models::qwen2::ModelForCausalLM) -> Self {
        let wrapper = crate::models::common::candle_llm::Qwen2CandleAdapter::new(m);
        AnyModel { inner: Box::new(wrapper) }
    }
    
    /// Create AnyModel from any Llama model from candle_transformers
    pub fn from_candle_llama(m: candle_transformers::models::llama::Llama) -> Self {
        let wrapper = crate::models::common::candle_llm::LlamaCandleAdapter::new(m);
        AnyModel { inner: Box::new(wrapper) }
    }
    
    /// Create AnyModel from any Phi model from candle_transformers
    pub fn from_candle_phi(m: candle_transformers::models::phi::Model) -> Self {
        let wrapper = crate::models::common::candle_llm::PhiCandleAdapter::new(m);
        AnyModel { inner: Box::new(wrapper) }
    }
}

impl ModelBackend for AnyModel {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner.forward_layered(input, position)
    }
}