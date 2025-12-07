use crate::core::{HttpProvider, ModelProvider};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;

mod agents;
mod config;
mod config_persistence;
mod core;
mod handlers;
mod local;
mod mcp;
mod state;
mod system;

use agents::calculator::CalculatorAgent;
use agents::classifier::ClassifierAgent;
use agents::decomposer::DecomposerAgent;
use agents::interrogator::InterrogatorAgent;
use agents::router::RouterAgent;
use agents::verifier::VerifierAgent;
use config::Config;
use config_persistence::{config_exists, load_config, save_config, DaemonConfigStore};
use handlers::{
    chat_handler, config_local_available_models, config_local_install_model,
    config_local_installed_models, config_local_uninstall_model, config_mcp_server_add,
    config_mcp_servers_list, config_mcp_tools_list, config_model_reload, config_model_rollback,
    config_model_save, config_model_test, health_check, system_profile_handler,
    system_recommendation_handler, version_check,
};
use local::LocalModelManager;
use mcp::manager::McpManager;
use state::AppState;

#[tokio::main]
async fn main() {
    println!("Boomai core daemon (Rust) starting...");

    // Load persistent configuration
    let config_store = match load_config().await {
        Ok(store) => {
            println!("Loaded configuration from disk");
            store
        }
        Err(e) => {
            eprintln!("Failed to load configuration, using defaults: {}", e);
            let default_config = crate::core::ModelConfig {
                base_url: "http://127.0.0.1:11434/v1".to_string(),
                api_key: None,
                model: "tinyllama".to_string(),
            };
            DaemonConfigStore::new(default_config)
        }
    };

    // new provider with loaded config
    let provider: Arc<dyn ModelProvider> = Arc::new(HttpProvider::new(
        config_store.active_config.base_url.clone(),
        config_store.active_config.api_key.clone(),
        config_store.active_config.model.clone(),
    ));
    let provider_lock = Arc::new(RwLock::new(provider.clone()));

    let local_manager = LocalModelManager::new();
    let mcp_manager = McpManager::new();

    let decomposer_agent = Arc::new(DecomposerAgent::new(provider_lock.clone()));
    let router_agent = Arc::new(RouterAgent::new(provider_lock.clone()));
    let verifier_agent = Arc::new(VerifierAgent::new(provider_lock.clone()));
    let classifier_agent = Arc::new(ClassifierAgent::new(provider_lock.clone()));
    let calculator_agent = Arc::new(CalculatorAgent::new(provider_lock.clone()));
    let interrogator_agent = Arc::new(InterrogatorAgent::new(provider_lock.clone()));

    let config_store_lock = Arc::new(TokioRwLock::new(config_store));

    let state = AppState {
        config_store: config_store_lock.clone(),
        model_provider: provider_lock,
        local_manager,
        mcp_manager,
        decomposer_agent,
        router_agent,
        verifier_agent,
        classifier_agent,
        calculator_agent,
        interrogator_agent,
    };

    if !config_exists().await {
        let store = config_store_lock.read().await;
        if let Err(e) = save_config(&store).await {
            eprintln!("Failed to save initial config: {}", e);
        }
    }

    let config = Config::from_env();

    // route set up
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version_check))
        .route("/system/profile", get(system_profile_handler))
        .route("/system/recommendation", get(system_recommendation_handler))
        .route("/config/model", post(config_model_save))
        .route("/config/model/test", post(config_model_test))
        .route("/config/model/reload", post(config_model_reload))
        .route("/config/model/rollback/:index", post(config_model_rollback))
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

    let listener = tokio::net::TcpListener::bind(config.addr).await.unwrap_or_else(|e| {
        eprintln!(
            "Failed to bind to {}: {}\nHint: Port might be in use. Try: lsof -i :{}",
            config.addr,
            e,
            config.addr.port()
        );
        std::process::exit(1);
    });
    axum::serve(listener, app).await.unwrap();
}
