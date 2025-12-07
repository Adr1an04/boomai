use crate::core::ModelProvider;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;

use crate::agents::calculator::CalculatorAgent;
use crate::agents::classifier::ClassifierAgent;
use crate::agents::decomposer::DecomposerAgent;
use crate::agents::interrogator::InterrogatorAgent;
use crate::agents::router::RouterAgent;
use crate::agents::verifier::VerifierAgent;
use crate::config_persistence::DaemonConfigStore;
use crate::local::LocalModelManager;
use crate::mcp::manager::McpManager;

#[derive(Clone)]
pub struct AppState {
    pub config_store: Arc<TokioRwLock<DaemonConfigStore>>,
    pub model_provider: Arc<RwLock<Arc<dyn ModelProvider>>>,
    pub local_manager: LocalModelManager,
    pub mcp_manager: McpManager,

    // agents
    pub decomposer_agent: Arc<DecomposerAgent>,
    pub router_agent: Arc<RouterAgent>,
    pub verifier_agent: Arc<VerifierAgent>,
    pub classifier_agent: Arc<ClassifierAgent>,
    pub calculator_agent: Arc<CalculatorAgent>,
    pub interrogator_agent: Arc<InterrogatorAgent>,
}
