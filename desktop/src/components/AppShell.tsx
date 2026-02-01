import { useState } from 'react';
import { MessageSquare, FolderOpen, Zap, Settings, Plus, Search, ChevronLeft, ChevronRight } from 'lucide-react';
import type { AppView } from '../App';

interface AppShellProps {
  currentView: AppView;
  onViewChange: (view: AppView) => void;
  children: React.ReactNode;
}

export function AppShell({ currentView, onViewChange, children }: AppShellProps) {
  const [collapsed, setCollapsed] = useState(false);

  const navItems: { id: AppView; label: string; icon: React.ReactNode }[] = [
    { id: 'chat', label: 'Chat', icon: <MessageSquare size={20} /> },
    { id: 'library', label: 'Library', icon: <FolderOpen size={20} /> },
    { id: 'automations', label: 'Automations', icon: <Zap size={20} /> },
  ];

  return (
    <div className="flex min-h-screen bg-paper">
      <aside className={`${collapsed ? 'w-16' : 'w-64'} h-screen bg-deep-space flex flex-col fixed left-0 top-0 transition-all duration-200`}>
        <div className="flex items-center gap-3 p-4 border-b border-deep-space-lighter">
          <img src="/boomai.svg" alt="Boomai" className="w-8 h-8 flex-shrink-0" />
          {!collapsed && (
            <span className="font-display text-lg tracking-wider text-paper uppercase">Boomai</span>
          )}
        </div>

        <div className="p-3">
          <button className={`w-full flex items-center justify-center gap-2 py-3 bg-rocket-red text-white rounded-lg font-semibold text-sm hover:bg-rocket-red-hover transition-colors ${collapsed ? 'px-0' : 'px-4'}`}>
            <Plus size={18} />
            {!collapsed && <span>New Chat</span>}
          </button>
        </div>

        {!collapsed && (
          <div className="px-3 pb-3">
            <div className="flex items-center gap-2 px-3 py-2 bg-deep-space-light rounded-lg">
              <Search size={16} className="text-muted-tan" />
              <input 
                type="text" 
                placeholder="Search..." 
                className="flex-1 bg-transparent text-sm text-paper placeholder:text-muted-tan outline-none"
              />
            </div>
          </div>
        )}

        <nav className="flex-1 p-3 overflow-y-auto">
          {!collapsed && (
            <span className="block text-xs font-semibold uppercase tracking-widest text-muted-tan px-3 mb-2">
              Navigation
            </span>
          )}
          <div className="space-y-1">
            {navItems.map(item => (
              <button
                key={item.id}
                onClick={() => onViewChange(item.id)}
                className={`w-full flex items-center gap-3 px-3 py-3 rounded-lg text-sm transition-colors relative ${
                  currentView === item.id
                    ? 'bg-deep-space-lighter text-paper'
                    : 'text-muted-tan hover:bg-deep-space-lighter hover:text-paper'
                }`}
                title={collapsed ? item.label : undefined}
              >
                {currentView === item.id && (
                  <div className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-rocket-red rounded-r" />
                )}
                {item.icon}
                {!collapsed && <span>{item.label}</span>}
              </button>
            ))}
          </div>
        </nav>

        <div className="p-3 border-t border-deep-space-lighter">
          <button
            onClick={() => onViewChange('settings')}
            className={`w-full flex items-center gap-3 px-3 py-3 rounded-lg text-sm transition-colors ${
              currentView === 'settings'
                ? 'bg-deep-space-lighter text-paper'
                : 'text-muted-tan hover:bg-deep-space-lighter hover:text-paper'
            }`}
            title={collapsed ? 'Settings' : undefined}
          >
            <Settings size={20} />
            {!collapsed && <span>Settings</span>}
          </button>
        </div>

        <button
          onClick={() => setCollapsed(!collapsed)}
          className="absolute -right-3 top-1/2 -translate-y-1/2 w-6 h-6 bg-deep-space border-2 border-deep-space-lighter rounded-full flex items-center justify-center text-paper hover:bg-deep-space-lighter transition-colors z-10"
        >
          {collapsed ? <ChevronRight size={14} /> : <ChevronLeft size={14} />}
        </button>
      </aside>

      <main className={`flex-1 ${collapsed ? 'ml-16' : 'ml-64'} transition-all duration-200`}>
        {children}
      </main>
    </div>
  );
}
