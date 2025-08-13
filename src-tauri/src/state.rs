use std::fs::File;
use std::sync::{Arc, Mutex};
use candle::Device;
use tokenizers::Tokenizer;
use crate::model::qwen3_offload::ModelWeights as Qwen3Gguf;

/// Глобальное состояние модели и токенизатора
pub(crate) struct ModelState {
    pub(crate) gguf_model: Option<Qwen3Gguf>,
    pub(crate) gguf_file: Option<File>,
    pub(crate) tokenizer: Option<Tokenizer>,
    pub(crate) device: Device,
    pub(crate) context_length: usize,
    pub(crate) model_path: Option<String>,
    pub(crate) tokenizer_path: Option<String>,
    pub(crate) fallback_to_cpu_on_oom: bool,
    pub(crate) n_gpu_layers: usize,
}

impl ModelState {
    pub(crate) fn new(device: Device) -> Self {
        Self {
            gguf_model: None,
            gguf_file: None,
            tokenizer: None,
            device,
            context_length: 4096,
            model_path: None,
            tokenizer_path: None,
            fallback_to_cpu_on_oom: true,
            n_gpu_layers: 0,
        }
    }
}

pub(crate) type SharedState = Arc<Mutex<ModelState>>;


