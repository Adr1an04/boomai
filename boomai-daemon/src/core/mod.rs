pub mod agent;
pub mod provider;
pub mod providers;
pub mod types;

pub use agent::{Agent, AgentContext};
pub use provider::ModelProvider;
pub use providers::HttpProvider;
pub use types::{
    AvailableLocalModel, ChatRequest, ChatResponse, ExecutionStatus, InstalledLocalModel,
    Message, ModelConfig, Role,
};

