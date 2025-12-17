use crate::core::types::{Message, ModelId};
use serde::{Deserialize, Serialize};

/// Comprehensive request structure that covers all provider needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>, // optional
    pub response_format: Option<ResponseFormat>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Vec<String>,
    pub seed: Option<u64>,
    pub stream: bool,

    // routing hints
    pub tags: Vec<String>,             // "classification", "code", "reasoning"
    pub priority: RequestPriority,     // interactive vs batch
    pub hard_deadline_ms: Option<u64>, // end-to-end SLA
    pub require_json: bool,

    // context control
    pub truncation: TruncationPolicy, // error vs auto-trim
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Interactive,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruncationPolicy {
    ErrorIfTooLarge,
    AutoTrimOldest,
    SummarizeThenTrim { max_depth: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    pub r#type: ResponseFormatType,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormatType {
    Text,
    Json,
}

/// Comprehensive response structure with normalized fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    pub usage: Usage, // prompt_tokens, completion_tokens
    pub model_id: ModelId,
    pub latency_ms: u64,
    pub warnings: Vec<ResponseWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWarning {
    pub kind: WarningKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningKind {
    ContextTruncated,
    TokenLimitApproached,
    UnsupportedFeature,
    Performance,
}

impl Default for ModelRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            tools: Vec::new(),
            response_format: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            stop: Vec::new(),
            seed: None,
            stream: false,
            tags: Vec::new(),
            priority: RequestPriority::Background,
            hard_deadline_ms: None,
            require_json: false,
            truncation: TruncationPolicy::ErrorIfTooLarge,
        }
    }
}
