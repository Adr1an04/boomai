use std::path::PathBuf;
use boomai_core::ModelConfig;
use serde::{Deserialize, Serialize};
use tokio::fs;
use anyhow::Result;

const CONFIG_FILE: &str = "config.json";
const BACKUP_HISTORY_SIZE: usize = 5;

/// Stores the active configuration and backup history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfigStore {
    pub active_config: ModelConfig,
    pub history: Vec<ModelConfig>, // Last N valid configurations
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl DaemonConfigStore {
    pub fn new(initial_config: ModelConfig) -> Self {
        Self {
            active_config: initial_config.clone(),
            history: vec![initial_config],
            last_updated: chrono::Utc::now(),
        }
    }
    
    /// Add current config to history before updating
    pub fn backup_and_update(&mut self, new_config: ModelConfig) {
        // Add current config to history (avoid duplicates)
        if !self.history.iter().any(|c| c == &self.active_config) {
            self.history.push(self.active_config.clone());
            
            // Keep only last N configs
            if self.history.len() > BACKUP_HISTORY_SIZE {
                self.history.remove(0);
            }
        }
        
        self.active_config = new_config;
        self.last_updated = chrono::Utc::now();
    }
    
    /// Get config at specific index from history
    pub fn get_history_config(&self, index: usize) -> Option<&ModelConfig> {
        self.history.get(index)
    }
    
    /// Validate configuration before applying
    pub fn validate_config(&self, config: &ModelConfig) -> Result<()> {
        config.validate()
    }
}

/// Get platform-specific config directory
pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    let base_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join("Library/Application Support");
    
    #[cfg(target_os = "linux")]
    let base_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config");
    
    #[cfg(target_os = "windows")]
    let base_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .expect("Could not find APPDATA environment variable");
    
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let base_dir = std::env::current_dir()
        .expect("Could not get current directory");
    
    base_dir.join("boomai")
}

/// Get full path to config file
pub fn get_config_path() -> PathBuf {
    get_config_dir().join(CONFIG_FILE)
}

/// Check if config file exists
pub async fn config_exists() -> bool {
    get_config_path().exists()
}

/// Load configuration from disk
pub async fn load_config() -> Result<DaemonConfigStore> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        // Return default config if no file exists
        let default_config = ModelConfig {
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            api_key: None,
            model: "tinyllama".to_string(),
        };
        
        return Ok(DaemonConfigStore::new(default_config));
    }
    
    let content = fs::read_to_string(&config_path).await?;
    let store: DaemonConfigStore = serde_json::from_str(&content)?;
    
    Ok(store)
}

/// Save configuration to disk
pub async fn save_config(store: &DaemonConfigStore) -> Result<()> {
    let config_path = get_config_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    let content = serde_json::to_string_pretty(store)?;
    fs::write(&config_path, content).await?;
    
    Ok(())
}

/// Atomic config update with backup
pub async fn update_config(
    current_store: &mut DaemonConfigStore,
    new_config: ModelConfig,
) -> Result<()> {
    // Validate new config
    current_store.validate_config(&new_config)?;
    
    // Backup current and update
    current_store.backup_and_update(new_config);
    
    // Persist to disk
    save_config(current_store).await?;
    
    Ok(())
}

