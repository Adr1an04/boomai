use crate::core::{
    ChatRequest, ChatResponse, ExecutionStatus, HttpProvider, Message, ModelConfig, ModelProvider,
    Role,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;

use crate::agents::MakerOrchestrator;
use crate::config_persistence::{save_config, update_config};
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

pub async fn config_model_test(Json(config): Json<ModelConfig>) -> Json<Value> {
    println!("Testing model config: {:?}", config);

    let provider =
        HttpProvider::new(config.base_url.clone(), config.api_key.clone(), config.model.clone());

    let test_req =
        ChatRequest { messages: vec![Message { role: Role::User, content: "Hello".to_string() }] };

    match provider.chat(test_req).await {
        Ok(_) => Json(json!({ "status": "success", "message": "Connection successful" })),
        Err(e) => {
            Json(json!({ "status": "error", "message": format!("Connection failed: {}", e) }))
        }
    }
}

pub async fn config_model_save(
    State(state): State<AppState>,
    Json(new_config): Json<ModelConfig>,
) -> Json<Value> {
    println!("Saving model config: {:?}", new_config);

    let mut config_store = state.config_store.write().await;

    match update_config(&mut config_store, new_config).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Configuration saved and backed up",
            "backup_count": config_store.history.len()
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Configuration validation failed: {}", e)
        })),
    }
}

pub async fn config_model_reload(State(state): State<AppState>) -> Json<Value> {
    let config_store = state.config_store.read().await;
    let active_config = &config_store.active_config;

    // new provider with current config
    let new_provider = Arc::new(HttpProvider::new(
        active_config.base_url.clone(),
        active_config.api_key.clone(),
        active_config.model.clone(),
    ));

    if let Ok(mut lock) = state.model_provider.write() {
        *lock = new_provider;
        Json(json!({
            "status": "success",
            "message": "Provider reloaded with current configuration"
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": "Failed to acquire provider lock for reload"
        }))
    }
}

pub async fn config_model_rollback(
    State(state): State<AppState>,
    Path(index): Path<usize>,
) -> Json<Value> {
    let mut config_store = state.config_store.write().await;

    if let Some(rollback_config) = config_store.get_history_config(index).cloned() {
        // validate rollback config
        if let Err(e) = config_store.validate_config(&rollback_config) {
            return Json(json!({
                "status": "error",
                "message": format!("Rollback config is invalid: {}", e)
            }));
        }

        // update active config
        config_store.active_config = rollback_config;

        // save rolled-back state
        if let Err(e) = save_config(&config_store).await {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to save rollback: {}", e)
            }));
        }

        Json(json!({
            "status": "success",
            "message": format!("Rolled back to configuration {}", index),
            "new_config": config_store.active_config
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": format!("No configuration found at index {}", index)
        }))
    }
}

pub async fn config_local_available_models(State(state): State<AppState>) -> Json<Value> {
    let installed_ids: HashSet<String> =
        state.local_manager.get_installed_models().iter().map(|m| m.model_id.clone()).collect();
    let models: Vec<_> = crate::local::get_available_models()
        .into_iter()
        .filter(|m| !installed_ids.contains(&m.id))
        .collect();
    Json(json!({ "models": models }))
}

pub async fn config_local_installed_models(State(state): State<AppState>) -> Json<Value> {
    let models = state.local_manager.get_installed_models();
    Json(json!({ "models": models }))
}

pub async fn config_local_install_model(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let model_id = match payload["model_id"].as_str() {
        Some(id) => id,
        None => {
            return Json(json!({
                "status": "error",
                "message": "Missing model_id in request"
            }))
        }
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
        None => {
            return Json(json!({
                "status": "error",
                "message": "Missing model_id in request"
            }))
        }
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

pub async fn config_mcp_servers_list(State(state): State<AppState>) -> Json<Value> {
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
        Ok(_) => {
            Json(json!({ "status": "success", "message": format!("MCP server {} added", id) }))
        }
        Err(e) => {
            Json(json!({ "status": "error", "message": format!("Failed to add server: {}", e) }))
        }
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
            Ok(tools) => Json(
                serde_json::to_value(tools).unwrap_or(json!({ "error": "Serialization failed" })),
            ),
            Err(e) => Json(
                json!({ "status": "error", "message": format!("Failed to list tools: {}", e) }),
            ),
        }
    } else {
        Json(json!({ "status": "error", "message": "Server not found" }))
    }
}

#[axum::debug_handler]
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    println!("Received chat request with {} messages", payload.messages.len());

    let orchestrator = MakerOrchestrator::new(Arc::new(state.clone()));

    match orchestrator.run(payload).await {
        Ok(response) => Json(response),
        Err(err) => {
            eprintln!("Error handling chat request via orchestrator: {}", err);
            Json(ChatResponse {
                message: Message { role: Role::System, content: format!("Error: {}", err) },
                status: ExecutionStatus::Failed,
                maker_context: None,
            })
        }
    }
}
