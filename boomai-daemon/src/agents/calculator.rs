use crate::core::{
    Agent, AgentContext, ChatRequest, ChatResponse, ExecutionStatus, Message, ModelProvider, Role,
};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct CalculatorAgent {
    model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
}

impl CalculatorAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for CalculatorAgent {
    async fn handle_chat(
        &self,
        req: ChatRequest,
        _ctx: AgentContext,
    ) -> anyhow::Result<ChatResponse> {
        let mut messages = req.messages.clone();

        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Calculator. Perform the math requested. Output ONLY the calculation and the result. No chatter.".to_string(),
        });

        let calc_req = ChatRequest { messages };

        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                return Err(anyhow::anyhow!("Failed to acquire read lock on model provider"));
            }
        };

        let mut response = provider.chat(calc_req).await?;
        response.status = ExecutionStatus::Done;
        response.maker_context = None;
        Ok(response)
    }
}
