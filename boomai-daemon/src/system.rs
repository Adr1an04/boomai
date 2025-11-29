use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemProfile {
    pub os_name: String,
    pub os_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub total_memory_gb: u64,
    pub used_memory_gb: u64,
    pub architecture: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineType {
    Local,
    Cloud,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineRecommendation {
    pub recommended_engine: EngineType,
    pub recommended_model: Option<String>,
    pub reason: String,
}

pub fn get_system_profile() -> SystemProfile {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    
    // sysinfo returns bytes, convert to GB
    let total_memory_gb = total_memory / 1024 / 1024 / 1024;
    let used_memory_gb = used_memory / 1024 / 1024 / 1024;

    let cpu = sys.cpus().first();
    let cpu_brand = cpu.map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown".to_string());
    let cpu_cores = sys.cpus().len();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    
    // architecture check
    let architecture = std::env::consts::ARCH.to_string();

    SystemProfile {
        os_name,
        os_version,
        cpu_brand,
        cpu_cores,
        total_memory_gb,
        used_memory_gb,
        architecture,
    }
}

pub fn get_recommendation(profile: &SystemProfile) -> EngineRecommendation {
    // Enhanced recommendation logic based on system capabilities

    let is_apple_silicon = profile.os_name.to_lowercase().contains("macos") && profile.architecture == "aarch64";
    let has_high_end_ram = profile.total_memory_gb >= 64; // For Qwen3-32B
    let has_good_ram = profile.total_memory_gb >= 16;     // For smaller models
    let has_minimal_ram = profile.total_memory_gb >= 8;   // For TinyLlama

    if is_apple_silicon || has_high_end_ram {
        EngineRecommendation {
            recommended_engine: EngineType::Local,
            recommended_model: Some("qwen3:32b".to_string()),
            reason: format!("Your system has excellent specs ({}GB RAM). Qwen3-32B is recommended for superior reasoning, math, and coding performance.", profile.total_memory_gb),
        }
    } else if has_good_ram {
        EngineRecommendation {
            recommended_engine: EngineType::Local,
            recommended_model: Some("gpt-oss:20b".to_string()),
            reason: format!("Your system has good specs ({}GB RAM). GPT-OSS-20B provides excellent reasoning and agentic capabilities.", profile.total_memory_gb),
        }
    } else if has_minimal_ram {
        EngineRecommendation {
            recommended_engine: EngineType::Local,
            recommended_model: Some("tinyllama".to_string()),
            reason: format!("Your system has basic specs ({}GB RAM). TinyLlama works well for testing and simple tasks.", profile.total_memory_gb),
        }
    } else {
        EngineRecommendation {
            recommended_engine: EngineType::Cloud,
            recommended_model: None,
            reason: format!("Your system has limited resources ({}GB RAM). Cloud API is recommended for best performance.", profile.total_memory_gb),
        }
    }
}

