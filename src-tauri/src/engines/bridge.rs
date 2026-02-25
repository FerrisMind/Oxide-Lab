use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct StreamEvent {
    pub thinking: String,
    pub content: String,
    pub done: bool,
}

pub fn parse_sse_chunk(chunk: &str) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    for line in chunk.lines() {
        let line = line.trim();
        if !line.starts_with("data: ") {
            continue;
        }

        let data = &line["data: ".len()..];
        if data == "[DONE]" {
            events.push(StreamEvent {
                thinking: String::new(),
                content: String::new(),
                done: true,
            });
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(data)
            && let Some(choices) = json.get("choices").and_then(|c| c.as_array())
            && let Some(choice) = choices.first()
        {
            // Extract delta
            if let Some(delta) = choice.get("delta") {
                let content = delta
                    .get("content")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                // DeepSeek style thinking? normally it's reasoning_content or within content with <think> tags.
                // <think> tags are handled by Frontend (listener.ts).
                // But some APIs (like Deepseek via OpenRouter) use "reasoning_content".
                // Standard OpenAI doesn't have thinking field.
                // We will just pass content.

                // Check for 'reasoning_content' field (DeepSeek R1 style in some APIs)
                let thinking = delta
                    .get("reasoning_content")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();

                events.push(StreamEvent {
                    thinking,
                    content,
                    done: false,
                });
            }
            if let Some(finish_reason) = choice.get("finish_reason").and_then(|s| s.as_str())
                && (finish_reason == "stop" || finish_reason == "length")
            {
                events.push(StreamEvent {
                    thinking: String::new(),
                    content: String::new(),
                    done: true,
                });
            }
        }
    }

    events
}
