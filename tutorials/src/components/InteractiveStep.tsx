import { useState, useCallback, useRef, useEffect } from 'react';
import type { PropertyGraph, PTAResult } from '@saf/web-shared/types';
import type { TutorialStep } from '../content/types';
import GraphViewer from './GraphViewer';
import './InteractiveStep.css';

/** Convert PTA results ({ points_to, value_labels, location_labels }) to PropertyGraph. */
function ptaToPropertyGraph(pta: PTAResult): PropertyGraph {
  const nodes: PropertyGraph['nodes'] = [];
  const edges: PropertyGraph['edges'] = [];
  const seenNodes = new Set<string>();

  for (const entry of pta.points_to) {
    if (!seenNodes.has(entry.value)) {
      seenNodes.add(entry.value);
      nodes.push({
        id: entry.value,
        labels: ['Value'],
        properties: { name: pta.value_labels?.[entry.value] ?? entry.value },
      });
    }
    for (const loc of entry.locations) {
      if (!seenNodes.has(loc)) {
        seenNodes.add(loc);
        nodes.push({
          id: loc,
          labels: ['Location'],
          properties: { name: pta.location_labels?.[loc] ?? loc },
        });
      }
      edges.push({
        src: entry.value,
        dst: loc,
        edge_type: 'POINTS_TO',
        properties: {},
      });
    }
  }

  return {
    schema_version: '0.1.0',
    graph_type: 'pta' as PropertyGraph['graph_type'],
    nodes,
    edges,
    metadata: {},
  };
}

interface InteractiveStepProps {
  step: TutorialStep;
  tutorialId: string;
}

// --- localStorage cache helpers ---

interface CachedResult {
  code: string;
  graph: PropertyGraph;
}

function readCache(key: string): CachedResult | null {
  try {
    const raw = localStorage.getItem(key);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore corrupt/missing cache */ }
  return null;
}

function writeCache(key: string, entry: CachedResult) {
  try {
    localStorage.setItem(key, JSON.stringify(entry));
  } catch { /* ignore quota exceeded */ }
}

// --- Component ---

