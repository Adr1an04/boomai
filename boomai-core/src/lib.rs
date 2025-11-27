pub mod provider;
pub mod providers;
pub mod types;

pub use provider::ModelProvider;
pub use providers::{DummyProvider, HttpProvider};
pub use types::{AvailableLocalModel, ChatRequest, ChatResponse, InstalledLocalModel, Message, ModelConfig, Role};

pub fn hello() -> String {
    "Hello from Boomai Core!".to_string()
}
