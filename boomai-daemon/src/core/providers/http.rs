use crate::core::provider::ModelProvider;
use crate::core::types::{ChatRequest, ChatResponse, ExecutionStatus, Message, Role};
use async_trait::async_trait;
use reqwest::Client;

pub struct HttpProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    client: Client,
}

impl HttpProvider {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        let client = Client::builder().no_proxy().build().unwrap_or_else(|_| Client::new());

        Self { base_url, api_key, model, client }
    }
}

#[async_trait]
impl ModelProvider for HttpProvider {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        let start = std::time::Instant::now();
        let body = serde_json::json!({
            "model": self.model,
            "messages": req.messages
        });

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            if !key.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", key));
            }
        }

        let response = request.send().await?;
        let duration_ms = start.elapsed().as_millis();

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "API request failed: {} - {}",
                error_text,
                "Check your API key or local server status"
            ));
        }

        let raw_text = response.text().await?;
        let data: serde_json::Value = serde_json::from_str(&raw_text)?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format or missing content"))?;

        tracing::debug!(
            target: "llm_trace",
            model = %self.model,
            duration_ms = duration_ms,
            prompt_messages = req.messages.len(),
            raw_response = %raw_text,
            "llm_call"
        );

        Ok(ChatResponse {
            message: Message { role: Role::Assistant, content: content.to_string() },
            status: ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
