use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Default)]
pub struct EngineClient {
    client: Client,
}

impl EngineClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(600)) // 10 min timeout for long generations
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn stream_chat_completion(
        &self,
        base_url: &str,
        req: Value,
    ) -> Result<impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, String>
    {
        // Construct full URL. Assume base_url is "http://127.0.0.1:PORT"
        // Standard OpenAI endpoint: /v1/chat/completions
        // Some engines might differ, but we target standard compliance first.
        let url = format!("{}/v1/chat/completions", base_url);

        let res = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("API Request Failed: {} - {}", url, error_text));
        }

        Ok(res.bytes_stream())
    }
}
