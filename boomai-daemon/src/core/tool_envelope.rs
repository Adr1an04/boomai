use serde::{Deserialize, Serialize};

use crate::core::capabilities::CapabilityRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolRequest {
    pub capability_request: CapabilityRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResponse {
    pub ok: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}
