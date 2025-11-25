use axum::{extract::State, Json};
use boomai_core::{ChatRequest, ChatResponse, Message, Role};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn version_check() -> Json<Value> {
    Json(json!({ "version": env!("CARGO_PKG_VERSION") }))
}

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    println!(
        "Received chat request with {} messages",
        payload.messages.len()
    );

    match state.model_provider.chat(payload).await {
        Ok(response) => Json(response),
        Err(err) => {
            eprintln!("Error handling chat request: {}", err);
            // Return an error message in the chat format for now
            Json(ChatResponse {
                message: Message {
                    role: Role::System,
                    content: format!("Error: {}", err),
                },
            })
        }
    }
}

