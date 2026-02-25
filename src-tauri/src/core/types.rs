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
