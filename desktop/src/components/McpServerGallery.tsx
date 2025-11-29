import { useState, useEffect } from "react";
import { api, McpTool } from "../lib/api";

interface Props {
  onBack: () => void;
}

export function McpServerGallery({ onBack }: Props) {
  const [servers, setServers] = useState<string[]>([]);
  const [newServerId, setNewServerId] = useState("");
  const [newServerCommand, setNewServerCommand] = useState("npx");
  const [newServerArgs, setNewServerArgs] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [selectedServer, setSelectedServer] = useState<string | null>(null);
  const [tools, setTools] = useState<McpTool[]>([]);

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    try {
      const data = await api.config.mcp.listServers();
      setServers(data.servers || []);
    } catch (e) {
      console.error("Failed to load MCP servers", e);
    }
  };

  const handleAddServer = async () => {
    if (!newServerId || !newServerCommand) return;
    setStatus(`Connecting to ${newServerId}...`);
    
    try {
      // Parse args string into array (simple space splitting for MVP)
      const args = newServerArgs.split(" ").filter(a => a.length > 0);
      
      const res = await api.config.mcp.addServer(newServerId, newServerCommand, args);
      if (res.status === "success") {
        setStatus("Server connected!");
        setNewServerId("");
        setNewServerArgs("");
        await loadServers();
      } else {
        setStatus(`Error: ${res.message}`);
      }
    } catch (e) {
      setStatus(`Network Error: ${e}`);
    }
  };

  const handleViewTools = async (serverId: string) => {
    setSelectedServer(serverId);
    setTools([]);
    try {
      const data = await api.config.mcp.listTools(serverId);
      setTools(data.tools || []);
    } catch (e) {
      console.error("Failed to list tools", e);
    }
  };

  return (
    <>
      <div className="card">
        <h3>MCP Server Management</h3>
        
        {/* Add New Server Form */}
        <div className="section" style={{ background: "#f5f5f5", padding: "1rem", borderRadius: "8px" }}>
          <h4>Connect New Server</h4>
          <div className="form-group">
            <label>Server ID</label>
            <input 
              placeholder="e.g., filesystem" 
              value={newServerId}
              onChange={(e) => setNewServerId(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Command</label>
            <input 
              placeholder="npx" 
              value={newServerCommand}
              onChange={(e) => setNewServerCommand(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Arguments</label>
            <input 
              placeholder="-y @modelcontextprotocol/server-filesystem /path/to/files" 
              value={newServerArgs}
              onChange={(e) => setNewServerArgs(e.target.value)}
            />
          </div>
          <button onClick={handleAddServer}>Connect Server</button>
          {status && <p className="status-message">{status}</p>}
        </div>

        {/* List Servers */}
        <div className="section">
          <h4>Connected Servers</h4>
          {servers.length === 0 ? (
            <p>No MCP servers connected</p>
          ) : (
            servers.map(server => (
              <div key={server} className="model-item installed">
                <div>
                  <strong>{server}</strong>
                  <span className="status running">Active</span>
                </div>
                <button onClick={() => handleViewTools(server)}>View Tools</button>
              </div>
            ))
          )}
        </div>

        {/* Tool Browser */}
        {selectedServer && (
          <div className="section">
            <h4>Tools from {selectedServer}</h4>
            {tools.length === 0 ? (
              <p>No tools found or loading...</p>
            ) : (
              <div className="tool-list">
                {tools.map((tool, i) => (
                  <div key={i} className="model-item">
                    <div>
                      <strong>{tool.name}</strong>
                      <p>{tool.description || "No description"}</p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="actions">
        <button onClick={onBack}>Back</button>
      </div>
    </>
  );
}



