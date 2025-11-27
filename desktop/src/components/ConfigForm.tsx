import { useState } from "react";
import { api, ModelConfig } from "../lib/api";

interface Props {
  initialConfig: ModelConfig;
  onBack: () => void;
  onComplete: () => void;
}

export function ConfigForm({ initialConfig, onBack, onComplete }: Props) {
  const [config, setConfig] = useState<ModelConfig>(initialConfig);
  const [testStatus, setTestStatus] = useState<string | null>(null);

  const handleConfigChange = (field: keyof ModelConfig, value: string) => {
    setConfig((prev) => ({ ...prev, [field]: value }));
  };

  const testConnection = async () => {
    setTestStatus("Testing...");
    try {
      const data = await api.config.model.test(config);
      if (data.status === "success") {
        setTestStatus("Success! Connection verified.");
        return true;
      } else {
        setTestStatus(`Error: ${data.message}`);
        return false;
      }
    } catch (e) {
      setTestStatus(`Network Error: ${e}`);
      return false;
    }
  };

  const saveAndContinue = async () => {
    const ok = await testConnection();
    if (ok) {
      await api.config.model.save(config);
      onComplete();
    }
  };

  return (
    <>
      <div className="card">
        <h3>Configure AI Engine</h3>

        <div className="form-group">
          <label>Base URL</label>
          <input
            value={config.base_url}
            onChange={(e) => handleConfigChange("base_url", e.target.value)}
          />
        </div>
        <div className="form-group">
          <label>Model Name</label>
          <input
            value={config.model}
            onChange={(e) => handleConfigChange("model", e.target.value)}
          />
        </div>
        <div className="form-group">
          <label>API Key (Optional for Local)</label>
          <input
            type="password"
            value={config.api_key}
            onChange={(e) => handleConfigChange("api_key", e.target.value)}
          />
        </div>
      </div>

      <div className="actions">
        <button onClick={onBack}>Back</button>
        <button onClick={testConnection}>Test Connection</button>
        <button onClick={saveAndContinue} disabled={testStatus !== "Success! Connection verified."}>
          Start Chatting
        </button>
      </div>
      {testStatus && <p className="status-message">{testStatus}</p>}
    </>
  );
}

