use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DevicePreference {
    Auto,
    Cpu,
    Cuda { index: usize },
    Metal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "lowercase")]
pub enum LoadRequest {
    Gguf {
        model_path: String,
        tokenizer_path: Option<String>,
        context_length: usize,
        device: Option<DevicePreference>,
    },
    #[serde(rename = "hub_gguf")]
    HubGguf {
        /// Репозиторий на HF Hub: например, "Qwen/Qwen2.5-3B-Instruct-GGUF"
        repo_id: String,
        /// Ревизия/ветка/коммит (опционально), по умолчанию — main
        revision: Option<String>,
        /// Имя файла .gguf в репозитории. Обязателен для однозначной загрузки.
        filename: String,
        context_length: usize,
        device: Option<DevicePreference>,
    },
    #[serde(rename = "hub_safetensors")]
    HubSafetensors {
        /// Репозиторий на HF Hub: например, "meta-llama/Meta-Llama-3-8B-Instruct"
        repo_id: String,
        /// Ревизия/ветка/коммит (опционально), по умолчанию — main
        revision: Option<String>,
        /// Контекст (KV-cache length)
        context_length: usize,
        /// Предпочтительное устройство
        device: Option<DevicePreference>,
    },
    #[serde(rename = "local_safetensors")]
    LocalSafetensors {
        /// Путь к локальной директории с моделью safetensors
        model_path: String,
        /// Контекст (KV-cache length)
        context_length: usize,
        /// Предпочтительное устройство
        device: Option<DevicePreference>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub min_p: Option<f64>,
    pub repeat_penalty: Option<f32>,
    pub repeat_last_n: usize,
    #[serde(default)]
    pub use_custom_params: bool,
    #[serde(default)]
    pub seed: Option<u64>,
}


