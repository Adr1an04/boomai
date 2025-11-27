import { fetch } from "@tauri-apps/plugin-http";

export interface SystemProfile {
  os_name: string;
  cpu_cores: number;
  total_memory_gb: number;
  architecture: string;
}

export interface Recommendation {
  recommended_engine: "Local" | "Cloud";
  reason: string;
}

export interface ModelConfig {
  base_url: string;
  api_key?: string;
  model: string;
}

export interface AvailableLocalModel {
  id: string;
  name: string;
  description: string;
  size_gb: number;
  recommended_ram_gb: number;
  download_url: string;
  local_port: number;
  runtime_type: string;
}

export interface InstalledLocalModel {
  model_id: string;
  install_path: string;
  is_running: boolean;
  port: number;
  runtime_type: string;
}

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

const API_BASE = "http://localhost:3046"; // Daemon running on port 3046

export const api = {
  system: {
    getProfile: async (): Promise<SystemProfile> => {
      const res = await fetch(`${API_BASE}/system/profile`);
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText || "Failed to fetch system profile"}`);
      }
      return res.json();
    },
    getRecommendation: async (): Promise<Recommendation> => {
      const res = await fetch(`${API_BASE}/system/recommendation`);
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText || "Failed to fetch recommendation"}`);
      }
      return res.json();
    },
  },
  config: {
    model: {
      test: async (config: ModelConfig): Promise<{ status: string; message: string }> => {
        const res = await fetch(`${API_BASE}/config/model/test`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(config),
        });
        return res.json();
      },
      save: async (config: ModelConfig): Promise<{ status: string; message: string }> => {
        const res = await fetch(`${API_BASE}/config/model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(config),
        });
        return res.json();
      },
    },
    local: {
      getAvailable: async (): Promise<{ models: AvailableLocalModel[] }> => {
        const res = await fetch(`${API_BASE}/config/local/available_models`);
        if (!res.ok) {
          throw new Error(`HTTP ${res.status}: ${res.statusText || "Failed to fetch available models"}`);
        }
        return res.json();
      },
      getInstalled: async (): Promise<{ models: InstalledLocalModel[] }> => {
        const res = await fetch(`${API_BASE}/config/local/installed_models`);
        if (!res.ok) {
          throw new Error(`HTTP ${res.status}: ${res.statusText || "Failed to fetch installed models"}`);
        }
        return res.json();
      },
      install: async (modelId: string): Promise<{ status: string; message: string }> => {
        const res = await fetch(`${API_BASE}/config/local/install_model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model_id: modelId }),
        });
        return res.json();
      },
      uninstall: async (modelId: string): Promise<{ status: string; message: string }> => {
        const res = await fetch(`${API_BASE}/config/local/uninstall_model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model_id: modelId }),
        });
        return res.json();
      },
    },
  },
  chat: {
    send: async (messages: ChatMessage[]): Promise<{ message: ChatMessage }> => {
      const res = await fetch(`${API_BASE}/chat`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ messages }),
      });
      
      if (!res.ok) {
        throw new Error(res.statusText);
      }
      
      return res.json();
    },
  },
};

