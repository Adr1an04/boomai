use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::client::McpClient;

#[derive(Clone)]
pub struct McpManager {
    clients: Arc<RwLock<HashMap<String, Arc<McpClient>>>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self { clients: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn add_stdio_client(
        &self,
        id: String,
        command: &str,
        args: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = McpClient::connect_stdio(command, args).await?;
        client.initialize().await?;

        let mut clients = self.clients.write().await;
        clients.insert(id, Arc::new(client));
        Ok(())
    }

    pub async fn add_sse_client(
        &self,
        id: String,
        url: &str,
        api_key: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = McpClient::connect_sse(url, api_key).await?;
        client.initialize().await?;

        let mut clients = self.clients.write().await;
        clients.insert(id, Arc::new(client));
        Ok(())
    }

    pub async fn get_client(&self, id: &str) -> Option<Arc<McpClient>> {
        let clients = self.clients.read().await;
        clients.get(id).cloned()
    }

    pub async fn list_clients(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
}
