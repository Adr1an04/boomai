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

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    try {
      const [availData, instData] = await Promise.all([
        api.config.local.getAvailable(),
        api.config.local.getInstalled(),
      ]);
      setAvailable(availData.models || []);
      setInstalled(instData.models || []);
    } catch (e) {
      console.error("Failed to load models", e);
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
      model: model.model_id.split(':')[0],
      api_key: "",
    });
  };

  return (
    <>
      <div className="card">
        <h3>Local Model Management</h3>

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

        <div className="section">
          <h4>Available Models</h4>
          {available.map((model) => (
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

        {installStatus && <p className="status-message">{installStatus}</p>}
      </div>

      <div className="actions">
        <button onClick={onBack}>Back</button>
      </div>
    </>
  );
}

