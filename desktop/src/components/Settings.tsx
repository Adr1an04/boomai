import { useState } from 'react';
import { Cloud, Cpu, Key, Settings as SettingsIcon, Database, User, ExternalLink, Globe, Server } from 'lucide-react';

type Section = 'providers' | 'appearance' | 'shortcuts' | 'data' | 'about';

export function Settings() {
  const [activeSection, setActiveSection] = useState<Section>('providers');
  const [apiKey, setApiKey] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);

  const sections: { id: Section; label: string; icon: React.ReactNode }[] = [
    { id: 'providers', label: 'AI Providers', icon: <Cloud size={18} /> },
    { id: 'appearance', label: 'Appearance', icon: <SettingsIcon size={18} /> },
    { id: 'shortcuts', label: 'Shortcuts', icon: <Key size={18} /> },
    { id: 'data', label: 'Data & Privacy', icon: <Database size={18} /> },
    { id: 'about', label: 'About', icon: <User size={18} /> },
  ];

  const shortcuts = [
    { action: 'New Chat', keys: ['Ctrl', 'N'] },
    { action: 'Search', keys: ['Ctrl', 'K'] },
    { action: 'Settings', keys: ['Ctrl', ','] },
    { action: 'Send Message', keys: ['Enter'] },
    { action: 'New Line', keys: ['Shift', 'Enter'] },
  ];

  return (
    <div className="flex flex-col h-screen">
      <header className="p-6 border-b border-paper-dark">
        <h1 className="font-display text-3xl tracking-wide uppercase">Settings</h1>
      </header>

      <div className="flex flex-1 overflow-hidden">
        <nav className="w-60 p-4 border-r border-paper-dark bg-paper-light flex flex-col gap-1">
          {sections.map(section => (
            <button
              key={section.id}
              onClick={() => setActiveSection(section.id)}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium text-left transition-colors ${
                activeSection === section.id
                  ? 'bg-white text-rocket-red shadow-sm'
                  : 'text-ink-gray-light hover:bg-white hover:text-ink-gray'
              }`}
            >
              {section.icon}
              <span>{section.label}</span>
            </button>
          ))}
        </nav>

        <div className="flex-1 overflow-y-auto p-6">
          <div className="max-w-xl">
            
            {activeSection === 'providers' && (
              <div className="animate-fade-in">
                <h2 className="font-display text-2xl tracking-wide uppercase mb-2">AI Providers</h2>
                <p className="text-sm text-ink-gray-light mb-6">
                  Configure your AI model providers and API keys.
                </p>

                <div className="card border-accent-teal mb-6">
                  <div className="flex items-center gap-4 mb-4 pb-4 border-b border-paper-dark">
                    <div className="w-12 h-12 rounded-lg bg-paper-dark flex items-center justify-center text-accent-teal">
                      <Cloud size={24} />
                    </div>
                    <div className="flex items-center gap-3">
                      <h3 className="text-lg font-semibold">OpenAI</h3>
                      <span className="badge badge-success">Active</span>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium mb-2">API Key</label>
                      <div className="flex gap-2">
                        <input
                          type={showApiKey ? 'text' : 'password'}
                          className="input flex-1"
                          value={apiKey}
                          onChange={e => setApiKey(e.target.value)}
                          placeholder="sk-..."
                        />
                        <button 
                          className="btn btn-ghost btn-sm"
                          onClick={() => setShowApiKey(!showApiKey)}
                        >
                          {showApiKey ? 'Hide' : 'Show'}
                        </button>
                      </div>
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-2">Model</label>
                      <select className="input">
                        <option>gpt-4o-mini</option>
                        <option>gpt-4o</option>
                        <option>gpt-4-turbo</option>
                      </select>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-4 pt-4 border-t border-paper-dark">
                    <button className="btn btn-secondary btn-sm">Test Connection</button>
                    <button className="btn btn-primary btn-sm">Save Changes</button>
                  </div>
                </div>

                <h3 className="font-display text-lg tracking-wide uppercase mb-4">Other Providers</h3>
                <div className="grid grid-cols-3 gap-4">
                  <div className="p-5 bg-white border border-paper-dark rounded-lg text-center hover:border-muted-tan transition-colors">
                    <div className="w-12 h-12 mx-auto mb-3 rounded-lg bg-paper-dark flex items-center justify-center text-muted-tan">
                      <Cpu size={24} />
                    </div>
                    <h4 className="text-sm font-semibold mb-1">Local Models</h4>
                    <p className="text-xs text-ink-gray-light mb-3">Run on your hardware</p>
                    <button className="btn btn-ghost btn-sm">Configure</button>
                  </div>
                  <div className="p-5 bg-white border border-paper-dark rounded-lg text-center hover:border-muted-tan transition-colors">
                    <div className="w-12 h-12 mx-auto mb-3 rounded-lg bg-paper-dark flex items-center justify-center text-muted-tan">
                      <Globe size={24} />
                    </div>
                    <h4 className="text-sm font-semibold mb-1">Anthropic</h4>
                    <p className="text-xs text-ink-gray-light mb-3">Claude models</p>
                    <button className="btn btn-ghost btn-sm">Add</button>
                  </div>
                  <div className="p-5 bg-white border border-paper-dark rounded-lg text-center hover:border-muted-tan transition-colors">
                    <div className="w-12 h-12 mx-auto mb-3 rounded-lg bg-paper-dark flex items-center justify-center text-muted-tan">
                      <Server size={24} />
                    </div>
                    <h4 className="text-sm font-semibold mb-1">Custom</h4>
                    <p className="text-xs text-ink-gray-light mb-3">OpenAI-compatible</p>
                    <button className="btn btn-ghost btn-sm">Add</button>
                  </div>
                </div>
              </div>
            )}

            {activeSection === 'shortcuts' && (
              <div className="animate-fade-in">
                <h2 className="font-display text-2xl tracking-wide uppercase mb-2">Keyboard Shortcuts</h2>
                <p className="text-sm text-ink-gray-light mb-6">
                  Quick actions to boost your productivity.
                </p>

                <div className="card">
                  <div className="space-y-3">
                    {shortcuts.map((shortcut, i) => (
                      <div key={i} className={`flex items-center justify-between py-2 ${i < shortcuts.length - 1 ? 'border-b border-paper-dark pb-3' : ''}`}>
                        <span className="text-sm">{shortcut.action}</span>
                        <div className="flex items-center gap-1 text-xs text-ink-gray-light">
                          {shortcut.keys.map((key, j) => (
                            <span key={j}>
                              <kbd className="px-2 py-1 bg-paper-dark border border-muted-tan rounded text-xs font-mono">{key}</kbd>
                              {j < shortcut.keys.length - 1 && <span className="mx-1">+</span>}
                            </span>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {activeSection === 'about' && (
              <div className="animate-fade-in">
                <h2 className="font-display text-2xl tracking-wide uppercase mb-6">About Boomai</h2>

                <div className="flex flex-col items-center text-center p-8 bg-white border border-paper-dark rounded-lg mb-6">
                  <div className="w-20 h-20 mb-4 rounded-lg bg-rocket-red text-white font-display text-4xl flex items-center justify-center">
                    B
                  </div>
                  <h3 className="font-display text-2xl tracking-wide uppercase">Boomai</h3>
                  <p className="text-sm text-ink-gray-light font-mono">v0.2.0</p>
                  <p className="text-sm text-muted-tan mt-1">Your Local AI Companion</p>
                </div>

                <div className="card">
                  <div className="space-y-2">
                    <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-paper-light transition-colors">
                      <ExternalLink size={16} className="text-muted-tan" />
                      <span>Documentation</span>
                    </a>
                    <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-paper-light transition-colors">
                      <ExternalLink size={16} className="text-muted-tan" />
                      <span>GitHub Repository</span>
                    </a>
                    <a href="#" className="flex items-center gap-3 p-3 rounded-lg hover:bg-paper-light transition-colors">
                      <ExternalLink size={16} className="text-muted-tan" />
                      <span>Report an Issue</span>
                    </a>
                  </div>
                </div>
              </div>
            )}

            {(activeSection === 'appearance' || activeSection === 'data') && (
              <div className="animate-fade-in">
                <h2 className="font-display text-2xl tracking-wide uppercase mb-2">
                  {activeSection === 'appearance' ? 'Appearance' : 'Data & Privacy'}
                </h2>
                <p className="text-sm text-ink-gray-light mb-6">
                  {activeSection === 'appearance' 
                    ? 'Customize how Boomai looks and feels.'
                    : 'Manage your data, exports, and privacy settings.'}
                </p>
                <div className="card text-center py-12">
                  <p className="text-ink-gray-light">Coming soon...</p>
                </div>
              </div>
            )}

          </div>
        </div>
      </div>
    </div>
  );
}
