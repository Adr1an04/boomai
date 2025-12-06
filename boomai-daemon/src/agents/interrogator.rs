use crate::core::{Agent, AgentContext, ChatRequest, ChatResponse, Message, Role, ExecutionStatus, ModelProvider};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct InterrogatorAgent {
    model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
}

impl InterrogatorAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for InterrogatorAgent {
    async fn handle_chat(&self, req: ChatRequest, _ctx: AgentContext) -> anyhow::Result<ChatResponse> {
        let mut messages = req.messages.clone();
        
        messages.insert(0, Message {
            role: Role::System,
            content: "You are the Interrogator Agent. Your job is to decide if the task is fully solved.
Review the history. 
- If the final answer has been reached and is consistent, output 'SOLVED'.
- If more steps are needed, output 'CONTINUE'.
- If the process is stuck or failing, output 'RETRY'.".to_string(),
        });

        let check_req = ChatRequest { messages };

        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                return Err(anyhow::anyhow!("Failed to acquire read lock on model provider"));
            }
        };

        let mut response = provider.chat(check_req).await?;
        response.status = ExecutionStatus::Done;
        response.maker_context = None;
        Ok(response)
    }
}

