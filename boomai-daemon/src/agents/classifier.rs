use crate::core::{
    Agent, AgentContext, ChatRequest, ChatResponse, ExecutionStatus, Message, ModelProvider, Role,
};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct ClassifierAgent {
    model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
}

impl ClassifierAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for ClassifierAgent {
    async fn handle_chat(
        &self,
        req: ChatRequest,
        _ctx: AgentContext,
    ) -> anyhow::Result<ChatResponse> {
        let mut messages = req.messages.clone();

        messages.insert(0, Message {
            role: Role::System,
            content: "Classify the request as SIMPLE, COMPLEX, or TOOL. Output the category name ONLY. Do not write a sentence.".to_string(),
        });

        let classify_req = ChatRequest { messages };

        // Get the current configured provider
        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                return Err(anyhow::anyhow!("Failed to acquire read lock on model provider"));
            }
        };

        // We use a fresh context or pass through? For now simple call.
        // We expect a simple string response.
        let response = provider.chat(classify_req).await?;

        // We just return the classification in the content
        Ok(ChatResponse {
            message: response.message,
            status: ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
