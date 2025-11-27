use axum::{extract::State, Json};
use boomai_core::{ChatRequest, ChatResponse, HttpProvider, Message, ModelConfig, ModelProvider, Role};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::state::AppState;
use crate::system::{get_recommendation, get_system_profile, EngineRecommendation, SystemProfile};

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn version_check() -> Json<Value> {
    Json(json!({ "version": env!("CARGO_PKG_VERSION") }))
}

pub async fn system_profile_handler() -> Json<SystemProfile> {
    Json(get_system_profile())
}

pub async fn system_recommendation_handler() -> Json<EngineRecommendation> {
    let profile = get_system_profile();
    Json(get_recommendation(&profile))
}

pub async fn config_model_test(
    Json(config): Json<ModelConfig>,
) -> Json<Value> {
    println!("Testing model config: {:?}", config);

    let provider = HttpProvider::new(
        config.base_url.clone(),
        config.api_key.clone(),
        config.model.clone(),
    );

    let test_req = ChatRequest {
        messages: vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
        }],
    };

    match provider.chat(test_req).await {
        Ok(_) => Json(json!({ "status": "success", "message": "Connection successful" })),
        Err(e) => Json(json!({ "status": "error", "message": format!("Connection failed: {}", e) })),
    }
}

pub async fn config_model_save(
    State(state): State<AppState>,
    Json(config): Json<ModelConfig>,
) -> Json<Value> {
    println!("Saving model config: {:?}", config);

    let new_provider = Arc::new(HttpProvider::new(
        config.base_url,
        config.api_key,
        config.model,
    ));

    // Acquire write lock and update the provider
    if let Ok(mut provider_lock) = state.model_provider.write() {
        *provider_lock = new_provider;
        Json(json!({ "status": "success", "message": "Configuration saved" }))
    } else {
        Json(json!({ "status": "error", "message": "Failed to acquire lock" }))
    }
}

pub async fn config_local_available_models() -> Json<Value> {
    let models = crate::local::get_available_models();
    Json(json!({ "models": models }))
}

pub async fn config_local_installed_models(
    State(state): State<AppState>,
) -> Json<Value> {
    let models = state.local_manager.get_installed_models();
    Json(json!({ "models": models }))
}

pub async fn config_local_install_model(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let model_id = match payload["model_id"].as_str() {
        Some(id) => id,
        None => return Json(json!({
            "status": "error",
            "message": "Missing model_id in request"
        })),
    };

    println!("Installing local model: {}", model_id);

    match state.local_manager.install_model(model_id).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": format!("Model {} installed successfully", model_id)
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Installation failed: {}", e)
        })),
    }
}

pub async fn config_local_uninstall_model(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let model_id = match payload["model_id"].as_str() {
        Some(id) => id,
        None => return Json(json!({
            "status": "error",
            "message": "Missing model_id in request"
        })),
    };

    println!("Uninstalling local model: {}", model_id);

    match state.local_manager.uninstall_model(model_id).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": format!("Model {} uninstalled successfully", model_id)
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Uninstallation failed: {}", e)
        })),
    }
}

// MCP Handlers

pub async fn config_mcp_servers_list(
    State(state): State<AppState>,
) -> Json<Value> {
    let servers = state.mcp_manager.list_clients().await;
    Json(json!({ "servers": servers }))
}

pub async fn config_mcp_server_add(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let id = match payload["id"].as_str() {
        Some(id) => id,
        None => return Json(json!({ "status": "error", "message": "Missing server id" })),
    };
    
    let command = match payload["command"].as_str() {
        Some(cmd) => cmd,
        None => return Json(json!({ "status": "error", "message": "Missing command" })),
    };

    let args_vec = match payload["args"].as_array() {
        Some(arr) => arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>(),
        None => vec![],
    };

    match state.mcp_manager.add_client(id.to_string(), command, &args_vec).await {
        Ok(_) => Json(json!({ "status": "success", "message": format!("MCP server {} added", id) })),
        Err(e) => Json(json!({ "status": "error", "message": format!("Failed to add server: {}", e) })),
    }
}

pub async fn config_mcp_tools_list(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let server_id = match payload["server_id"].as_str() {
        Some(id) => id,
        None => return Json(json!({ "status": "error", "message": "Missing server_id" })),
    };

    if let Some(client) = state.mcp_manager.get_client(server_id).await {
        match client.list_tools().await {
            Ok(tools) => Json(serde_json::to_value(tools).unwrap_or(json!({ "error": "Serialization failed" }))),
            Err(e) => Json(json!({ "status": "error", "message": format!("Failed to list tools: {}", e) })),
        }
    } else {
        Json(json!({ "status": "error", "message": "Server not found" }))
    }
}

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    println!(
        "Received chat request with {} messages",
        payload.messages.len()
    );

    // get read lock to get the current provider
    let provider = {
        if let Ok(guard) = state.model_provider.read() {
            guard.clone()
        } else {
             // fails
             eprintln!("Failed to acquire read lock on model provider");
             return Json(ChatResponse {
                message: Message {
                    role: Role::System,
                    content: "Internal System Error: Failed to access model provider".to_string(),
                },
            });
        }
    };

    match provider.chat(payload).await {
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
