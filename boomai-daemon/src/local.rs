use crate::core::{AvailableLocalModel, InstalledLocalModel, ModelId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::process::Command;

// this is a temp file to manage local models via ollama
pub fn get_available_models() -> Vec<AvailableLocalModel> {
    vec![
        AvailableLocalModel {
            id: ModelId::from("smollm:135m"),
            name: "SmolLM 135M".to_string(),
            description: "Ultra-lightweight model for quick testing (81MB)".to_string(),
            size_gb: 0.08,
            recommended_ram_gb: 2,
            download_url: "ollama:smollm:135m".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: ModelId::from("qwen2:0.5b"),
            name: "Qwen2 0.5B".to_string(),
            description: "Very small but capable model (352MB)".to_string(),
            size_gb: 0.35,
            recommended_ram_gb: 4,
            download_url: "ollama:qwen2:0.5b".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: ModelId::from("tinyllama"),
            name: "TinyLlama 1.1B".to_string(),
            description: "Minimal but functional model for testing".to_string(),
            size_gb: 0.6,
            recommended_ram_gb: 4,
            download_url: "ollama:tinyllama".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: ModelId::from("llama3.2:3b"),
            name: "Llama 3.2 3B".to_string(),
            description: "Fast, lightweight model for general tasks".to_string(),
            size_gb: 2.0,
            recommended_ram_gb: 8,
            download_url: "ollama:llama3.2:3b".to_string(), // ollama model specific
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: ModelId::from("qwen3:32b"),
            name: "Qwen3 32B".to_string(),
            description: "Advanced reasoning model with thinking/non-thinking modes. Superior performance on math, coding, and complex tasks".to_string(),
            size_gb: 65.6, // 32.8B params * ~2 bytes/param (BF16)
            recommended_ram_gb: 64, // Substantial RAM requirement for 32B model
            download_url: "ollama:qwen3:32b".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: ModelId::from("gpt-oss:20b"),
            name: "GPT-OSS 20B".to_string(),
            description: "OpenAI's open-weight model with configurable reasoning levels. Excellent for agentic tasks, chain-of-thought reasoning, and complex problem solving".to_string(),
            size_gb: 44.0,
            recommended_ram_gb: 16,
            download_url: "ollama:gpt-oss:20b".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
    ]
}

#[derive(Clone)]
pub struct LocalModelManager {
    installed_models: Arc<Mutex<HashMap<ModelId, InstalledLocalModel>>>,
}

impl LocalModelManager {
    pub fn new() -> Self {
        Self { installed_models: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Refresh installed_models from Ollama so models persist across daemon restarts.
    pub async fn sync_with_ollama(&self) -> Result<(), String> {
        let output = Command::new("ollama")
            .args(["list"])
            .output()
            .await
            .map_err(|e| format!("Failed to run ollama list: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ollama list failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut discovered: HashSet<ModelId> = HashSet::new();
        for line in stdout.lines() {
            if let Some(model_id) = line.split_whitespace().next() {
                discovered.insert(ModelId::from(model_id));
            }
        }

        let available_models = get_available_models();
        let mut models =
            self.installed_models.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        models.clear();

        for model_id in discovered {
            if let Some(avail) = available_models.iter().find(|m| m.id == model_id) {
                let installed = InstalledLocalModel {
                    model_id: model_id.clone(),
                    install_path: format!("ollama:{}", model_id.as_str()),
                    is_running: false,
                    port: avail.local_port,
                    runtime_type: avail.runtime_type.clone(),
                };
                models.insert(model_id, installed);
            }
        }

        Ok(())
    }

    pub async fn install_model(&self, model_id: &ModelId) -> Result<(), String> {
        let available_models = get_available_models();
        let model = available_models
            .iter()
            .find(|m| m.id == *model_id)
            .ok_or_else(|| format!("Model {} not found", model_id.as_str()))?;

        println!("Installing model: {} ({})", model.name, model_id.as_str());

        // use the ollama pull command
        if model.runtime_type == "ollama" {
            let pull_result = Command::new("ollama")
                .args(["pull", model_id.as_str()])
                .output()
                .await
                .map_err(|e| format!("Failed to run ollama pull: {}", e))?;

            if !pull_result.status.success() {
                let stderr = String::from_utf8_lossy(&pull_result.stderr);
                return Err(format!("Ollama pull failed: {}", stderr));
            }

            // add to installed models
            let installed = InstalledLocalModel {
                model_id: model_id.clone(),
                install_path: format!("ollama:{}", model_id.as_str()), // ollama manages paths internally
                is_running: false,
                port: model.local_port,
                runtime_type: model.runtime_type.clone(),
            };

            if let Ok(mut models) = self.installed_models.lock() {
                models.insert(model_id.clone(), installed);
            }

            Ok(())
        } else {
            Err(format!("Unsupported runtime type: {}", model.runtime_type))
        }
    }

    pub async fn uninstall_model(&self, model_id: &ModelId) -> Result<(), String> {
        println!("Uninstalling model: {}", model_id.as_str());

        // remove from Ollama
        let remove_result = Command::new("ollama")
            .args(["rm", model_id.as_str()])
            .output()
            .await
            .map_err(|e| format!("Failed to run ollama rm: {}", e))?;

        if !remove_result.status.success() {
            let stderr = String::from_utf8_lossy(&remove_result.stderr);
            return Err(format!("Ollama remove failed: {}", stderr));
        }

        // delete from installed models
        if let Ok(mut models) = self.installed_models.lock() {
            models.remove(model_id);
        }

        Ok(())
    }

    // Runtime lifecycle controls can be added when integrating with model runtimes.

    pub fn get_installed_models(&self) -> Vec<InstalledLocalModel> {
        if let Ok(models) = self.installed_models.lock() {
            models.values().cloned().collect()
        } else {
            vec![]
        }
    }
}