export default function InteractiveStep({ step, tutorialId }: InteractiveStepProps) {
  const [code, setCode] = useState(step.code ?? '');
  const [analyzedCode, setAnalyzedCode] = useState<string | null>(null);
  const [status, setStatus] = useState<'idle' | 'compiling' | 'parsing' | 'converting' | 'analyzing' | 'ready' | 'error'>('idle');
  const [liveGraph, setLiveGraph] = useState<PropertyGraph | null>(null);
  const [highlightedLines, setHighlightedLines] = useState<[number, number] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLPreElement>(null);

  const graphType = step.graphType ?? 'cfg';
  const cacheKey = `saf-tutorial:${tutorialId}:${step.title}:${graphType}`;

  // Load cached result on step mount (no auto-analyze)
  useEffect(() => {
    const defaultCode = step.code ?? '';
    setCode(defaultCode);
    setHighlightedLines(null);
    setError(null);

    const cached = readCache(cacheKey);
    if (cached && cached.code === defaultCode) {
      setLiveGraph(cached.graph);
      setAnalyzedCode(cached.code);
      setStatus('ready');
    } else {
      setLiveGraph(null);
      setAnalyzedCode(null);
      setStatus('idle');
    }
  }, [step, cacheKey]);

  const handleAnalyze = useCallback(async () => {
    setError(null);
    setHighlightedLines(null);
    try {
      setStatus('compiling');
      const { compileToLLVM, initParser, parseLLVMIR, convertToAIR, initWasm, runAnalysis } =
        await import('@saf/web-shared/analysis');

      const { ir, instSourceLines } = await compileToLLVM(code);

      setStatus('parsing');
      await initParser();
      const tree = parseLLVMIR(ir);

      setStatus('converting');
      const { air } = convertToAIR(tree, instSourceLines);

      setStatus('analyzing');
      await initWasm();
      const results = runAnalysis(JSON.stringify(air));

      // Extract the graph for this step's graphType
      const gt = graphType as keyof typeof results;
      const graph = results[gt];
      let pg: PropertyGraph | null = null;
      if (graph && typeof graph === 'object') {
        if ('nodes' in graph) {
          pg = graph as PropertyGraph;
        } else if ('points_to' in graph) {
          pg = ptaToPropertyGraph(graph as PTAResult);
        }
      }
      if (pg) {
        setLiveGraph(pg);
        setAnalyzedCode(code);
        writeCache(cacheKey, { code, graph: pg });
      }
      setStatus('ready');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('error');
    }
  }, [code, graphType, cacheKey]);

  const handleReset = useCallback(() => {
    const defaultCode = step.code ?? '';
    setCode(defaultCode);
    setHighlightedLines(null);
    setError(null);

    // Try to restore cached result for the default code
    const cached = readCache(cacheKey);
    if (cached && cached.code === defaultCode) {
      setLiveGraph(cached.graph);
      setAnalyzedCode(cached.code);
      setStatus('ready');
      return;
    }

    setLiveGraph(null);
    setAnalyzedCode(null);
    setStatus('idle');
  }, [step.code, cacheKey]);

  // Cmd/Ctrl+Enter to analyze
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      handleAnalyze();
    }
  }, [handleAnalyze]);

  // Sync scroll between textarea and highlight layer
  const syncScroll = useCallback(() => {
    if (textareaRef.current && highlightRef.current) {
      highlightRef.current.scrollTop = textareaRef.current.scrollTop;
      highlightRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  }, []);

  const isHighlighted = (lineNum: number) =>
    highlightedLines != null && lineNum >= highlightedLines[0] && lineNum <= highlightedLines[1];

  const isAnalyzing = status === 'compiling' || status === 'parsing' || status === 'converting' || status === 'analyzing';
  const codeStale = liveGraph != null && analyzedCode != null && code !== analyzedCode;

  const statusLabel = isAnalyzing
    ? { compiling: 'Compiling C to LLVM IR...', parsing: 'Parsing LLVM IR...', converting: 'Converting to AIR...', analyzing: 'Running analysis...' }[status]
    : status === 'ready' ? 'Analysis complete'
    : status === 'error' ? 'Analysis failed'
    : '';

  return (
    <div className="interactive-step">
      <div className="interactive-editor">
        <div className="interactive-toolbar">
          <button
            className="analyze-btn"
            onClick={handleAnalyze}
            disabled={isAnalyzing}
          >
            {isAnalyzing ? 'Analyzing...' : 'Analyze'}
          </button>
          <button className="reset-btn" onClick={handleReset}>Reset</button>
          <span className={`status-text status-${status}`}>{statusLabel}</span>
        </div>
        <div className="code-editor-container">
          <pre className="code-highlight-layer" ref={highlightRef}>
            {code.split('\n').map((line, i) => (
              <div
                key={i}
                className={`code-line ${isHighlighted(i + 1) ? 'highlighted' : ''}`}
              >
                <span className="line-number">{i + 1}</span>
                <span className="line-text">{line || ' '}</span>
              </div>
            ))}
          </pre>
          <textarea
            ref={textareaRef}
            className="code-edit-layer"
            value={code}
            onChange={e => setCode(e.target.value)}
            onKeyDown={handleKeyDown}
            onScroll={syncScroll}
            spellCheck={false}
          />
        </div>
        {error && <div className="error-message">{error}</div>}
      </div>
      <div className="interactive-graph">
        {codeStale && (
          <div className="graph-stale-banner">
            Code changed — click <strong>Analyze</strong> to update
          </div>
        )}
        {liveGraph ? (
          <GraphViewer
            graph={liveGraph}
            graphType={graphType}
            onNodeClick={setHighlightedLines}
          />
        ) : !isAnalyzing && (
          <div className="graph-empty-hint">
            <div>
              <div className="graph-empty-icon">&#x25B7;</div>
              <div>Click <strong>Analyze</strong> to generate the graph</div>
              <div className="graph-empty-shortcut">or press <kbd>Cmd</kbd>+<kbd>Enter</kbd></div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
