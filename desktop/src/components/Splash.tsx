import { useEffect, useState } from 'react';

interface SplashProps {
  onComplete: () => void;
}

export function Splash({ onComplete }: SplashProps) {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const duration = 2000;
    const startTime = Date.now();
    
    const interval = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const newProgress = Math.min((elapsed / duration) * 100, 100);
      setProgress(newProgress);

      if (newProgress >= 100) {
        clearInterval(interval);
        setTimeout(onComplete, 200);
      }
    }, 16);

    return () => clearInterval(interval);
  }, [onComplete]);

  return (
    <div className="fixed inset-0 flex flex-col items-center justify-center bg-deep-space">
      <div className="mb-8 animate-fade-in">
        <img src="/boomai.svg" alt="Boomai" className="w-24 h-24" />
      </div>

      <h1 className="font-display text-5xl tracking-wider text-paper mb-2">
        BOOMAI
      </h1>
      <p className="text-muted-tan mb-12">Your Local AI Companion</p>

      <div className="w-48">
        <div className="h-1 bg-deep-space-lighter rounded-full overflow-hidden">
          <div 
            className="h-full bg-rocket-red rounded-full transition-all duration-100"
            style={{ width: `${progress}%` }}
          />
        </div>
        <p className="text-xs text-muted-tan text-center mt-3 uppercase tracking-widest">
          Loading...
        </p>
      </div>

      <p className="absolute bottom-6 text-xs text-deep-space-lighter font-mono">
        v0.2.0
      </p>
    </div>
  );
}
