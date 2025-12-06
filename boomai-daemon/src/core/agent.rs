use crate::core::types::{ChatRequest, ChatResponse};
use async_trait::async_trait;

#[derive(Debug, Clone, Default)]
pub struct AgentContext;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn handle_chat(&self, req: ChatRequest, ctx: AgentContext) -> anyhow::Result<ChatResponse>;
}

