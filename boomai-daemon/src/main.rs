use axum::{
    routing::{get, post},
    Router,
};
use boomai_core::{DummyProvider, ModelProvider};
use std::sync::Arc;

mod config;
mod handlers;
mod state;
mod system;

use config::Config;
use handlers::{chat_handler, health_check, system_profile_handler, system_recommendation_handler, version_check};
use state::AppState;

#[tokio::main]
async fn main() {
    // log test
    println!("Boomai core daemon (Rust) starting...");

    // Load config
    let config = Config::from_env();

    // Initialize provider - default to DummyProvider for now
    // In the future, we can load this from config or env vars
    let provider: Arc<dyn ModelProvider> = Arc::new(DummyProvider);
    let state = AppState {
        model_provider: provider,
    };

    // route set up for inital health and version checks
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version_check))
        .route("/system/profile", get(system_profile_handler))
        .route("/system/recommendation", get(system_recommendation_handler))
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
