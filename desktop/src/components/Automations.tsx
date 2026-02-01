import { Plus } from 'lucide-react';

export function Automations() {
  return (
    <div className="flex flex-col h-screen">
      <header className="p-6 border-b border-paper-dark">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="font-display text-3xl tracking-wide uppercase">Automations</h1>
            <p className="text-sm text-ink-gray-light">Manage agent tasks and scheduled jobs</p>
          </div>
          <button className="btn btn-primary">
            <Plus size={16} /> New Automation
          </button>
        </div>
      </header>

      <div className="flex gap-4 p-6 bg-paper-light border-b border-paper-dark">
        <div className="flex flex-col items-center px-5 py-3 bg-white rounded-lg min-w-[100px]">
          <span className="font-display text-2xl">0</span>
          <span className="text-xs uppercase tracking-wide text-ink-gray-light">Active</span>
        </div>
        <div className="flex flex-col items-center px-5 py-3 bg-white rounded-lg min-w-[100px]">
          <span className="font-display text-2xl">0</span>
          <span className="text-xs uppercase tracking-wide text-ink-gray-light">Scheduled</span>
        </div>
        <div className="flex flex-col items-center px-5 py-3 bg-white rounded-lg min-w-[100px]">
          <span className="font-display text-2xl">0</span>
          <span className="text-xs uppercase tracking-wide text-ink-gray-light">Completed</span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="flex flex-col items-center justify-center min-h-[50vh] text-center">
          <div className="relative w-40 h-40 mb-6">
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-20 h-20 rounded-full bg-gradient-to-br from-muted-tan to-paper-dark" />
            <div className="absolute top-5 left-5 w-[120px] h-[120px] border-[3px] border-rocket-red rounded-full opacity-60" style={{ clipPath: 'polygon(0 0, 100% 0, 100% 50%, 0 50%)' }} />
            <div className="absolute top-2 right-8 w-4 h-4 bg-rocket-red animate-twinkle" style={{ clipPath: 'polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%)' }} />
            <div className="absolute bottom-5 left-5 w-3 h-3 bg-rocket-red animate-twinkle" style={{ clipPath: 'polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%)', animationDelay: '0.5s' }} />
          </div>
          <h3 className="font-display text-2xl tracking-wide uppercase mb-2">No Automations Yet</h3>
          <p className="text-ink-gray-light max-w-md mb-6">
            Create automated tasks to run on a schedule or trigger based on events.
          </p>
          <button className="btn btn-primary">
            <Plus size={16} /> Create Your First Automation
          </button>
        </div>
      </div>
    </div>
  );
}
