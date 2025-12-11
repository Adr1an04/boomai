use crate::core::{Agent, AgentContext, ChatRequest, ChatResponse, Message, ModelProvider, Role};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RouterAgent {
    model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
}

impl RouterAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for RouterAgent {
    async fn handle_chat(
        &self,
        req: ChatRequest,
        _ctx: AgentContext,
    ) -> anyhow::Result<ChatResponse> {
        // Router logic: Decide if tools are needed or if it's a direct answer.
        // For MVP, we'll just simulate this decision or pass it to the model with a specific prompt.

        let mut messages = req.messages.clone();
        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Router Agent. Analyze the user's request. If it requires external tools (like file search, web search), output a tool call. If it can be answered directly, answer it.".to_string(),
        });

        let router_req = ChatRequest { messages };

        // Get the current configured provider
        let provider = self.model_provider.read().await.clone();

        provider.chat(router_req).await
    }
}
