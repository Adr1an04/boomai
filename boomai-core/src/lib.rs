pub mod provider;
pub mod providers;
pub mod types;

pub use provider::ModelProvider;
pub use providers::{DummyProvider, HttpProvider};
pub use types::{ChatRequest, ChatResponse, Message, Role};

pub fn hello() -> String {
    "Hello from Boomai Core!".to_string()
}
