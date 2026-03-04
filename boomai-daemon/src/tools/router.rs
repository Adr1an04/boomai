use crate::core::{
    tool_envelope::{ToolRequest, ToolResponse},
    Capability, CapabilityArgs,
};
use crate::safety::{RiskAction, RiskDecision, RiskLevel, SafetySidecar};
use crate::tools::stubs::run_internal_stub;
use serde_json::json;
use tracing::warn;

pub struct ToolRouter {
    sidecar: SafetySidecar,
}

impl ToolRouter {
    pub fn new() -> Self {
        Self { sidecar: SafetySidecar::new(Default::default()) }
    }

    pub async fn execute(&self, req: ToolRequest) -> ToolResponse {
        let decision = self.sidecar.evaluate_tool_request(&req);

        match decision.action {
            RiskAction::Block => denied_response(decision, false),
            RiskAction::RequireHumanApproval => denied_response(decision, true),
            RiskAction::ExecuteWithAudit => {
                warn!(
                    target: "security",
                    capability = ?req.capability_request.capability,
                    reason = %decision.reason,
                    "tool request allowed with amber audit"
                );
                self.execute_allowed(req, decision.level).await
            }
            RiskAction::Execute => self.execute_allowed(req, decision.level).await,
        }
    }

    async fn execute_allowed(&self, req: ToolRequest, risk_level: RiskLevel) -> ToolResponse {
        match (&req.capability_request.capability, req.capability_request.args) {
            (Capability::InternalStub, CapabilityArgs::InternalStub { name, input }) => {
                match run_internal_stub(&name, input.as_deref()) {
                    Some(result) => success_response(json!({ "tool": name, "result": result }), risk_level),
                    None => ToolResponse {
                        ok: false,
                        output: None,
                        error: Some(format!("internal stub '{name}' failed")),
                        risk: Some(risk_level.as_str().to_string()),
                        requires_confirmation: false,
                    },
                }
            }
            (Capability::FsRead, CapabilityArgs::FsRead { path }) => {
                match tokio::fs::read_to_string(&path).await {
                    Ok(contents) => success_response(
                        json!({ "path": path.display().to_string(), "contents": contents }),
                        risk_level,
                    ),
                    Err(err) => ToolResponse {
                        ok: false,
                        output: None,
                        error: Some(format!("failed to read file '{}': {}", path.display(), err)),
                        risk: Some(risk_level.as_str().to_string()),
                        requires_confirmation: false,
                    },
                }
            }
            _ => ToolResponse {
                ok: false,
                output: None,
                error: Some(
                    "tool request passed policy checks but executor for this capability is not wired"
                        .to_string(),
                ),
                risk: Some(risk_level.as_str().to_string()),
                requires_confirmation: false,
            },
        }
    }
}

fn success_response(output: serde_json::Value, risk_level: RiskLevel) -> ToolResponse {
    ToolResponse {
        ok: true,
        output: Some(output),
        error: None,
        risk: Some(risk_level.as_str().to_string()),
        requires_confirmation: false,
    }
}

fn denied_response(decision: RiskDecision, requires_confirmation: bool) -> ToolResponse {
    ToolResponse {
        ok: false,
        output: None,
        error: Some(if requires_confirmation {
            format!("blocked pending explicit user approval: {}", decision.reason)
        } else {
            format!("blocked by safety sidecar: {}", decision.reason)
        }),
        risk: Some(decision.level.as_str().to_string()),
        requires_confirmation,
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
    async fn tool_router_executes_allowed_internal_stub() {
        let router = ToolRouter::new();
        let req = ToolRequest {
            capability_request: CapabilityRequest {
                run_id: RunId::new(),
                capability: Capability::InternalStub,
                args: CapabilityArgs::InternalStub {
                    name: "calculator".to_string(),
                    input: Some("12 * 2 + 1".to_string()),
                },
                caller: CapabilityCaller::Orchestrator,
                taint: TaintLevel::UserProvided,
                step_id: None,
            },
        };
        let resp = router.execute(req).await;
        assert!(resp.ok);
        assert_eq!(resp.risk.as_deref(), Some("green"));
    }

    #[tokio::test]
    async fn tool_router_blocks_unknown_internal_stub() {
        let router = ToolRouter::new();
        let req = ToolRequest {
            capability_request: CapabilityRequest {
                run_id: RunId::new(),
                capability: Capability::InternalStub,
                args: CapabilityArgs::InternalStub { name: "bash".to_string(), input: None },
                caller: CapabilityCaller::Orchestrator,
                taint: TaintLevel::UserProvided,
                step_id: None,
            },
        };
        let resp = router.execute(req).await;
        assert!(!resp.ok);
        assert_eq!(resp.risk.as_deref(), Some("red"));
    }

    #[tokio::test]
    async fn tool_router_requires_confirmation_for_tainted_amber_calls() {
        let router = ToolRouter::new();
        let req = ToolRequest {
            capability_request: CapabilityRequest {
                run_id: RunId::new(),
                capability: Capability::McpCall,
                args: CapabilityArgs::McpCall {
                    server: "local".to_string(),
                    tool: "search".to_string(),
                },
                caller: CapabilityCaller::Orchestrator,
                taint: TaintLevel::RetrievedUntrusted,
                step_id: None,
            },
        };
        let resp = router.execute(req).await;
        assert!(!resp.ok);
        assert!(resp.requires_confirmation);
        assert_eq!(resp.risk.as_deref(), Some("red"));
    }
}
