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

export interface McpTool {
  name: string;
  description?: string;
}

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

const API_BASE = "http://localhost:3030"; // Daemon running on port temp

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
    mcp: {
      async listServers(): Promise<{ servers: string[] }> {
        const res = await fetch(`${API_BASE}/config/mcp/servers`);
        if (!res.ok) throw new Error("Failed to list MCP servers");
        return res.json();
      },
      async addServer(id: string, command: string, args: string[]) {
        const res = await fetch(`${API_BASE}/config/mcp/server/add`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id, command, args }),
        });
        if (!res.ok) throw new Error("Failed to add MCP server");
        return res.json();
      },
      async listTools(serverId: string): Promise<{ tools: McpTool[] }> {
        const res = await fetch(`${API_BASE}/config/mcp/tools`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ server_id: serverId }),
        });
        if (!res.ok) throw new Error("Failed to list MCP tools");
        return res.json();
      },
    },
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

