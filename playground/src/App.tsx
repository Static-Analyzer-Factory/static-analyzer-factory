import { useCallback, useEffect, useRef, useState } from 'react';
import SiteNav from '@saf/web-shared/ui/SiteNav';
import './App.css';
import type { AnalysisResults } from '@saf/web-shared/types';
import type { WasmAnalysisConfig } from '@saf/web-shared/analysis';
import { compileToLLVM, initParser, parseLLVMIR, convertToAIR, initWasm, runAnalysis, loadAllSpecs } from '@saf/web-shared/analysis';
import { SourcePanel } from './components/SourcePanel';
import { CompiledIRPanel } from './components/CompiledIRPanel';
import { AIRPanel } from './components/AIRPanel';
import { GraphPanel } from './components/GraphPanel';
import { SpecViewer } from './components/SpecViewer';
import { ConfigPanel, defaultConfig } from './components/ConfigPanel';
import type { AnalysisConfig } from './components/ConfigPanel';
import { ExamplesMenu } from './components/ExamplesMenu';
import { StatusBar } from './components/StatusBar';
import { examples } from './examples';
import { AnalyzerPanel } from './components/AnalyzerPanel';
import { QueryPanel } from './components/QueryPanel';
import { preloadPyodide } from './analysis/pyodide-bridge';
import { TutorialPanel } from './components/TutorialPanel';
import { loadTutorial } from './tutorials/loader';
import type { Tutorial } from './tutorials/types';

/** Enriched result from handleAnalyze, including AIR JSON for Pyodide reanalysis. */
export interface AnalyzeOutput {
  results: AnalysisResults;
  airJson: string;
  wasmConfig: WasmAnalysisConfig;
}

type InputMode = 'c' | 'llvm';
type Status = 'idle' | 'compiling' | 'parsing' | 'converting' | 'analyzing' | 'ready' | 'error';
type GraphType = 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';
type RightPanel = 'analysis' | 'specs' | 'analyzer' | 'query';

function useUrlParams() {
  const params = new URLSearchParams(window.location.search);
  return {
    embed: params.get('embed') === 'true',
    split: params.get('split') === 'true',
    example: params.get('example'),
    graph: params.get('graph') as GraphType | null,
    tutorial: params.get('tutorial'),
    step: params.has('step') ? parseInt(params.get('step')!, 10) : null,
  };
}

