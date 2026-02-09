import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import type { TutorialStep } from '../content/types';
import CodeBlock from './CodeBlock';
import InteractiveStep from './InteractiveStep';
import PlaygroundEmbed from './PlaygroundEmbed';
import LocalSection from './LocalSection';
import AlgorithmStepper from './AlgorithmStepper';

interface StepContentProps {
  step: TutorialStep;
  tutorialId: string;
}

export default function StepContent({ step, tutorialId }: StepContentProps) {
  const isInteractive = step.code && step.codeLanguage === 'c' && step.graphType;
  const isAlgorithm = step.stepType === 'algorithm' && step.algorithmTrace;

  return (
    <div className="step-content step-content-single">
      <div className="step-left">
        <h2>{step.title}</h2>

        <div className="step-prose">
          <Markdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
            {step.content}
          </Markdown>
        </div>

        {isAlgorithm ? (
          <AlgorithmStepper tutorialId={tutorialId} traceFile={step.algorithmTrace!} />
        ) : isInteractive ? (
          <InteractiveStep step={step} tutorialId={tutorialId} />
        ) : (
          <>
            {step.code && step.codeLanguage && (
              <CodeBlock
                code={step.code}
                language={step.codeLanguage}
                highlightLines={step.highlightLines}
              />
            )}
          </>
        )}

        {step.challenge && (
          <div className="challenge-box">
            <div className="challenge-header">Challenge</div>
            <Markdown remarkPlugins={[remarkGfm]}>{step.challenge}</Markdown>
          </div>
        )}

        {step.playground && <PlaygroundEmbed url={step.playground} />}

        {(step.localCmd || step.localScript) && (
          <LocalSection
            cmd={step.localCmd}
            script={step.localScript}
            tutorialId={tutorialId}
          />
        )}
      </div>
    </div>
  );
}
