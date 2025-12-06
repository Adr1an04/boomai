use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::types::{JsonRpcId, JsonRpcRequest, JsonRpcResponse, McpInitializeParams, McpCapabilities, McpClientInfo, McpInitializeResult};

pub struct McpClient {
    server_process: Arc<Mutex<Child>>, // keep child alive while client lives
    request_tx: mpsc::Sender<JsonRpcRequest>,
    pending_requests: Arc<Mutex<HashMap<String, mpsc::Sender<JsonRpcResponse>>>>,
}

impl McpClient {
    pub async fn new(command: &str, args: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Ensure cleanup on drop
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        // logging handler
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                eprintln!("[MCP Server Error]: {}", line);
            }
        });

        let (request_tx, mut request_rx) = mpsc::channel::<JsonRpcRequest>(32);
        let pending_requests: Arc<Mutex<HashMap<String, mpsc::Sender<JsonRpcResponse>>>> = Arc::new(Mutex::new(HashMap::new()));
        let pending_requests_clone = pending_requests.clone();

        // writer task
        let mut stdin_writer = stdin;
        tokio::spawn(async move {
            while let Some(req) = request_rx.recv().await {
                if let Ok(json_str) = serde_json::to_string(&req) {
                    let _ = stdin_writer.write_all(json_str.as_bytes()).await;
                    let _ = stdin_writer.write_all(b"\n").await;
                    let _ = stdin_writer.flush().await;
                }
            }
        });

        // reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                    if let Some(JsonRpcId::String(id)) = &response.id {
                        let mut pending = pending_requests_clone.lock().await;
                        if let Some(tx) = pending.remove(id) {
                            let _ = tx.send(response).await;
                        }
                    } else if let Some(JsonRpcId::Number(id)) = &response.id {
                        let mut pending = pending_requests_clone.lock().await;
                        if let Some(tx) = pending.remove(&id.to_string()) {
                            let _ = tx.send(response).await;
                        }
                    }
                }
            }
        });

        Ok(Self {
            server_process: Arc::new(Mutex::new(child)),
            request_tx,
            pending_requests,
        })
    }

    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(JsonRpcId::String(id.clone())),
        };

        let (response_tx, mut response_rx) = mpsc::channel(1);

        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), response_tx);
        }

        self.request_tx.send(request).await?;

        let response = response_rx.recv().await.ok_or("Connection closed")?;

        if let Some(error) = response.error {
            return Err(format!("RPC Error {}: {}", error.code, error.message).into());
        }

        response.result.ok_or("No result in response".into())
    }

    pub async fn initialize(&self) -> Result<McpInitializeResult, Box<dyn std::error::Error>> {
        let params = McpInitializeParams {
            protocol_version: "2024-11-05".to_string(), // Latest draft
            capabilities: McpCapabilities {
                experimental: None,
                sampling: None,
                roots: None,
            },
            client_info: McpClientInfo {
                name: "boomai-daemon".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let result_value = self.send_request("initialize", Some(serde_json::to_value(params)?)).await?;
        let result: McpInitializeResult = serde_json::from_value(result_value)?;
        
        // After initialize, send initialized notification
        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
            params: None,
            id: None,
        };
        self.request_tx.send(notification).await?;

        Ok(result)
    }
    
    pub async fn list_tools(&self) -> Result<super::types::McpListToolsResult, Box<dyn std::error::Error>> {
        let result_value = self.send_request("tools/list", None).await?;
        let result: super::types::McpListToolsResult = serde_json::from_value(result_value)?;
        Ok(result)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Best-effort kill on drop to avoid leaks.
        let server = self.server_process.clone();
        tokio::spawn(async move {
            let mut child = server.lock().await;
            let _ = child.kill().await;
        });
    }
}

