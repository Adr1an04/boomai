import { useState, useEffect } from 'react';
import { api, SystemProfile, Recommendation, ModelConfig, AvailableLocalModel, InstalledLocalModel } from '../lib/api';
import { ChevronRight, ChevronLeft, Cloud, Cpu, Check, AlertCircle, RefreshCw, Key, Download, Trash2 } from 'lucide-react';

type Step = 'welcome' | 'system' | 'engine' | 'models' | 'config';

interface OnboardingProps {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: OnboardingProps) {
  const [step, setStep] = useState<Step>('welcome');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const [profile, setProfile] = useState<SystemProfile | null>(null);
  const [recommendation, setRecommendation] = useState<Recommendation | null>(null);
  
  const [availableModels, setAvailableModels] = useState<AvailableLocalModel[]>([]);
  const [installedModels, setInstalledModels] = useState<InstalledLocalModel[]>([]);
  const [installStatus, setInstallStatus] = useState<string | null>(null);
  
  const [config, setConfig] = useState<ModelConfig>({
    base_url: 'https://api.openai.com/v1',
    model: 'gpt-4o-mini',
    api_key: '',
  });
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const [testMessage, setTestMessage] = useState('');

  useEffect(() => {
    loadSystemProfile();
  }, []);

  const loadSystemProfile = async () => {
    setLoading(true);
    setError(null);
    try {
      const [p, r] = await Promise.all([
        api.system.getProfile(),
        api.system.getRecommendation(),
      ]);
      setProfile(p);
      setRecommendation(r);
    } catch (e: any) {
      setError(e?.message || 'Failed to connect to daemon');
    } finally {
      setLoading(false);
    }
  };

  const loadModels = async () => {
    setLoading(true);
    try {
      const [avail, inst] = await Promise.all([
        api.config.local.getAvailable(),
        api.config.local.getInstalled(),
      ]);
      setAvailableModels(avail.models || []);
      setInstalledModels(inst.models || []);
    } catch (e) {
      console.error('Failed to load models', e);
    } finally {
      setLoading(false);
    }
  };

  const handleInstallModel = async (modelId: string) => {
    setInstallStatus(`Installing ${modelId}...`);
    try {
      const res = await api.config.local.install(modelId);
      if (res.status === 'success') {
        setInstallStatus('Installation complete!');
        await loadModels();
      } else {
        setInstallStatus(`Error: ${res.message}`);
      }
    } catch (e) {
      setInstallStatus(`Error: ${e}`);
    }
  };

  const handleUninstallModel = async (modelId: string) => {
    try {
      await api.config.local.uninstall(modelId);
      await loadModels();
    } catch (e) {
      console.error('Failed to uninstall', e);
    }
  };

  const handleSelectModel = (model: InstalledLocalModel) => {
    setConfig({
      base_url: `http://localhost:${model.port}/v1`,
      model: model.model_id,
      api_key: '',
    });
    setStep('config');
  };

  const handleTestConnection = async () => {
    setTestStatus('testing');
    setTestMessage('Testing...');
    try {
      const data = await api.config.model.test(config);
      if (data.status === 'success') {
        setTestStatus('success');
        setTestMessage('Connection verified!');
      } else {
        setTestStatus('error');
        setTestMessage(data.message || 'Connection failed');
      }
    } catch (e) {
      setTestStatus('error');
      setTestMessage(`Error: ${e}`);
    }
  };

  const handleSaveAndComplete = async () => {
    if (testStatus !== 'success') {
      await handleTestConnection();
      return;
    }
    await api.config.model.save(config);
    onComplete();
  };

  const goToStep = (s: Step) => {
    if (s === 'models') loadModels();
    setStep(s);
  };

