import { useState } from "react";
import { Splash } from "./components/Splash";
import { Onboarding } from "./components/Onboarding";
import { AppShell } from "./components/AppShell";
import { Chat } from "./components/Chat";
import { Library } from "./components/Library";
import { Automations } from "./components/Automations";
import { Settings } from "./components/Settings";

export type AppView = 'chat' | 'library' | 'automations' | 'settings';
type AppState = 'splash' | 'onboarding' | 'main';

function App() {
  const [appState, setAppState] = useState<AppState>('splash');
  const [currentView, setCurrentView] = useState<AppView>('chat');

  const handleSplashComplete = () => {
    setAppState('onboarding');
  };

  const handleOnboardingComplete = () => {
    setAppState('main');
  };

  const renderView = () => {
    switch (currentView) {
      case 'chat':
        return <Chat />;
      case 'library':
        return <Library />;
      case 'automations':
        return <Automations />;
      case 'settings':
        return <Settings />;
      default:
        return <Chat />;
    }
  };

  return (
    <>
      {appState === 'splash' && (
        <Splash onComplete={handleSplashComplete} />
      )}

      {appState === 'onboarding' && (
        <Onboarding onComplete={handleOnboardingComplete} />
      )}

      {appState === 'main' && (
        <AppShell currentView={currentView} onViewChange={setCurrentView}>
          {renderView()}
        </AppShell>
      )}
    </>
  );
}

export default App;
