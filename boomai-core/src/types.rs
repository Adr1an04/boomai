use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    // later: metadata like workspace_id, tools, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AvailableLocalModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size_gb: f64,
    pub recommended_ram_gb: u32,
    pub download_url: String,
    pub local_port: u16,
    pub runtime_type: String, // "ollama", "lm-studio", etc.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledLocalModel {
    pub model_id: String,
    pub install_path: String,
    pub is_running: bool,
    pub port: u16,
    pub runtime_type: String,
}
