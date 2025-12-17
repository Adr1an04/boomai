use crate::core::{Agent, AgentContext, ChatRequest, ChatResponse, Message, ModelRequest, ModelResponse, ProviderRegistry, Role, ExecutionStatus};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

pub struct RouterAgent {
    provider_registry: Arc<TokioRwLock<ProviderRegistry>>,
}

impl RouterAgent {
    pub fn new(provider_registry: Arc<TokioRwLock<ProviderRegistry>>) -> Self {
        Self { provider_registry }
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
