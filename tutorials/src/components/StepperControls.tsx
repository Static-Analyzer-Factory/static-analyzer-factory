import { useEffect, useRef } from 'react';
import './StepperControls.css';

interface StepperControlsProps {
  currentStep: number;
  totalSteps: number;
  isPlaying: boolean;
  speed: number;
  onStepForward: () => void;
  onStepBack: () => void;
  onJumpToStart: () => void;
  onJumpToEnd: () => void;
  onTogglePlay: () => void;
  onSpeedChange: (speed: number) => void;
}

const SPEEDS = [0.5, 1, 2];

const IconJumpStart = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="1" y="3" width="2" height="10" />
    <polygon points="9,3 9,13 4,8" />
    <polygon points="15,3 15,13 10,8" />
  </svg>
);

const IconStepBack = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="14,3 14,13 4,8" />
    <rect x="2" y="3" width="2" height="10" />
  </svg>
);

const IconPlay = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="4,2 14,8 4,14" />
  </svg>
);

const IconPause = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="3" y="2" width="3.5" height="12" />
    <rect x="9.5" y="2" width="3.5" height="12" />
  </svg>
);

const IconStepForward = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="2,3 2,13 12,8" />
    <rect x="12" y="3" width="2" height="10" />
  </svg>
);

const IconJumpEnd = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="1,3 1,13 6,8" />
    <polygon points="7,3 7,13 12,8" />
    <rect x="13" y="3" width="2" height="10" />
  </svg>
);

export default function StepperControls({
  currentStep,
  totalSteps,
  isPlaying,
  speed,
  onStepForward,
  onStepBack,
  onJumpToStart,
  onJumpToEnd,
  onTogglePlay,
  onSpeedChange,
}: StepperControlsProps) {
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (isPlaying) {
      timerRef.current = setInterval(onStepForward, 1500 / speed);
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [isPlaying, speed, onStepForward]);

  // Stop playing at the last step
  useEffect(() => {
    if (isPlaying && currentStep >= totalSteps - 1) {
      onTogglePlay();
    }
  }, [currentStep, totalSteps, isPlaying, onTogglePlay]);

  const isFirst = currentStep === 0;
  const isLast = currentStep >= totalSteps - 1;

  const cycleSpeed = () => {
    const currentIndex = SPEEDS.indexOf(speed);
    const nextIndex = (currentIndex + 1) % SPEEDS.length;
    onSpeedChange(SPEEDS[nextIndex]);
  };

  return (
    <div className="stepper-controls">
      <div className="stepper-controls-left">
        <button
          className="stepper-btn"
          onClick={onJumpToStart}
          disabled={isFirst}
          title="Jump to start"
          aria-label="Jump to start"
        >
          <IconJumpStart />
        </button>
        <button
          className="stepper-btn"
          onClick={onStepBack}
          disabled={isFirst}
          title="Step back"
          aria-label="Step back"
        >
          <IconStepBack />
        </button>
        <button
          className="stepper-btn stepper-btn-play"
          onClick={onTogglePlay}
          disabled={isLast && !isPlaying}
          title={isPlaying ? 'Pause' : 'Play'}
          aria-label={isPlaying ? 'Pause' : 'Play'}
        >
          {isPlaying ? <IconPause /> : <IconPlay />}
        </button>
        <button
          className="stepper-btn"
          onClick={onStepForward}
          disabled={isLast}
          title="Step forward"
          aria-label="Step forward"
        >
          <IconStepForward />
        </button>
        <button
          className="stepper-btn"
          onClick={onJumpToEnd}
          disabled={isLast}
          title="Jump to end"
          aria-label="Jump to end"
        >
          <IconJumpEnd />
        </button>
      </div>
      <div className="stepper-controls-center">
        <span className="step-counter">
          Step {currentStep + 1} / {totalSteps}
        </span>
      </div>
      <div className="stepper-controls-right">
        <button
          className="stepper-btn stepper-btn-speed"
          onClick={cycleSpeed}
          title={`Speed: ${speed}x (click to cycle)`}
          aria-label={`Speed: ${speed}x`}
        >
          {speed}x
        </button>
      </div>
    </div>
  );
}
