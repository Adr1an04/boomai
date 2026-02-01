use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::core::taint::TaintLevel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    FsRead,
    FsWrite,
    FsDelete,
    NetHttp,
    InternalStub,
    McpCall,
    RestrictedCommand,
    CreateRun,
    SpawnAgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl RunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityArgs {
    FsRead { path: PathBuf },
    FsWrite { path: PathBuf, bytes: usize, diff_preview: Option<String> },
    FsDelete { path: PathBuf },
    NetHttp { method: String, domain: String },
    InternalStub { name: String },
    McpCall { server: String, tool: String },
    RestrictedCommand { command: String, args: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityCaller {
    Orchestrator,
    ActorAgent { agent_id: String },
    ReaderAgent,
    Tool { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub run_id: RunId,
    pub capability: Capability,
    pub args: CapabilityArgs,
    pub caller: CapabilityCaller,
    pub taint: TaintLevel,
    pub step_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_request_round_trip() {
        let req = CapabilityRequest {
            run_id: RunId::new(),
            capability: Capability::FsRead,
            args: CapabilityArgs::FsRead { path: PathBuf::from("/tmp/one") },
            caller: CapabilityCaller::Orchestrator,
            taint: TaintLevel::UserProvided,
            step_id: Some("step-1".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CapabilityRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn capability_args_round_trip() {
        let args = CapabilityArgs::FsWrite {
            path: PathBuf::from("/tmp/two"),
            bytes: 12,
            diff_preview: Some("+hi".to_string()),
        };
        let json = serde_json::to_string(&args).unwrap();
        let back: CapabilityArgs = serde_json::from_str(&json).unwrap();
        assert_eq!(args, back);
    }
}