export default function App() {
  const urlParams = useUrlParams();

  const [inputMode, setInputMode] = useState<InputMode>('c');
  const [sourceCode, setSourceCode] = useState(examples[0].source);
  const [compiledIR, setCompiledIR] = useState<string | null>(null);
  const [airJSON, setAirJSON] = useState<object | null>(null);
  const [results, setResults] = useState<AnalysisResults | null>(null);
  const [activeGraph, setActiveGraph] = useState<GraphType>(urlParams.graph || 'cfg');
  const [selectedFunction, setSelectedFunction] = useState<string | null>(null);
  const [status, setStatus] = useState<Status>('idle');
  const [error, setError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);
  const [rightPanel, setRightPanel] = useState<RightPanel>('analysis');
  const [analysisConfig, setAnalysisConfig] = useState<AnalysisConfig>(defaultConfig);
  const [showConfig, setShowConfig] = useState(true);
  const [highlightedLines, setHighlightedLines] = useState<[number, number] | null>(null);
  const [tutorial, setTutorial] = useState<Tutorial | null>(null);
  const [tutorialStep, setTutorialStep] = useState(0);

  const handleAnalyze = useCallback(async (): Promise<AnalyzeOutput | null> => {
    const startTime = performance.now();
    setError(null);
    setResults(null);
    setAirJSON(null);

    try {
      // Step 1: Get LLVM IR (compile from C or use direct input)
      let llvmIR: string;
      let instSourceLines: Record<string, number> = {};

      if (inputMode === 'c') {
        setStatus('compiling');
        const compileResult = await compileToLLVM(sourceCode, analysisConfig.mem2reg);
        llvmIR = compileResult.ir;
        instSourceLines = compileResult.instSourceLines;
        setCompiledIR(llvmIR);
      } else {
        llvmIR = sourceCode;
        setCompiledIR(llvmIR);
      }

      // Step 2: Parse LLVM IR with tree-sitter
      setStatus('parsing');
      await initParser();
      const tree = parseLLVMIR(llvmIR);

      // Step 3: Convert CST to AIR JSON (with source lines baked into spans)
      setStatus('converting');
      const { air } = convertToAIR(tree, instSourceLines);

      setAirJSON(air as unknown as object);

      // Step 4: Build WASM config from UI settings
      const wasmConfig: WasmAnalysisConfig = {
        vf_mode: analysisConfig.vf_mode,
        pta_solver: analysisConfig.pta_solver,
        pta_max_iterations: analysisConfig.pta_max_iterations,
        max_refinement_iters: analysisConfig.max_refinement_iters,
      };

      // Step 4b: Load specs if enabled
      if (analysisConfig.enable_specs) {
        setStatus('analyzing');
        const specs = await loadAllSpecs();
        wasmConfig.spec_yamls = specs.map((s) => s.yaml);
      }

      // Step 5: Run SAF analysis via WASM (or mock)
      setStatus('analyzing');
      await initWasm();
      const airJsonStr = JSON.stringify(air);
      const analysisResults = runAnalysis(airJsonStr, wasmConfig);
      setResults(analysisResults);

      setStatus('ready');
      setElapsed(performance.now() - startTime);
      return { results: analysisResults, airJson: airJsonStr, wasmConfig };
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : String(err));
      setElapsed(performance.now() - startTime);
      return null;
    }
  }, [inputMode, sourceCode, analysisConfig]);

  // Keep a ref to the latest handleAnalyze so setTimeout closures
  // don't capture a stale version with outdated sourceCode.
  const handleAnalyzeRef = useRef(handleAnalyze);
  handleAnalyzeRef.current = handleAnalyze;

  const handleExampleSelect = useCallback(
    (index: number) => {
      setSourceCode(examples[index].source);
      setInputMode('c');
      setCompiledIR(null);
      setAirJSON(null);
      setResults(null);
      setError(null);
      setStatus('idle');
      setSelectedFunction(null);
      setHighlightedLines(null);
    },
    [],
  );

  const handleModeChange = useCallback((mode: InputMode) => {
    setInputMode(mode);
    setCompiledIR(null);
    setAirJSON(null);
    setResults(null);
    setError(null);
    setStatus('idle');
  }, []);

  useEffect(() => {
    if (urlParams.tutorial) {
      loadTutorial(urlParams.tutorial).then(t => {
        setTutorial(t);
        const startStep = urlParams.step ?? 0;
        setTutorialStep(startStep);
        const step = t.steps[startStep];
        if (step?.code) {
          setSourceCode(step.code);
        }
        if (step?.graph) {
          setActiveGraph(step.graph);
        }
        setTimeout(() => handleAnalyzeRef.current(), 100);
      }).catch(console.error);
    } else if (urlParams.example) {
      const idx = examples.findIndex(e => e.slug === urlParams.example);
      if (idx >= 0) {
        handleExampleSelect(idx);
        // Auto-trigger analysis after a tick
        setTimeout(() => handleAnalyzeRef.current(), 100);
      }
    }
  }, []); // mount only

  const handleTutorialStepChange = useCallback((newStep: number) => {
    if (!tutorial) return;
    const step = tutorial.steps[newStep];
    if (!step) return;
    setTutorialStep(newStep);
    if (step.code) {
      setSourceCode(step.code);
      setTimeout(() => handleAnalyzeRef.current(), 100);
    }
    if (step.graph) {
      setActiveGraph(step.graph);
    }
    history.replaceState(null, '', `?tutorial=${tutorial.id}&step=${newStep}`);
  }, [tutorial, handleAnalyze]);

  const handleTutorialExit = useCallback(() => {
    setTutorial(null);
    setTutorialStep(0);
    setSourceCode(examples[0].source);
    setCompiledIR(null);
    setAirJSON(null);
    setResults(null);
    setError(null);
    setStatus('idle');
    setSelectedFunction(null);
    setHighlightedLines(null);
    history.replaceState(null, '', window.location.pathname);
  }, []);

  const isProcessing =
    status === 'compiling' ||
    status === 'parsing' ||
    status === 'converting' ||
    status === 'analyzing';

  const base = import.meta.env.BASE_URL;
  const appClassName = ['app', urlParams.embed && 'embed', urlParams.embed && urlParams.split && 'split'].filter(Boolean).join(' ');

  // Embed mode: graph-only (or source+graph if split)
  if (urlParams.embed) {
    return (
      <div className={appClassName}>
        <div className="panels">
          {urlParams.split && (
            <SourcePanel
              inputMode={inputMode}
              sourceCode={sourceCode}
              onModeChange={handleModeChange}
              onSourceChange={setSourceCode}
              highlightedLines={highlightedLines}
            />
          )}
          <GraphPanel
            activeGraph={activeGraph}
            onGraphChange={setActiveGraph}
            results={results}
            selectedFunction={selectedFunction}
            onFunctionSelect={setSelectedFunction}
            onSourceHighlight={setHighlightedLines}
          />
        </div>
      </div>
    );
  }

  // Normal mode: full UI
  return (
    <div className={appClassName}>
      <SiteNav active="playground" siteRoot={`${base}../`} />
      <header className="header">
        <div className="header-left">
          {!tutorial && (
            <ExamplesMenu
              examples={examples}
              onSelect={handleExampleSelect}
              disabled={isProcessing}
            />
          )}
        </div>
        <div className="header-right">
          {!tutorial && (
            <div className="header-tabs">
              <button
                className={`header-tab ${rightPanel === 'analysis' ? 'active' : ''}`}
                onClick={() => setRightPanel('analysis')}
              >
                Analysis
              </button>
              <button
                className={`header-tab ${rightPanel === 'specs' ? 'active' : ''}`}
                onClick={() => setRightPanel('specs')}
              >
                Specs
              </button>
              <button
                className={`header-tab ${rightPanel === 'analyzer' ? 'active' : ''}`}
                onClick={() => setRightPanel('analyzer')}
                onMouseEnter={preloadPyodide}
              >
                Analyzer
              </button>
              <button
                className={`header-tab ${rightPanel === 'query' ? 'active' : ''}`}
                onClick={() => setRightPanel('query')}
              >
                Query
              </button>
            </div>
          )}
          {!tutorial && (
            <button
              className={`btn-config ${showConfig ? 'active' : ''}`}
              onClick={() => setShowConfig((v) => !v)}
              title="Toggle settings"
            >
              Settings
            </button>
          )}
          <button
            className="btn-primary"
            onClick={handleAnalyze}
            disabled={isProcessing || !sourceCode.trim()}
          >
            {isProcessing ? 'Analyzing...' : 'Analyze'}
          </button>
        </div>
      </header>

      {tutorial && (
        <div className="tutorial-bar">
          <TutorialPanel
            tutorial={tutorial}
            currentStep={tutorialStep}
            onStepChange={handleTutorialStepChange}
            onExit={handleTutorialExit}
          />
        </div>
      )}

      {!tutorial && showConfig && (
        <ConfigPanel
          config={analysisConfig}
          onChange={setAnalysisConfig}
          disabled={isProcessing}
        />
      )}

      {error && <div className="error-banner">{error}</div>}

      <div className="panels">
        <SourcePanel
          inputMode={inputMode}
          sourceCode={sourceCode}
          onModeChange={handleModeChange}
          onSourceChange={setSourceCode}
          highlightedLines={highlightedLines}
        />

        <div className="panel">
          <CompiledIRPanel compiledIR={compiledIR} />
          <AIRPanel airJSON={airJSON} />
        </div>

        {rightPanel === 'analysis' ? (
          <GraphPanel
            activeGraph={activeGraph}
            onGraphChange={setActiveGraph}
            results={results}
            selectedFunction={selectedFunction}
            onFunctionSelect={setSelectedFunction}
            onSourceHighlight={setHighlightedLines}
          />
        ) : rightPanel === 'analyzer' ? (
          <AnalyzerPanel onAnalyze={handleAnalyze} />
        ) : rightPanel === 'query' ? (
          <QueryPanel hasResults={results !== null} />
        ) : (
          <div className="panel">
            <div className="panel-header">
              <h2>Function Specs</h2>
            </div>
            <div className="panel-content">
              <SpecViewer />
            </div>
          </div>
        )}
      </div>

      <StatusBar status={status} elapsed={elapsed} />
    </div>
  );
}
