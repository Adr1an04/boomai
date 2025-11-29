use async_trait::async_trait;
use crate::types::{ChatRequest, ChatResponse, MAKERContext, AgentStep};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub task_id: String,
    pub step_number: usize,
    pub depth: usize,
    pub max_depth: usize,
    pub maker_context: Option<MAKERContext>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn handle_chat(&self, req: ChatRequest, ctx: AgentContext) -> anyhow::Result<ChatResponse>;
    // granular step execution
    async fn handle_task(&self, _req: ChatRequest, _ctx: &MAKERContext) -> anyhow::Result<AgentStep> {
        // backward compatibility or simple agents
        Ok(AgentStep {
            agent_type: "Generic".to_string(),
            input_context: "N/A".to_string(),
            votes_drawn: 0,
            result_action: "N/A".to_string(),
            decision_made: true,
        })
    }
}

