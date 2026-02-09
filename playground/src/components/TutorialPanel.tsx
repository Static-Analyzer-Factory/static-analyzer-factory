import type { Tutorial } from '../tutorials/types';
import './TutorialPanel.css';

interface TutorialPanelProps {
  tutorial: Tutorial;
  currentStep: number;
  onStepChange: (step: number) => void;
  onExit: () => void;
}

export function TutorialPanel({ tutorial, currentStep, onStepChange, onExit }: TutorialPanelProps) {
  const step = tutorial.steps[currentStep];
  const total = tutorial.steps.length;
  const isFirst = currentStep === 0;
  const isLast = currentStep === total - 1;

  return (
    <div className="tutorial-panel">
      <div className="tutorial-header">
        <div className="tutorial-title">{tutorial.title}</div>
        <button className="tutorial-exit" onClick={onExit}>Exit</button>
      </div>
      <div className="tutorial-body">
        <div className="tutorial-step-counter">Step {currentStep + 1} of {total}</div>
        <div className="tutorial-step-title">{step.title}</div>
        <div className="tutorial-step-text">{step.text}</div>
        {step.prompt && (
          <div className="tutorial-prompt">{step.prompt}</div>
        )}
      </div>
      <div className="tutorial-nav">
        <button
          className="btn-primary"
          onClick={() => onStepChange(currentStep - 1)}
          disabled={isFirst}
        >
          Previous
        </button>
        <button
          className="btn-primary"
          onClick={() => onStepChange(currentStep + 1)}
          disabled={isLast}
        >
          Next Step
        </button>
      </div>
    </div>
  );
}
