use axum::{
    routing::{get, post},
    Router,
};
use boomai_core::{DummyProvider, ModelProvider};
use std::sync::{Arc, RwLock};

mod config;
mod handlers;
mod state;
mod system;
mod local;
mod mcp;

use config::Config;
use handlers::{
    chat_handler, config_local_available_models, config_local_install_model,
    config_local_installed_models, config_local_uninstall_model, config_mcp_server_add,
    config_mcp_servers_list, config_mcp_tools_list, config_model_save, config_model_test,
    health_check, system_profile_handler, system_recommendation_handler, version_check,
};
use state::AppState;
use local::LocalModelManager;
use mcp::manager::McpManager;

#[tokio::main]
async fn main() {
    // log test
    println!("Boomai core daemon (Rust) starting...");

    // Load config
    let config = Config::from_env();

    // Initialize provider - default to DummyProvider for now
    // Wrap in RwLock for dynamic updates
    let provider: Arc<dyn ModelProvider> = Arc::new(DummyProvider);
    let local_manager = LocalModelManager::new();
    let mcp_manager = McpManager::new();

    let state = AppState {
        model_provider: Arc::new(RwLock::new(provider)),
        local_manager,
        mcp_manager,
    };

    // route set up for inital health and version checks
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version_check))
        .route("/system/profile", get(system_profile_handler))
        .route("/system/recommendation", get(system_recommendation_handler))
        .route("/config/model", post(config_model_save))
        .route("/config/model/test", post(config_model_test))
        .route("/config/local/available_models", get(config_local_available_models))
        .route("/config/local/installed_models", get(config_local_installed_models))
        .route("/config/local/install_model", post(config_local_install_model))
        .route("/config/local/uninstall_model", post(config_local_uninstall_model))
        .route("/config/mcp/servers", get(config_mcp_servers_list))
        .route("/config/mcp/server/add", post(config_mcp_server_add))
        .route("/config/mcp/tools", post(config_mcp_tools_list))
        .route("/chat", post(chat_handler))
        .with_state(state);

    println!("Listening on http://{}", config.addr);

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!(
                "Failed to bind to {}: {}\nHint: Port might be in use. Try: lsof -i :{}",
                config.addr, e, config.addr.port()
            );
            std::process::exit(1);
        });
    axum::serve(listener, app).await.unwrap();
}
