use crate::core::tool_envelope::{ToolRequest, ToolResponse};

pub struct ToolRouter {}

impl ToolRouter {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, _req: ToolRequest) -> ToolResponse {
        ToolResponse {
            ok: false,
            output: None,
            error: Some("Tool execution not wired yet".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::{
        Capability, CapabilityArgs, CapabilityCaller, CapabilityRequest, RunId,
    };
    use crate::core::taint::TaintLevel;
    use crate::core::tool_envelope::ToolRequest;

    #[tokio::test]
    async fn tool_router_denies_by_default() {
        let router = ToolRouter::new();
        let req = ToolRequest {
            capability_request: CapabilityRequest {
                run_id: RunId::new(),
                capability: Capability::InternalStub,
                args: CapabilityArgs::InternalStub { name: "calculator".to_string() },
                caller: CapabilityCaller::Orchestrator,
                taint: TaintLevel::UserProvided,
                step_id: None,
            },
        };
        let resp = router.execute(req).await;
        assert!(!resp.ok);
        assert!(resp.error.is_some());
    }
}
