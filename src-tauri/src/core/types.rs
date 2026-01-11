use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Structured message for streaming with thinking support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamMessage {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub thinking: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
}

impl StreamMessage {
    pub fn is_empty(&self) -> bool {
        self.thinking.is_empty() && self.content.is_empty()
    }
}

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
    #[serde(default)]
    pub messages: Option<Vec<ChatMessage>>,
    // Вложения временно отключены
    #[serde(default)]
    pub attachments: Option<Vec<Attachment>>, // deprecated
    #[serde(default)]
    pub max_new_tokens: Option<usize>,
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
    #[serde(default)]
    pub split_prompt: Option<bool>,
    #[serde(default)]
    pub verbose_prompt: Option<bool>,
    #[serde(default)]
    pub tracing: Option<bool>,
    /// Edit/Regenerate: if set, truncate history at this message index before generating.
    /// Used for regenerating from a specific point or editing a message.
    #[serde(default)]
    pub edit_index: Option<usize>,
    /// Output format constraint for grammar sampling (json, json_schema)
    #[serde(default)]
    pub format: Option<crate::generate::grammar::OutputFormat>,
    /// Tools available for function calling. If provided, enables tool call parsing.
    #[serde(default)]
    pub tools: Option<Vec<crate::generate::tool_call_parser::Tool>>,
    /// Stop sequences - generation stops when any of these are encountered
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,
    /// Tool choice: auto, none, required, or specific function
    #[serde(default)]
    pub tool_choice: Option<ToolChoice>,
}

/// Tool choice options for controlling function calling behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// "auto", "none", "required"
    Mode(String),
    /// {"type": "function", "function": {"name": "..."}}
    Function {
        r#type: String,
        function: ToolChoiceFunction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    pub name: String,
}

// Структура Attachment оставлена на будущее (компат), но можно удалить полностью при необходимости.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub kind: Option<String>,
    pub mime: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub bytes_b64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SttModelSource {
    Bundled,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SttSettings {
    pub source: SttModelSource,
    pub custom_dir: Option<String>,
}

impl Default for SttSettings {
    fn default() -> Self {
        Self {
            source: SttModelSource::Bundled,
            custom_dir: None,
        }
    }
}
