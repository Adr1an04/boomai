use crate::provider::ModelProvider;
use crate::types::{ChatRequest, ChatResponse, Message, Role};
use async_trait::async_trait;

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
            status: crate::types::ExecutionStatus::Done,
            maker_context: None,
        })
    }
}

