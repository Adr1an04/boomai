use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub fn hello() -> String {
    "Hello from Boomai Core!".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    // later: metadata like workspace_id, tools, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse>;
}

// dummy provider for testing
pub struct DummyProvider;

#[async_trait]
impl ModelProvider for DummyProvider {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        let last = req
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "no message".to_string());

        Ok(ChatResponse {
            message: Message {
                role: Role::Assistant,
                content: format!("(dummy) I received: {}", last),
            },
        })
    }
}

// HTTP provider for real API calls
pub struct HttpProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    client: Client,
}

impl HttpProvider {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ModelProvider for HttpProvider {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        // OpenAI-style api body
        let body = serde_json::json!({
            "model": self.model,
            "messages": req.messages
        });

        // Prepare request
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        // Send and parse
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API request failed: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;

        // Extract content
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format or missing content"))?;

        Ok(ChatResponse {
            message: Message {
                role: Role::Assistant,
                content: content.to_string(),
            },
        })
    }
}
