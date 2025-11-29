use boomai_core::{Agent, AgentContext, ChatRequest, ChatResponse, Message, Role};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct DecomposerAgent {
    model_provider: Arc<RwLock<Arc<dyn boomai_core::ModelProvider>>>,
}

impl DecomposerAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn boomai_core::ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for DecomposerAgent {
    async fn handle_chat(&self, req: ChatRequest, _ctx: AgentContext) -> anyhow::Result<ChatResponse> {
        // Basic decomposition logic: For now, just pass through or add a system prompt
        // In a real implementation, this would break down the task.
        // MAKER principle: m=1 (Maximal Decomposition)

        let mut messages = req.messages.clone();
        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Planning Agent. Given a user request, identify the IMMEDIATE next step to solve it. Keep it extremely brief. Example: 'Calculate 15 * 23'.".to_string(),
        });

        let decompose_req = ChatRequest { messages };

        // Get the current configured provider
        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                eprintln!("Failed to acquire read lock on model provider in DecomposerAgent");
                return Ok(ChatResponse {
                    message: Message {
                        role: Role::System,
                        content: "Internal Error: Failed to access model provider".to_string(),
                    },
                    status: boomai_core::types::ExecutionStatus::Failed,
                    maker_context: None,
                });
            }
        };

        provider.chat(decompose_req).await
    }
}

