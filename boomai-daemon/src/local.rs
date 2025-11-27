use boomai_core::{AvailableLocalModel, InstalledLocalModel};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

// this is a temp file to manage local models via ollama
pub fn get_available_models() -> Vec<AvailableLocalModel> {
    vec![
        AvailableLocalModel {
            id: "llama3.2:3b".to_string(),
            name: "Llama 3.2 3B".to_string(),
            description: "Fast, lightweight model for general tasks".to_string(),
            size_gb: 2.0,
            recommended_ram_gb: 8,
            download_url: "ollama:llama3.2:3b".to_string(), // ollama model specific
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: "mistral:7b".to_string(),
            name: "Mistral 7B".to_string(),
            description: "Balanced performance and quality".to_string(),
            size_gb: 4.1,
            recommended_ram_gb: 16,
            download_url: "ollama:mistral:7b".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
        AvailableLocalModel {
            id: "codellama:7b".to_string(),
            name: "Code Llama 7B".to_string(),
            description: "Specialized for code generation and understanding".to_string(),
            size_gb: 3.8,
            recommended_ram_gb: 16,
            download_url: "ollama:codellama:7b".to_string(),
            local_port: 11434,
            runtime_type: "ollama".to_string(),
        },
    ]
}

#[derive(Clone)]
pub struct LocalModelManager {
    installed_models: Arc<Mutex<HashMap<String, InstalledLocalModel>>>,
}

impl LocalModelManager {
    pub fn new() -> Self {
        Self {
            installed_models: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn install_model(&self, model_id: &str) -> Result<(), String> {
        let available_models = get_available_models();
        let model = available_models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model {} not found", model_id))?;

        println!("Installing model: {} ({})", model.name, model_id);

        // use the ollama pull command
        if model.runtime_type == "ollama" {
            let pull_result = Command::new("ollama")
                .args(&["pull", &model_id])
                .output()
                .await
                .map_err(|e| format!("Failed to run ollama pull: {}", e))?;

            if !pull_result.status.success() {
                let stderr = String::from_utf8_lossy(&pull_result.stderr);
                return Err(format!("Ollama pull failed: {}", stderr));
            }

            // add to installed models
            let installed = InstalledLocalModel {
                model_id: model_id.to_string(),
                install_path: format!("ollama:{}", model_id), // ollama manages paths internally
                is_running: false,
                port: model.local_port,
                runtime_type: model.runtime_type.clone(),
            };

            if let Ok(mut models) = self.installed_models.lock() {
                models.insert(model_id.to_string(), installed);
            }

            Ok(())
        } else {
            Err(format!("Unsupported runtime type: {}", model.runtime_type))
        }
    }

    pub async fn uninstall_model(&self, model_id: &str) -> Result<(), String> {
        println!("Uninstalling model: {}", model_id);

        // remove from Ollama
        let remove_result = Command::new("ollama")
            .args(&["rm", model_id])
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

    pub async fn start_model(&self, model_id: &str) -> Result<(), String> {
        println!("Starting model: {}", model_id);

        let _serve_result = Command::new("ollama")
            .args(&["serve"])
            .spawn()
            .map_err(|e| format!("Failed to start ollama serve: {}", e))?;

        // Update running status
        if let Ok(mut models) = self.installed_models.lock() {
            if let Some(model) = models.get_mut(model_id) {
                model.is_running = true;
            }
        }

        Ok(())
    }

    pub async fn stop_model(&self, model_id: &str) -> Result<(), String> {
        println!("Stopping model: {}", model_id);

        // kill ollama process
        let _kill_result = Command::new("pkill")
            .args(&["-f", "ollama"])
            .output()
            .await
            .map_err(|e| format!("Failed to stop ollama: {}", e))?;

        // update running status
        if let Ok(mut models) = self.installed_models.lock() {
            if let Some(model) = models.get_mut(model_id) {
                model.is_running = false;
            }
        }

        Ok(())
    }

    pub fn get_installed_models(&self) -> Vec<InstalledLocalModel> {
        if let Ok(models) = self.installed_models.lock() {
            models.values().cloned().collect()
        } else {
            vec![]
        }
    }
}

