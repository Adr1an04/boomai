use crate::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse>;
}

