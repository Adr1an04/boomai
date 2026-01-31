import { useState, useEffect } from "react";
import { api, AvailableLocalModel, InstalledLocalModel, ModelConfig } from "../lib/api";

interface Props {
  onBack: () => void;
  onSelectModel: (config: ModelConfig) => void;
}

export function ModelGallery({ onBack, onSelectModel }: Props) {
  const [available, setAvailable] = useState<AvailableLocalModel[]>([]);
  const [installed, setInstalled] = useState<InstalledLocalModel[]>([]);
  const [installStatus, setInstallStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    setLoading(true);
    setError(null);
    try {
      const [availData, instData] = await Promise.all([
        api.config.local.getAvailable(),
        api.config.local.getInstalled(),
      ]);
      setAvailable(availData.models || []);
      setInstalled(instData.models || []);
    } catch (e: any) {
      const errorMsg = e?.message || String(e) || "Failed to connect to daemon. Make sure it's running on port 3030.";
      setError(errorMsg);
      console.error("Failed to load models", e);
    } finally {
      setLoading(false);
    }
  };

  const handleInstall = async (modelId: string) => {
    setInstallStatus(`Installing ${modelId}...`);
    try {
      const res = await api.config.local.install(modelId);
      if (res.status === "success") {
        setInstallStatus("Installation complete!");
        await loadModels();
      } else {
        setInstallStatus(`Error: ${res.message}`);
      }
    } catch (e) {
      setInstallStatus(`Network Error: ${e}`);
    }
  };

  const handleUninstall = async (modelId: string) => {
    try {
      const res = await api.config.local.uninstall(modelId);
      if (res.status === "success") {
        await loadModels();
      } else {
        alert(`Error: ${res.message}`);
      }
    } catch (e) {
      alert(`Network Error: ${e}`);
    }
  };

  const handleSelect = (model: InstalledLocalModel) => {
    onSelectModel({
      base_url: `http://localhost:${model.port}/v1`,
      model: model.model_id, // Use full model name like "smollm:135m"
      api_key: "",
    });
  };

  return (
    <>
      <div className="card">
        <h3>Local Model Management</h3>

        {error && (
          <div className="error-message" style={{ background: "#ffebee", color: "#c62828", padding: "1rem", borderRadius: "8px", marginBottom: "1rem" }}>
            <strong>Connection Error:</strong> {error}
            <br />
            <small>Make sure the daemon is running: <code>export BOOMAI_PORT=3030 && cargo run -p boomai-daemon</code></small>
          </div>
        )}

        {loading && <p>Loading models...</p>}

        {!loading && !error && (
          <>
            {installed.length > 0 && (
              <div className="section">
                <h4>Installed Models</h4>
                {installed.map((model) => (
                  <div key={model.model_id} className="model-item installed">
                    <div>
                      <strong>{model.model_id}</strong>
                      {model.is_running && <span className="status running">Running</span>}
                    </div>
                    <div className="button-group">
                       <button onClick={() => handleSelect(model)}>Use This</button>
                       <button className="danger" onClick={() => handleUninstall(model.model_id)}>Uninstall</button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {available.filter(m => m.runtime_type !== "cloud").length > 0 && (
              <div className="section">
                <h4>Available Local Models</h4>
                {available.filter(m => m.runtime_type !== "cloud").map((model) => (
                  <div key={model.id} className="model-item">
                    <div>
                      <strong>{model.name}</strong>
                      <p>{model.description}</p>
                      <small>Size: {model.size_gb}GB | RAM: {model.recommended_ram_gb}GB</small>
                    </div>
                    <button onClick={() => handleInstall(model.id)}>Install</button>
                  </div>
                ))}
              </div>
            )}

            {available.filter(m => m.runtime_type === "cloud").length > 0 && (
              <div className="section">
                <h4>Cloud API Models</h4>
                {available.filter(m => m.runtime_type === "cloud").map((model) => (
                  <div key={model.id} className="model-item cloud">
                    <div>
                      <strong>{model.name}</strong>
                      <span className="badge cloud">API</span>
                      <p>{model.description}</p>
                    </div>
                    <button onClick={() => onSelectModel({
                      base_url: model.download_url.replace("cloud:", ""),
                      model: model.id,
                      api_key: "",
                    })}>Configure</button>
                  </div>
                ))}
              </div>
            )}
          </>
        )}

        {installStatus && <p className="status-message">{installStatus}</p>}
      </div>

      <div className="actions">
        <button onClick={onBack}>Back</button>
        {error && <button onClick={loadModels}>Retry Connection</button>}
      </div>
    </>
  );
}

