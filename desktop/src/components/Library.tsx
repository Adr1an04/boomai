import { useState } from 'react';
import { Upload, Plus, Search } from 'lucide-react';

export function Library() {
  const [searchQuery, setSearchQuery] = useState('');

  return (
    <div className="flex flex-col h-screen">
      <header className="p-6 border-b border-paper-dark">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="font-display text-3xl tracking-wide uppercase">Library</h1>
            <p className="text-sm text-ink-gray-light">Manage your documents and indexed sources</p>
          </div>
          <div className="flex gap-3">
            <button className="btn btn-ghost">
              <Upload size={16} /> Import
            </button>
            <button className="btn btn-primary">
              <Plus size={16} /> Add Source
            </button>
          </div>
        </div>
      </header>

      <div className="p-6 pb-0">
        <div className="flex items-center gap-3 px-4 py-3 bg-white border border-paper-dark rounded-lg max-w-md">
          <Search size={18} className="text-muted-tan" />
          <input
            type="text"
            placeholder="Search files..."
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-tan"
          />
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
          <h3 className="font-display text-2xl tracking-wide uppercase mb-2">No Files Yet</h3>
          <p className="text-ink-gray-light max-w-md mb-6">
            Add documents to your library to use them as context in conversations.
          </p>
          <button className="btn btn-primary">
            <Plus size={16} /> Add Your First Source
          </button>
        </div>
      </div>
    </div>
  );
}
