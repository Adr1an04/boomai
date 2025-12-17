use crate::core::{
    Agent, AgentContext, ChatRequest, ChatResponse, ExecutionStatus, Message, ModelRequest,
    ProviderRegistry, Role,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

pub struct DecomposerAgent {
    provider_registry: Arc<TokioRwLock<ProviderRegistry>>,
}

impl DecomposerAgent {
    pub fn new(provider_registry: Arc<TokioRwLock<ProviderRegistry>>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait]
impl Agent for DecomposerAgent {
    async fn handle_chat(
        &self,
        req: ChatRequest,
        _ctx: AgentContext,
    ) -> anyhow::Result<ChatResponse> {
        // Basic decomposition logic: For now, just pass through or add a system prompt
        // In a real implementation, this would break down the task.
        // MAKER principle: m=1 (Maximal Decomposition)

        let mut messages = req.messages.clone();
        messages.insert(0, Message {
            role: Role::System,
            content: "You are a Planning Agent. Given a user request, identify the IMMEDIATE next step to solve it. Keep it extremely brief. Example: 'Calculate 15 * 23'.".to_string(),
        });

        let model_req = ModelRequest {
            messages,
            tools: Vec::new(),
            response_format: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            stop: Vec::new(),
            seed: None,
            stream: false,
            tags: Vec::new(),
            priority: crate::core::model_request::RequestPriority::Background,
            hard_deadline_ms: None,
            require_json: false,
            truncation: crate::core::model_request::TruncationPolicy::ErrorIfTooLarge,
        };

        // Execute using the provider registry
        let registry = self.provider_registry.read().await;
        let model_resp = registry.execute_default(model_req).await?;

        // Convert back to ChatResponse
        Ok(ChatResponse {
            message: Message { role: Role::Assistant, content: model_resp.content },
            status: ExecutionStatus::Done,
            maker_context: None,
        })
    }
}
