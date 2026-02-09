import { useState, useEffect, useCallback } from 'react';
import type { TutorialStep } from './content/types';
import NavBar from './components/NavBar';
import Sidebar from './components/Sidebar';
import Catalog from './components/Catalog';
import TutorialPage from './components/TutorialPage';
import './App.css';

function parseHash(): { tutorialId: string | null; step: number } {
  const hash = window.location.hash.slice(1); // remove '#'
  if (!hash) return { tutorialId: null, step: 0 };
  const parts = hash.split('/');
  const tutorialId = parts[0] || null;
  const step = parts[1] ? parseInt(parts[1], 10) : 0;
  return { tutorialId, step: isNaN(step) ? 0 : step };
}

export default function App() {
  const [tutorialId, setTutorialId] = useState<string | null>(null);
  const [step, setStep] = useState(0);
  const [steps, setSteps] = useState<TutorialStep[]>([]);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  // Parse hash on mount and on hashchange
  useEffect(() => {
    const update = () => {
      const parsed = parseHash();
      setTutorialId(parsed.tutorialId);
      setStep(parsed.step);
    };
    update();
    window.addEventListener('hashchange', update);
    return () => window.removeEventListener('hashchange', update);
  }, []);

  const navigate = useCallback((id: string | null, s?: number) => {
    if (id === null) {
      window.location.hash = '';
      setSteps([]);
    } else {
      window.location.hash = `${id}/${s ?? 0}`;
    }
    setSidebarOpen(false);
  }, []);

  const handleStepsLoaded = useCallback((loadedSteps: TutorialStep[]) => {
    setSteps(loadedSteps);
  }, []);

  return (
    <div className="app-layout">
      <NavBar onMenuToggle={() => setSidebarOpen((o) => !o)} />
      <Sidebar
        currentTutorialId={tutorialId}
        currentStep={step}
        steps={steps}
        onNavigate={navigate}
        isOpen={sidebarOpen}
        onToggle={() => setSidebarOpen((o) => !o)}
      />
      <main className="app-main">
        {tutorialId ? (
          <TutorialPage
            key={tutorialId}
            tutorialId={tutorialId}
            step={step}
            onNavigate={navigate}
            onStepsLoaded={handleStepsLoaded}
          />
        ) : (
          <Catalog onNavigate={(id) => navigate(id, 0)} />
        )}
      </main>
    </div>
  );
}
