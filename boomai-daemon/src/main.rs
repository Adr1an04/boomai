use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use boomai_core::{ChatRequest, ChatResponse, DummyProvider, Message, ModelProvider, Role};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    model_provider: Arc<dyn ModelProvider>,
}

#[tokio::main]
async fn main() {
    // log test
    println!("Boomai core daemon (Rust) starting...");

    // grounds for ai model proivider
    let provider: Arc<dyn ModelProvider> = Arc::new(DummyProvider);
    let state = AppState {
        model_provider: provider,
    };

    // route set up for inital health and version checks
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version_check))
        .route("/chat", post(chat_handler))
        .with_state(state);

    // pass port from env or default to 3030
    let port = std::env::var("BOOMAI_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn version_check() -> Json<Value> {
    Json(json!({ "version": env!("CARGO_PKG_VERSION") }))
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    println!(
        "chat request with {} messages",
        payload.messages.len()
    );

    match state.model_provider.chat(payload).await {
        Ok(response) => Json(response),
        Err(err) => {
            eprintln!("Error chat request: {}", err);
            // error response
            Json(ChatResponse {
                message: Message {
                    role: Role::System,
                    content: format!("Error: {}", err),
                },
            })
        }
    }
}
