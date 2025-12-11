use crate::core::ModelProvider;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

use crate::agents::decomposer::DecomposerAgent;
use crate::agents::router::RouterAgent;
use crate::config_persistence::DaemonConfigStore;
use crate::local::LocalModelManager;
use crate::mcp::manager::McpManager;

#[derive(Clone)]
pub struct AppState {
    pub config_store: Arc<TokioRwLock<DaemonConfigStore>>,
    pub model_provider: Arc<TokioRwLock<Arc<dyn ModelProvider>>>,
    pub local_manager: LocalModelManager,
    pub mcp_manager: McpManager,

    // agents
    pub decomposer_agent: Arc<DecomposerAgent>,
    pub router_agent: Arc<RouterAgent>,
}
