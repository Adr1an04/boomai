use axum::{
    routing::{get, post},
    Json, Router,
};
use boomai_core::{ChatRequest, ChatResponse, Message, Role};
use serde_json::{json, Value};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // log test
    println!("Boomai core daemon (Rust) starting...");

    // route set up for inital health and version checks
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version_check))
        .route("/chat", post(chat_handler));

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

async fn chat_handler(Json(payload): Json<ChatRequest>) -> Json<ChatResponse> {
    println!("Received chat request with {} messages", payload.messages.len());
    
    // test mock response rn
    let response_text = format!("I received your message: '{}'", 
        payload.messages.last().map(|m| m.content.as_str()).unwrap_or(""));

    let response = ChatResponse {
        message: Message {
            role: Role::Assistant,
            content: response_text,
        },
    };
    Json(response)
}
