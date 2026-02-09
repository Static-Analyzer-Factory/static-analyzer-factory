import { useState, useEffect, useCallback, useMemo } from 'react';
import type { AlgorithmTrace } from '../content/trace-types';
import type { PropertyGraph } from '@saf/web-shared/types';
import GraphViewer from './GraphViewer';
import ExplanationPanel from './ExplanationPanel';
import StateInspector from './StateInspector';
import StepperControls from './StepperControls';
import PhaseBar from './PhaseBar';
import PseudocodeRail from './PseudocodeRail';
import './AlgorithmStepper.css';

interface AlgorithmStepperProps {
  tutorialId: string;
  traceFile: string;
}

function traceToPropertyGraph(trace: AlgorithmTrace, stepIndex: number): PropertyGraph {
  const step = trace.steps[stepIndex];
  return {
    schema_version: '0.1.0',
    graph_type: 'cfg',
    metadata: { algorithm: trace.algorithm },
    nodes: step.graph.nodes.map(n => ({
      id: n.id,
      labels: [n.type],
      properties: {
        name: n.label,
        ...n.properties,
        highlighted: step.highlights.nodes.includes(n.id),
      },
    })),
    edges: step.graph.edges.map(e => ({
      src: e.src,
      dst: e.dst,
      edge_type: e.type,
      properties: {
        label: e.label,
        highlighted: step.highlights.edges.some(
          h => h.src === e.src && h.dst === e.dst
        ),
      },
    })),
  };
}

export default function AlgorithmStepper({ tutorialId, traceFile }: AlgorithmStepperProps) {
  const [trace, setTrace] = useState<AlgorithmTrace | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [stepIndex, setStepIndex] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);

  const baseUrl = import.meta.env.BASE_URL || '/';

  useEffect(() => {
    const url = `${baseUrl}content/${tutorialId}/${traceFile}`;
    fetch(url)
      .then(res => {
        if (!res.ok) throw new Error(`Failed to load trace: ${res.status}`);
        return res.json();
      })
      .then((data: AlgorithmTrace) => setTrace(data))
      .catch(err => setError(err.message));
  }, [tutorialId, traceFile, baseUrl]);

  const stepForward = useCallback(() => {
    setStepIndex(i => (trace ? Math.min(i + 1, trace.steps.length - 1) : i));
  }, [trace]);
  const stepBack = useCallback(() => {
    setStepIndex(i => Math.max(i - 1, 0));
  }, []);
  const jumpToStart = useCallback(() => setStepIndex(0), []);
  const jumpToEnd = useCallback(() => {
    if (trace) setStepIndex(trace.steps.length - 1);
  }, [trace]);
  const togglePlay = useCallback(() => setIsPlaying(p => !p), []);

  const visitedLines = useMemo(() => {
    if (!trace) return [];
    const visited = new Set<number>();
    for (let i = 0; i < stepIndex; i++) {
      const step = trace.steps[i];
      if (step.activeLines) {
        for (const line of step.activeLines) visited.add(line);
      }
    }
    return Array.from(visited);
  }, [trace, stepIndex]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      switch (e.key) {
        case 'ArrowRight': case 'l': e.preventDefault(); stepForward(); break;
        case 'ArrowLeft': case 'h': e.preventDefault(); stepBack(); break;
        case ' ': e.preventDefault(); togglePlay(); break;
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [stepForward, stepBack, togglePlay]);

  if (error) return <div className="algorithm-stepper algorithm-stepper-error">Error: {error}</div>;
  if (!trace) return <div className="algorithm-stepper algorithm-stepper-loading">Loading trace...</div>;

  const currentStep = trace.steps[stepIndex];
  const graph = traceToPropertyGraph(trace, stepIndex);

  return (
    <div className="algorithm-stepper">
      <div className="stepper-header">
        <div className="stepper-header-top">
          <h3 className="stepper-action">{currentStep.action}</h3>
          <span className="stepper-step-label">Step {stepIndex + 1} / {trace.steps.length}</span>
        </div>
        {trace.phases && (
          <PhaseBar phases={trace.phases} currentPhase={currentStep.phase ?? 0} />
        )}
      </div>
      <div className="stepper-main">
        {trace.pseudocode && (
          <PseudocodeRail
            pseudocode={trace.pseudocode}
            activeLines={currentStep.activeLines ?? []}
            visitedLines={visitedLines}
          />
        )}
        <div className="stepper-graph">
          <GraphViewer graph={graph} graphType="algorithm" />
        </div>
      </div>
      <div className="stepper-bottom">
        <div className="stepper-bottom-left">
          <div className="stepper-section-header">Explanation</div>
          <ExplanationPanel
            stepId={currentStep.id}
            action={currentStep.action}
            explanation={currentStep.explanation}
          />
        </div>
        <div className="stepper-bottom-right">
          <div className="stepper-section-header">Algorithm State</div>
          <StateInspector
            algorithm={trace.algorithm}
            state={currentStep.algorithmState}
            diff={currentStep.diff}
          />
        </div>
      </div>
      <StepperControls
        currentStep={stepIndex}
        totalSteps={trace.steps.length}
        isPlaying={isPlaying}
        speed={speed}
        onStepForward={stepForward}
        onStepBack={stepBack}
        onJumpToStart={jumpToStart}
        onJumpToEnd={jumpToEnd}
        onTogglePlay={togglePlay}
        onSpeedChange={setSpeed}
      />
    </div>
  );
}