  return (
    <div className="min-h-screen bg-paper flex flex-col">
      <header className="flex items-center gap-3 p-5">
        <img src="/boomai.svg" alt="Boomai" className="w-9 h-9 invert" />
        <span className="font-display text-xl tracking-wider uppercase">Boomai</span>
      </header>

      <main className="flex-1 flex justify-center p-6">
        <div className="w-full max-w-xl">
          
          {step === 'welcome' && (
            <div className="text-center animate-fade-in-up">
              <h1 className="font-display text-5xl tracking-wide uppercase mb-4">
                Welcome to Boomai
              </h1>
              <p className="text-lg text-ink-gray-light mb-12 max-w-md mx-auto">
                Your local-first AI companion for productivity and creativity.
              </p>
              
              <div className="space-y-4 mb-12 max-w-sm mx-auto text-left">
                <div className="flex items-start gap-4 p-4 bg-white rounded-lg border border-paper-dark">
                  <div className="w-12 h-12 rounded-lg bg-paper-dark flex items-center justify-center text-rocket-red">
                    <Cpu size={24} />
                  </div>
                  <div>
                    <h4 className="font-semibold">Local-First</h4>
                    <p className="text-sm text-ink-gray-light">Run AI models on your own hardware</p>
                  </div>
                </div>
                <div className="flex items-start gap-4 p-4 bg-white rounded-lg border border-paper-dark">
                  <div className="w-12 h-12 rounded-lg bg-paper-dark flex items-center justify-center text-rocket-red">
                    <Cloud size={24} />
                  </div>
                  <div>
                    <h4 className="font-semibold">Cloud Ready</h4>
                    <p className="text-sm text-ink-gray-light">Connect to OpenAI or other providers</p>
                  </div>
                </div>
              </div>
              
              <button 
                className="btn btn-primary btn-lg"
                onClick={() => goToStep('system')}
              >
                Get Started <ChevronRight size={18} />
              </button>
            </div>
          )}

          {step === 'system' && (
            <div className="animate-fade-in-up">
              <h2 className="font-display text-4xl tracking-wide uppercase text-center mb-2">
                System Check
              </h2>
              <p className="text-center text-ink-gray-light mb-8">
                Let's see what your system can handle.
              </p>

              {error ? (
                <div className="flex items-start gap-4 p-5 bg-error-light border-2 border-error rounded-lg mb-6">
                  <AlertCircle className="text-error flex-shrink-0" size={24} />
                  <div>
                    <h4 className="font-semibold text-error mb-1">Connection Error</h4>
                    <p className="text-sm mb-2">{error}</p>
                    <code className="block text-xs bg-deep-space text-paper p-2 rounded mt-2">
                      export BOOMAI_PORT=3030 && cargo run -p boomai-daemon
                    </code>
                  </div>
                  <button className="btn btn-ghost btn-sm" onClick={loadSystemProfile}>
                    <RefreshCw size={14} /> Retry
                  </button>
                </div>
              ) : loading ? (
                <div className="text-center py-12">
                  <RefreshCw className="animate-spin mx-auto text-rocket-red mb-4" size={32} />
                  <p className="text-ink-gray-light">Scanning your system...</p>
                </div>
              ) : profile && (
                <>
                  <div className="card mb-4">
                    <h3 className="font-display text-xl tracking-wide uppercase mb-4">Your System</h3>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <span className="text-xs uppercase tracking-wide text-ink-gray-light">OS</span>
                        <p className="font-semibold">{profile.os_name}</p>
                      </div>
                      <div>
                        <span className="text-xs uppercase tracking-wide text-ink-gray-light">Memory</span>
                        <p className="font-semibold">{profile.total_memory_gb} GB</p>
                      </div>
                      <div>
                        <span className="text-xs uppercase tracking-wide text-ink-gray-light">CPU Cores</span>
                        <p className="font-semibold">{profile.cpu_cores}</p>
                      </div>
                      <div>
                        <span className="text-xs uppercase tracking-wide text-ink-gray-light">Architecture</span>
                        <p className="font-semibold">{profile.architecture}</p>
                      </div>
                    </div>
                  </div>

                  {recommendation && (
                    <div className="p-4 bg-accent-teal text-white rounded-lg mb-6">
                      <div className="flex items-center gap-2 font-semibold mb-1">
                        <Check size={16} />
                        Recommended: {recommendation.recommended_engine}
                      </div>
                      <p className="text-sm opacity-90">{recommendation.reason}</p>
                    </div>
                  )}
                </>
              )}

              <div className="flex justify-between mt-8">
                <button className="btn btn-ghost" onClick={() => goToStep('welcome')}>
                  <ChevronLeft size={16} /> Back
                </button>
                <button 
                  className="btn btn-primary" 
                  onClick={() => goToStep('engine')}
                  disabled={!profile}
                >
                  Continue <ChevronRight size={16} />
                </button>
              </div>
            </div>
          )}

          {step === 'engine' && (
            <div className="animate-fade-in-up">
              <h2 className="font-display text-4xl tracking-wide uppercase text-center mb-2">
                Choose Your AI Engine
              </h2>
              <p className="text-center text-ink-gray-light mb-8">
                How would you like to run your AI models?
              </p>

              <div className="grid grid-cols-2 gap-4 mb-8">
                <button 
                  className="p-6 bg-white border-2 border-paper-dark rounded-lg text-center hover:border-rocket-red hover:-translate-y-0.5 transition-all"
                  onClick={() => {
                    setConfig({
                      base_url: 'https://api.openai.com/v1',
                      model: 'gpt-4o-mini',
                      api_key: '',
                    });
                    goToStep('config');
                  }}
                >
                  <div className="w-16 h-16 mx-auto mb-4 rounded-lg bg-paper-dark flex items-center justify-center">
                    <Cloud size={32} />
                  </div>
                  <h3 className="font-display text-xl tracking-wide uppercase mb-2">Cloud API</h3>
                  <p className="text-sm text-ink-gray-light mb-3">Connect to OpenAI or compatible APIs</p>
                  <span className="badge badge-default">Recommended</span>
                </button>

                <button 
                  className="p-6 bg-white border-2 border-paper-dark rounded-lg text-center hover:border-rocket-red hover:-translate-y-0.5 transition-all"
                  onClick={() => goToStep('models')}
                >
                  <div className="w-16 h-16 mx-auto mb-4 rounded-lg bg-paper-dark flex items-center justify-center">
                    <Cpu size={32} />
                  </div>
                  <h3 className="font-display text-xl tracking-wide uppercase mb-2">Local Models</h3>
                  <p className="text-sm text-ink-gray-light mb-3">Run models on your own hardware</p>
                  <span className="badge badge-teal">Private</span>
                </button>
              </div>

              <div className="flex justify-between">
                <button className="btn btn-ghost" onClick={() => goToStep('system')}>
                  <ChevronLeft size={16} /> Back
                </button>
              </div>
            </div>
          )}

          {step === 'models' && (
            <div className="animate-fade-in-up">
              <h2 className="font-display text-4xl tracking-wide uppercase text-center mb-2">
                Local Models
              </h2>
              <p className="text-center text-ink-gray-light mb-8">
                Install and manage local AI models.
              </p>

              {loading ? (
                <div className="text-center py-12">
                  <RefreshCw className="animate-spin mx-auto text-rocket-red mb-4" size={32} />
                  <p className="text-ink-gray-light">Loading models...</p>
                </div>
              ) : (
                <>
                  {installedModels.length > 0 && (
                    <div className="mb-6">
                      <h3 className="font-display text-lg tracking-wide uppercase mb-3">Installed</h3>
                      <div className="space-y-2">
                        {installedModels.map(model => (
                          <div key={model.model_id} className="flex items-center justify-between p-4 bg-paper-light border border-accent-teal rounded-lg">
                            <div>
                              <span className="font-semibold">{model.model_id}</span>
                              {model.is_running && <span className="badge badge-success ml-2">Running</span>}
                            </div>
                            <div className="flex gap-2">
                              <button className="btn btn-primary btn-sm" onClick={() => handleSelectModel(model)}>
                                Use This
                              </button>
                              <button className="btn btn-ghost btn-sm" onClick={() => handleUninstallModel(model.model_id)}>
                                <Trash2 size={14} />
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {availableModels.filter(m => m.runtime_type !== 'cloud').length > 0 && (
                    <div className="mb-6">
                      <h3 className="font-display text-lg tracking-wide uppercase mb-3">Available</h3>
                      <div className="space-y-2">
                        {availableModels.filter(m => m.runtime_type !== 'cloud').map(model => (
                          <div key={model.id} className="flex items-center justify-between p-4 bg-white border border-paper-dark rounded-lg">
                            <div>
                              <span className="font-semibold">{model.name}</span>
                              <p className="text-sm text-ink-gray-light">{model.description}</p>
                              <span className="text-xs text-muted-tan">
                                Size: {model.size_gb}GB • RAM: {model.recommended_ram_gb}GB
                              </span>
                            </div>
                            <button className="btn btn-secondary btn-sm" onClick={() => handleInstallModel(model.id)}>
                              <Download size={14} /> Install
                            </button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {installStatus && (
                    <p className="text-center p-3 bg-paper-dark rounded-lg text-sm">{installStatus}</p>
                  )}
                </>
              )}

              <div className="flex justify-between mt-8">
                <button className="btn btn-ghost" onClick={() => goToStep('engine')}>
                  <ChevronLeft size={16} /> Back
                </button>
              </div>
            </div>
          )}

          {step === 'config' && (
            <div className="animate-fade-in-up">
              <h2 className="font-display text-4xl tracking-wide uppercase text-center mb-2">
                Configure Connection
              </h2>
              <p className="text-center text-ink-gray-light mb-8">
                Enter your API details.
              </p>

              <div className="card mb-6">
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Base URL</label>
                    <input
                      className="input"
                      value={config.base_url}
                      onChange={e => setConfig({ ...config, base_url: e.target.value })}
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Model Name</label>
                    <input
                      className="input"
                      value={config.model}
                      onChange={e => setConfig({ ...config, model: e.target.value })}
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2 flex items-center gap-2">
                      <Key size={14} /> API Key
                    </label>
                    <input
                      className="input"
                      type="password"
                      value={config.api_key}
                      onChange={e => setConfig({ ...config, api_key: e.target.value })}
                      placeholder="Optional for local models"
                    />
                  </div>

                  <div className="flex items-center gap-4 pt-4 border-t border-paper-dark">
                    <button 
                      className="btn btn-secondary"
                      onClick={handleTestConnection}
                      disabled={testStatus === 'testing'}
                    >
                      {testStatus === 'testing' ? (
                        <><RefreshCw className="animate-spin" size={14} /> Testing...</>
                      ) : (
                        'Test Connection'
                      )}
                    </button>
                    {testStatus !== 'idle' && (
                      <span className={`flex items-center gap-2 text-sm ${
                        testStatus === 'success' ? 'text-success' : 
                        testStatus === 'error' ? 'text-error' : 'text-ink-gray-light'
                      }`}>
                        {testStatus === 'success' && <Check size={14} />}
                        {testStatus === 'error' && <AlertCircle size={14} />}
                        {testMessage}
                      </span>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex justify-between">
                <button className="btn btn-ghost" onClick={() => goToStep('engine')}>
                  <ChevronLeft size={16} /> Back
                </button>
                <button 
                  className="btn btn-primary"
                  onClick={handleSaveAndComplete}
                  disabled={testStatus === 'testing'}
                >
                  {testStatus === 'success' ? 'Start Chatting' : 'Test & Continue'} <ChevronRight size={16} />
                </button>
              </div>
            </div>
          )}

        </div>
      </main>
    </div>
  );
}
