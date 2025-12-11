use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use super::types::{
    JsonRpcId, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpClientInfo,
    McpInitializeParams, McpInitializeResult, McpListToolsResult,
};

enum Transport {
    Stdio {
        server_process: Arc<Mutex<Child>>,
        request_tx: mpsc::Sender<JsonRpcRequest>,
        pending: Arc<Mutex<HashMap<String, mpsc::Sender<JsonRpcResponse>>>>,
    },
    Http {
        client: reqwest::Client,
        url: String,
        api_key: Option<String>,
    },
}

pub struct McpClient {
    transport: Transport,
}

impl McpClient {
    /// Connect to a local MCP server via stdio
    pub async fn connect_stdio(
        command: &str,
        args: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        // stderr logging
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                eprintln!("[MCP stderr]: {}", line);
            }
        });

        let (request_tx, mut request_rx) = mpsc::channel::<JsonRpcRequest>(32);
        let pending: Arc<Mutex<HashMap<String, mpsc::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        // writer
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

        // reader
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                    if let Some(JsonRpcId::String(id)) = &response.id {
                        let mut pending = pending_clone.lock().await;
                        if let Some(tx) = pending.remove(id) {
                            let _ = tx.send(response).await;
                        }
                    } else if let Some(JsonRpcId::Number(id)) = &response.id {
                        let mut pending = pending_clone.lock().await;
                        if let Some(tx) = pending.remove(&id.to_string()) {
                            let _ = tx.send(response).await;
                        }
                    }
                }
            }
        });

        Ok(Self {
            transport: Transport::Stdio {
                server_process: Arc::new(Mutex::new(child)),
                request_tx,
                pending,
            },
        })
    }

    /// connect to mcp server via sse
    pub async fn connect_sse(
        url: &str,
        api_key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        Ok(Self { transport: Transport::Http { client, url: url.to_string(), api_key } })
    }

    pub async fn send_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(JsonRpcId::String(id.clone())),
        };

        match &self.transport {
            Transport::Stdio { request_tx, pending, .. } => {
                let (response_tx, mut response_rx) = mpsc::channel(1);
                {
                    let mut p = pending.lock().await;
                    p.insert(id.clone(), response_tx);
                }
                request_tx.send(request).await?;
                let response = response_rx.recv().await.ok_or("Connection closed")?;
                if let Some(error) = response.error {
                    return Err(format!("RPC Error {}: {}", error.code, error.message).into());
                }
                response.result.ok_or("No result in response".into())
            }
            Transport::Http { client, url, api_key } => {
                let mut req = client.post(url).json(&request);
                if let Some(key) = api_key {
                    req = req.bearer_auth(key);
                }
                let resp = req.send().await?;
                let rpc: JsonRpcResponse = resp.json().await?;
                if let Some(error) = rpc.error {
                    return Err(format!("RPC Error {}: {}", error.code, error.message).into());
                }
                rpc.result.ok_or("No result in response".into())
            }
        }
    }

    pub async fn initialize(&self) -> Result<McpInitializeResult, Box<dyn std::error::Error>> {
        let params = McpInitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: McpCapabilities { experimental: None, sampling: None, roots: None },
            client_info: McpClientInfo {
                name: "boomai-daemon".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let result_value =
            self.send_request("initialize", Some(serde_json::to_value(params)?)).await?;
        let result: McpInitializeResult = serde_json::from_value(result_value)?;

        // send initialized notification
        if let Transport::Stdio { request_tx, .. } = &self.transport {
            let notification = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "notifications/initialized".to_string(),
                params: None,
                id: None,
            };
            let _ = request_tx.send(notification).await;
        }

        Ok(result)
    }

    pub async fn list_tools(&self) -> Result<McpListToolsResult, Box<dyn std::error::Error>> {
        let result_value = self.send_request("tools/list", None).await?;
        let result: McpListToolsResult = serde_json::from_value(result_value)?;
        Ok(result)
    }

    /// Explicit, awaited shutdown for transports that need cleanup.
    #[allow(dead_code)]
    pub async fn shutdown(&self) {
        if let Transport::Stdio { server_process, .. } = &self.transport {
            let mut child = server_process.lock().await;
            let _ = child.kill().await;
        }
        // HTTP/SSE transport has no explicit shutdown hook today.
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Transport::Stdio { server_process, .. } = &self.transport {
            let server = server_process.clone();
            tokio::spawn(async move {
                let mut child = server.lock().await;
                let _ = child.kill().await;
            });
        }
    }
}
