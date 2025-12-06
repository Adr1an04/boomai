import { useState, useEffect } from "react";
import { api, SystemProfile, Recommendation, ModelConfig } from "./lib/api";
import { SystemCheck } from "./components/SystemCheck";
import { ModelGallery } from "./components/ModelGallery";
import { McpServerGallery } from "./components/McpServerGallery";
import { ConfigForm } from "./components/ConfigForm";
import { ChatInterface } from "./components/ChatInterface";
import "./App.css";

function App() {
  // app state
  const [hasConfigured, setHasConfigured] = useState(false);
  const [step, setStep] = useState(1); // 1: Profile, 2: Engine Choice, 3: Local Gallery, 4: MCP Gallery, 5: Config

  // data state
  const [profile, setProfile] = useState<SystemProfile | null>(null);
  const [recommendation, setRecommendation] = useState<Recommendation | null>(null);
  const [systemError, setSystemError] = useState<string | null>(null);
  const [config, setConfig] = useState<ModelConfig>({
    base_url: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    api_key: "",
  });

  // check system profile on mount
  useEffect(() => {
    async function loadSystem() {
      setSystemError(null);
      try {
        const [pData, rData] = await Promise.all([
          api.system.getProfile(),
          api.system.getRecommendation(),
        ]);
        setProfile(pData);
        setRecommendation(rData);
      } catch (e: any) {
        const errorMsg = e?.message || String(e) || "Failed to connect to daemon";
        setSystemError(errorMsg);
        console.error("Failed to load system profile", e);
      }
    }
    loadSystem();
  }, []);

  // nav state
  const nextStep = () => setStep(prev => prev + 1);
  const prevStep = () => setStep(prev => prev - 1);

  // If already configured, go to chat

  if (hasConfigured) {
    return <ChatInterface />;
  }

  return (
    <main className="container onboarding">
      <h1>Welcome to Boomai</h1>
      
      {systemError && (
        <div className="card" style={{ background: "#c78c95ff", border: "2px solid #c62828", color: "#000000" }}>
          <h3 style={{ color: "#000000" }}>Connection Error</h3>
          <p>{systemError}</p>
          <p><strong>Make sure the daemon is running:</strong></p>
          <code style={{ display: "block", padding: "0.5rem", background: "#111111", color: "#ffffff", borderRadius: "4px", marginTop: "0.5rem" }}>
            export BOOMAI_PORT=3030 && cargo run -p boomai-daemon
          </code>
          <button onClick={() => window.location.reload()} style={{ marginTop: "1rem" }}>Retry</button>
        </div>
      )}
      
      {step === 1 && (
        <SystemCheck 
          profile={profile} 
          recommendation={recommendation} 
          onContinue={nextStep} 
        />
      )}

      {step === 2 && (
        <>
          <div className="card">
            <h3>Choose Your AI Engine</h3>
            <div className="row">
              <button 
                type="button" 
                onClick={() => {
                  setConfig({
                    base_url: "https://api.openai.com/v1",
                    model: "gpt-4o-mini",
                    api_key: ""
                  });
                  setStep(5);
                }}
              >
                Cloud API (OpenAI)
              </button>
              <button 
                type="button" 
                onClick={() => {
                  setStep(3);
                }}
              >
                Local Models (Private)
              </button>
            </div>
            <div style={{ marginTop: "1rem", textAlign: "center" }}>
               <button onClick={() => setStep(4)} style={{ background: "#4a4a4a" }}>Manage MCP Tools</button>
            </div>
          </div>

          <div className="actions">
            <button onClick={prevStep}>Back</button>
          </div>
        </>
      )}

      {step === 3 && (
        <ModelGallery 
          onBack={prevStep} 
          onSelectModel={(modelConfig) => {
            setConfig(modelConfig);
            setStep(5); 
          }}
        />
      )}

      {step === 4 && (
        <McpServerGallery onBack={() => setStep(2)} />
      )}

      {step === 5 && (
        <ConfigForm 
          initialConfig={config} 
          onBack={() => setStep(2)}
          onComplete={() => setHasConfigured(true)} 
        />
      )}

    </main>
  );
}

export default App;
