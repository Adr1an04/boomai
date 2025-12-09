use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StepType {
    Deterministic, // Math, time, system info
    Probabilistic, // Creative or reasoning-heavy tasks
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum StepKind {
    Math,
    Time,
    Creative,
    #[default]
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanStep {
    pub id: usize,
    pub description: String,
    pub step_type: StepType,
    #[serde(default)]
    pub context_keys: Vec<String>,
    #[serde(default)]
    pub kind: StepKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionPolicy {
    DecomposeAndExecute,
    InternalStub { tool_name: String, args: String },
    SingleProbe { prompt: String },
    MakerRace { prompt: String, n: usize, k: usize },
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum ExecutionStatus {
    Classifying,
    Decomposing,
    Voting { round: u32 },
    ToolCall { tool: String },
    Solved,
    Error,
    Processing,
    Done,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStep {
    pub agent_type: String,
    pub input_context: String,
    pub votes_drawn: u32,
    pub result_action: String,
    pub decision_made: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MAKERContext {
    pub current_depth: u8,
    pub max_depth: u8,
    pub history: Vec<AgentStep>,
    pub k_min: u8,
    pub t_target: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    #[serde(default = "default_status")]
    pub status: ExecutionStatus,
    #[serde(default)]
    pub maker_context: Option<MAKERContext>,
}

fn default_status() -> ExecutionStatus {
    ExecutionStatus::Done
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ModelConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

impl ModelConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(anyhow::anyhow!("base_url cannot be empty"));
        }

        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(anyhow::anyhow!("base_url must be a valid HTTP/HTTPS URL"));
        }

        if self.model.trim().is_empty() {
            return Err(anyhow::anyhow!("model name cannot be empty"));
        }

        Ok(())
    }
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
    pub runtime_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledLocalModel {
    pub model_id: String,
    pub install_path: String,
    pub is_running: bool,
    pub port: u16,
    pub runtime_type: String,
}

// --- MCP core types ---

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum McpTransport {
    /// Local process via stdio (e.g., npx/pip tool).
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: Vec<(String, String)>,
    },
    /// Remote SSE/HTTP endpoint (e.g., hosted MCP server).
    Sse {
        url: String,
        #[serde(default)]
        api_key: Option<String>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpManifest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub required_env_vars: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledMod {
    pub id: String,
    pub manifest: McpManifest,
    pub transport: McpTransport,
    #[serde(default)]
    pub enabled: bool,
}
