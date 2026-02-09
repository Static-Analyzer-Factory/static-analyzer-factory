import { useState, useEffect, useCallback } from 'react';
import type { TutorialStep } from '../content/types';
import { TUTORIALS } from '../content/registry';
import { loadTutorialSteps } from '../content/loader';
import StepContent from './StepContent';
import './TutorialPage.css';

interface TutorialPageProps {
  tutorialId: string;
  step: number;
  onNavigate: (id: string | null, step?: number) => void;
  onStepsLoaded: (steps: TutorialStep[]) => void;
}

export default function TutorialPage({
  tutorialId,
  step,
  onNavigate,
  onStepsLoaded,
}: TutorialPageProps) {
  const [steps, setSteps] = useState<TutorialStep[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const meta = TUTORIALS.find((t) => t.id === tutorialId);

  // Load tutorial steps
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setSteps([]);

    loadTutorialSteps(tutorialId)
      .then((loaded) => {
        if (!cancelled) {
          setSteps(loaded);
          onStepsLoaded(loaded);
          setLoading(false);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to load tutorial');
          setLoading(false);
        }
      });

    return () => { cancelled = true; };
  }, [tutorialId, onStepsLoaded]);

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'ArrowLeft' && step > 0) {
        onNavigate(tutorialId, step - 1);
      } else if (e.key === 'ArrowRight' && step < steps.length - 1) {
        onNavigate(tutorialId, step + 1);
      }
    },
    [tutorialId, step, steps.length, onNavigate],
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  if (loading) {
    return (
      <div className="tutorial-page">
        <div className="tutorial-loading">
          <div className="spinner" />
          <span>Loading tutorial...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="tutorial-page">
        <div className="tutorial-error">
          <h2>Error</h2>
          <p>{error}</p>
          <button onClick={() => onNavigate(null)}>Back to Catalog</button>
        </div>
      </div>
    );
  }

  const currentStep = steps[step];
  if (!currentStep) return null;

  const progress = steps.length > 1 ? (step / (steps.length - 1)) * 100 : 100;

  return (
    <div className="tutorial-page">
      <header className="tutorial-header">
        <div className="tutorial-header-left">
          <h1>{meta?.title ?? tutorialId}</h1>
          <span className="step-counter">
            Step {step + 1} of {steps.length}
          </span>
        </div>
        <div className="tutorial-header-right">
          <button
            className="nav-btn"
            disabled={step === 0}
            onClick={() => onNavigate(tutorialId, step - 1)}
          >
            &#8592; Prev
          </button>
          <button
            className="nav-btn"
            disabled={step >= steps.length - 1}
            onClick={() => onNavigate(tutorialId, step + 1)}
          >
            Next &#8594;
          </button>
        </div>
      </header>

      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${progress}%` }} />
      </div>

      <StepContent step={currentStep} tutorialId={tutorialId} />
    </div>
  );
}
