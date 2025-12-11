use crate::core::ModelConfig;
use anyhow::Result;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{fs, task};
use tracing::warn;

const CONFIG_FILE: &str = "config.json";
const BACKUP_HISTORY_SIZE: usize = 5;
const KEYRING_SERVICE: &str = "boomai-daemon";
const KEYRING_USER: &str = "model_api_key";

/// active configuration and backup history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfigStore {
    pub active_config: ModelConfig,
    pub history: Vec<ModelConfig>, // N valid configurations
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

    pub fn backup_and_update(&mut self, new_config: ModelConfig) {
        // current config history (avoid duplicates)
        if !self.history.iter().any(|c| c == &self.active_config) {
            self.history.push(self.active_config.clone());

            if self.history.len() > BACKUP_HISTORY_SIZE {
                self.history.remove(0);
            }
        }

        self.active_config = new_config;
        self.last_updated = chrono::Utc::now();
    }

    pub fn get_history_config(&self, index: usize) -> Option<&ModelConfig> {
        self.history.get(index)
    }

    pub fn validate_config(&self, config: &ModelConfig) -> Result<()> {
        config.validate()
    }
}

pub fn get_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    let base_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join("Library/Application Support");

    #[cfg(target_os = "linux")]
    let base_dir = dirs::home_dir().expect("Could not find home directory").join(".config");

    #[cfg(target_os = "windows")]
    let base_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .expect("Could not find APPDATA environment variable");

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let base_dir = std::env::current_dir().expect("Could not get current directory");

    base_dir.join("boomai")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join(CONFIG_FILE)
}

pub async fn config_exists() -> bool {
    get_config_path().exists()
}

pub async fn load_config() -> Result<DaemonConfigStore> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // Return default
        let default_config = ModelConfig {
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            api_key: None,
            model: "tinyllama".to_string(),
        };

        return Ok(DaemonConfigStore::new(default_config));
    }

    let content = fs::read_to_string(&config_path).await?;
    let mut store: DaemonConfigStore = serde_json::from_str(&content)?;

    let keyring_api_key = load_api_key().await;
    match keyring_api_key {
        Some(secret) => store.active_config.api_key = Some(secret),
        None => {
            if let Some(api_key) = store.active_config.api_key.clone() {
                if let Err(e) = store_api_key(Some(api_key)).await {
                    warn!("Failed to migrate API key to keyring: {}", e);
                }
            } else {
                store.active_config.api_key = None;
            }
        }
    }

    Ok(store)
}
pub async fn save_config(store: &DaemonConfigStore) -> Result<()> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    if let Err(e) = store_api_key(store.active_config.api_key.clone()).await {
        warn!("Failed to persist API key to keyring: {}", e);
    }

    let mut sanitized = store.clone();
    sanitized.active_config.api_key = None;
    for cfg in sanitized.history.iter_mut() {
        cfg.api_key = None;
    }

    let content = serde_json::to_string_pretty(&sanitized)?;
    fs::write(&config_path, content).await?;

    Ok(())
}

pub async fn update_config(
    current_store: &mut DaemonConfigStore,
    new_config: ModelConfig,
) -> Result<()> {
    current_store.validate_config(&new_config)?;
    current_store.backup_and_update(new_config);
    save_config(current_store).await?;

    Ok(())
}

async fn load_api_key() -> Option<String> {
    task::spawn_blocking(|| {
        Entry::new(KEYRING_SERVICE, KEYRING_USER).ok().and_then(|e| e.get_password().ok())
    })
    .await
    .ok()
    .flatten()
}

async fn store_api_key(secret: Option<String>) -> Result<()> {
    task::spawn_blocking(move || -> Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
        match secret {
            Some(secret) => entry.set_password(&secret)?,
            None => {
                let _ = entry.delete_password();
            }
        }
        Ok(())
    })
    .await??;
    Ok(())
}
