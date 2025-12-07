use crate::core::{
    Agent, AgentContext, ChatRequest, ChatResponse, ExecutionStatus, Message, ModelProvider, Role,
};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct VerifierAgent {
    model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
}

impl VerifierAgent {
    pub fn new(model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>) -> Self {
        Self { model_provider }
    }
}

#[async_trait]
impl Agent for VerifierAgent {
    async fn handle_chat(
        &self,
        req: ChatRequest,
        _ctx: AgentContext,
    ) -> anyhow::Result<ChatResponse> {
        // Verifier logic: Check the proposed answer for errors.

        let mut messages = req.messages.clone();
        // In a real flow, the 'last' message would be the candidate answer to verify.
        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Verifier Agent. Review the last message (the proposed solution). If it is correct, output 'CORRECT'. If it is incorrect, explain the error.".to_string(),
        });

        let verify_req = ChatRequest { messages };

        // Get the current configured provider
        let provider = {
            if let Ok(guard) = self.model_provider.read() {
                guard.clone()
            } else {
                eprintln!("Failed to acquire read lock on model provider in VerifierAgent");
                return Ok(ChatResponse {
                    message: Message {
                        role: Role::System,
                        content: "Internal Error: Failed to access model provider".to_string(),
                    },
                    status: ExecutionStatus::Failed,
                    maker_context: None,
                });
            }
        };

        provider.chat(verify_req).await
    }
}
