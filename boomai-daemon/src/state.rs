use crate::core::ProviderRegistry;
use std::sync::Arc;
use tokio::sync::{RwLock as TokioRwLock, Semaphore};

use crate::agents::decomposer::DecomposerAgent;
use crate::agents::router::RouterAgent;
use crate::config_persistence::DaemonConfigStore;
use crate::local::LocalModelManager;
use crate::mcp::manager::McpManager;

#[derive(Clone)]
pub struct AppState {
    pub config_store: Arc<TokioRwLock<DaemonConfigStore>>,
    pub provider_registry: Arc<TokioRwLock<ProviderRegistry>>,
    pub _global_concurrency_limiter: Arc<Semaphore>, // Global max in-flight requests (used during provider registration)
    pub local_manager: LocalModelManager,
    pub mcp_manager: McpManager,

    // agents
    pub decomposer_agent: Arc<DecomposerAgent>,
    pub router_agent: Arc<RouterAgent>,
}
