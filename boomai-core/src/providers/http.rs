use crate::provider::ModelProvider;
use crate::types::{ChatRequest, ChatResponse, Message, Role};
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
        // mac os build without proxy
        let client = Client::builder()
            .no_proxy() 
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url,
            api_key,
            model,
            client,
        }
    }
}

#[async_trait]
impl ModelProvider for HttpProvider {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        // open ai style http provider
        let body = serde_json::json!({
            "model": self.model,
            "messages": req.messages
        });

        // prepare request
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            if !key.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", key));
            }
        }

        // parse
        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API request failed: {} - {}", error_text, "Check your API key or local server status"));
        }

        let data: serde_json::Value = response.json().await?;

        // extract content
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format or missing content"))?;

        Ok(ChatResponse {
            message: Message {
                role: Role::Assistant,
                content: content.to_string(),
            },
            status: crate::types::ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
