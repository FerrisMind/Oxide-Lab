use std::fs::File;
use std::sync::{Arc, Mutex};
use candle::Device;
use tokenizers::Tokenizer;

/// Универсальное состояние для любой модели
pub(crate) struct ModelState<M> {
    pub(crate) gguf_model: Option<M>,
    pub(crate) gguf_file: Option<File>,
    pub(crate) tokenizer: Option<Tokenizer>,
    pub(crate) device: Device,
    pub(crate) context_length: usize,
    pub(crate) model_path: Option<String>,
    pub(crate) tokenizer_path: Option<String>,
    pub(crate) model_config_json: Option<String>,
    pub(crate) chat_template: Option<String>,
    // HF Hub (safetensors) связанные артефакты
    pub(crate) hub_repo_id: Option<String>,
    pub(crate) hub_revision: Option<String>,
    pub(crate) safetensors_files: Option<Vec<String>>,
}

impl<M> ModelState<M> {
    pub(crate) fn new(device: Device) -> Self {
        Self {
            gguf_model: None,
            gguf_file: None,
            tokenizer: None,
            device,
            context_length: 4096,
            model_path: None,
            tokenizer_path: None,
            model_config_json: None,
            chat_template: None,
            hub_repo_id: None,
            hub_revision: None,
            safetensors_files: None,
        }
    }
}

pub(crate) type SharedState<M> = Arc<Mutex<ModelState<M>>>;


