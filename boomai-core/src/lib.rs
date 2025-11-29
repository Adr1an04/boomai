pub mod provider;
pub mod providers;
pub mod types;
pub mod agent;

pub use provider::ModelProvider;
pub use providers::{DummyProvider, HttpProvider};
pub use types::{AvailableLocalModel, ChatRequest, ChatResponse, InstalledLocalModel, Message, ModelConfig, Role, ExecutionStatus};
pub use agent::{Agent, AgentContext};

pub fn hello() -> String {
    "Hello from Boomai Core!".to_string()
}
