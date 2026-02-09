/**
 * AnalyzerPanel — Python analyzer editor with Pyodide execution.
 *
 * Provides a CodeMirror Python editor, a Run button, template selector,
 * Pyodide loading status, and an output pane with Console/Findings sub-tabs.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { indentWithTab } from '@codemirror/commands';
import { basicSetup } from 'codemirror';
import { python } from '@codemirror/lang-python';
import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';

import type { AnalyzeOutput } from '../App';
import { safAutocomplete } from '../editor/saf-completions';
import {
  initPyodide,
  isPyodideReady,
  preloadPyodide,
  setupSafBridge,
  runUserScript,
  type Finding,
  type ScriptResult,
} from '../analysis/pyodide-bridge';
import './AnalyzerPanel.css';

// ---------------------------------------------------------------------------
// Script templates
// ---------------------------------------------------------------------------

const TEMPLATES: Record<string, { label: string; code: string }> = {
  list_free: {
    label: 'List all free() calls',
    code: `import saf

result = saf.analyze()
cg = result.callgraph

# Find free() in the call graph
free_fns = [n for n in cg.nodes if n.properties.get("name") == "free"]
if not free_fns:
    print("No free() function found in this program")
else:
    for fn in free_fns:
        kind = fn.properties.get("kind", "?")
        print(f"Found free() (kind={kind}, id={fn.id})")
        callers = cg.predecessors(fn.id)
        for c in callers:
            print(f"  called from {c.properties.get('name', '?')}()")
        if not callers:
            print("  declared but never called")

# Show all external functions
externals = [n for n in cg.nodes if n.properties.get("kind") == "external"]
if externals:
    names = sorted(n.properties.get("name", "?") for n in externals)
    print(f"\\nAll external functions: {', '.join(names)}")

# Show all heap allocations via value flow
vf = result.valueflow
print(f"\\nValue flow: {len(vf.nodes)} nodes, {len(vf.edges)} edges")
edge_types = sorted(set(e.edge_type for e in vf.edges))
print(f"Edge types: {edge_types}")
`,
  },
  explore_cfg: {
    label: 'Explore CFG structure',
    code: `import saf

result = saf.analyze()
cfg = result.cfg

print(f"CFG has {len(cfg.nodes)} nodes and {len(cfg.edges)} edges")

for node in cfg.nodes:
    succs = cfg.successors(node.id)
    if len(succs) > 1:
        name = node.properties.get("name", node.id)
        print(f"Branch point: {name} -> {len(succs)} successors")
`,
  },
  detect_memory_leak: {
    label: 'Detect Memory Leak (CWE-401)',
    code: `import saf

# Use SAF's built-in memory-leak checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("memory-leak")

if not findings:
    print("No memory leaks found")
else:
    print(f"Found {len(findings)} memory leak(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.source_id or "unknown", f.severity or "warning", f.message)
`,
  },
  detect_double_free: {
    label: 'Detect Double Free (CWE-415)',
    code: `import saf

# Use SAF's built-in double-free checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("double-free")

if not findings:
    print("No double-free bugs found")
else:
    print(f"Found {len(findings)} double-free bug(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.sink_id or "unknown", f.severity or "critical", f.message)
`,
  },
  detect_null_deref: {
    label: 'Detect Null Dereference (CWE-476)',
    code: `import saf

# Use SAF's built-in null-deref checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("null-deref")

if not findings:
    print("No null dereferences found")
else:
    print(f"Found {len(findings)} null dereference(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.sink_id or "unknown", f.severity or "error", f.message)
`,
  },
  detect_file_leak: {
    label: 'Detect File Descriptor Leak (CWE-775)',
    code: `import saf

# Use SAF's built-in file-descriptor-leak checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("file-descriptor-leak")

if not findings:
    print("No file descriptor leaks found")
else:
    print(f"Found {len(findings)} file descriptor leak(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.source_id or "unknown", f.severity or "warning", f.message)
`,
  },
  detect_lock_leak: {
    label: 'Detect Lock Not Released (CWE-764)',
    code: `import saf

# Use SAF's built-in lock-not-released checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("lock-not-released")

if not findings:
    print("No unreleased locks found")
else:
    print(f"Found {len(findings)} unreleased lock(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.source_id or "unknown", f.severity or "warning", f.message)
`,
  },
  detect_uninit_use: {
    label: 'Detect Uninitialized Use (CWE-908)',
    code: `import saf

# Use SAF's built-in uninit-use checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("uninit-use")

if not findings:
    print("No uninitialized uses found")
else:
    print(f"Found {len(findings)} uninitialized use(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.sink_id or "unknown", f.severity or "warning", f.message)
`,
  },
  detect_stack_escape: {
    label: 'Detect Stack Escape (CWE-562)',
    code: `import saf

# Use SAF's built-in stack-escape checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("stack-escape")

if not findings:
    print("No stack escapes found")
else:
    print(f"Found {len(findings)} stack escape(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.source_id or "unknown", f.severity or "error", f.message)
`,
  },
  detect_uaf: {
    label: 'Detect Use-After-Free (CWE-416)',
    code: `import saf

# Use SAF's built-in use-after-free checker via the SDK
proj = saf.Project()
q = proj.query()
findings = q.check("use-after-free")

if not findings:
    print("No use-after-free bugs found")
else:
    print(f"Found {len(findings)} use-after-free bug(s):")
    for f in findings:
        print(f"  {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace and f.trace.steps:
            print(f"  Trace: {f.trace.pretty()}")
        saf.report(f.sink_id or "unknown", f.severity or "critical", f.message)
`,
  },
  sdk_taint_flow: {
    label: 'SDK-style Taint Flow Analysis',
    code: `import saf

# SDK-compatible workflow — same API as the native SAF Python SDK
proj = saf.Project()
q = proj.query()

# Define taint sources and sinks using selector factories
# saf.call("gets") matches the return value of gets() calls
# saf.arg_to("printf", 0) matches the first argument to printf()
sources = saf.call("gets")
sinks = saf.arg_to("printf", 0)

# Run taint flow analysis
findings = q.taint_flow(sources, sinks)

if not findings:
    print("No taint flows found from gets() to printf()")
    print("Tip: Select a C program that uses gets() and printf()")
else:
    print(f"Found {len(findings)} taint flow(s):")
    for f in findings:
        print(f"  {f.check}: {f.message}")
        print(f"  Source: {f.source_location}")
        print(f"    Sink: {f.sink_location}")
        if f.trace.steps:
            print(f"  Trace:")
            print(f"    {f.trace.pretty()}")
        saf.report(f.sink_id or "unknown", "high",
                   f"Taint flow: {f.source_location} -> {f.sink_location}")

# Also demonstrate graph access via SDK
gs = proj.graphs()
print(f"\\nAvailable graphs: {gs.available()}")
cg = proj.call_graph()
print(f"Call graph: {len(cg.nodes)} nodes, {len(cg.edges)} edges")
`,
  },
  custom: {
    label: 'Custom analyzer (blank)',
    code: `import saf

# === SAF Python API ===
# Legacy API (PropertyGraph-based):
#   result = saf.analyze()
#   result.cfg / result.callgraph / result.defuse / result.valueflow
#
# SDK-compatible API (same as native SAF Python SDK):
#   proj = saf.Project()
#   q = proj.query()
#   findings = q.taint_flow(saf.call("gets"), saf.arg_to("strcpy", 0))
#   findings = q.flows(saf.function_return("malloc"), saf.arg_to("free", 0))
#   pta = proj.pta_result()
#   gs = proj.graphs()
#
# Selectors: saf.call(fn), saf.arg_to(fn, idx),
#            saf.function_param(fn, idx), saf.function_return(fn),
#            saf.global_var(name)

result = saf.analyze()
print(f"Loaded {len(result.cfg.nodes)} CFG nodes")
`,
  },
};

type OutputTab = 'console' | 'findings';

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface AnalyzerPanelProps {
  onAnalyze: () => Promise<AnalyzeOutput | null>;
}

const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

export function AnalyzerPanel({ onAnalyze }: AnalyzerPanelProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const codeRef = useRef(TEMPLATES.list_free.code);

  const [pyodideStatus, setPyodideStatus] = useState<
    'idle' | 'loading' | 'ready'
  >(isPyodideReady() ? 'ready' : 'idle');
  const [running, setRunning] = useState(false);
  const [outputTab, setOutputTab] = useState<OutputTab>('console');
  const [consoleOutput, setConsoleOutput] = useState('');
  const [consoleError, setConsoleError] = useState<string | null>(null);
  const [findings, setFindings] = useState<Finding[]>([]);
  const [showHelp, setShowHelp] = useState(false);

  // Pre-warm Pyodide on mount
  useEffect(() => {
    if (!isPyodideReady()) {
      setPyodideStatus('loading');
      preloadPyodide();
      // Poll until ready
      const interval = setInterval(() => {
        if (isPyodideReady()) {
          setPyodideStatus('ready');
          clearInterval(interval);
        }
      }, 500);
      return () => clearInterval(interval);
    }
  }, []);

  // Mount CodeMirror editor
  useEffect(() => {
    if (!editorRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        codeRef.current = update.state.doc.toString();
      }
    });

    const state = EditorState.create({
      doc: codeRef.current,
      extensions: [
        basicSetup,
        python(),
        morandiTheme,
        safAutocomplete,
        updateListener,
        keymap.of([indentWithTab]),
        EditorView.theme({
          '&': { height: '100%' },
          '.cm-scroller': { overflow: 'auto' },
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  // Template selection
  const handleTemplateChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const tmpl = TEMPLATES[e.target.value];
      if (!tmpl) return;
      codeRef.current = tmpl.code;
      const view = viewRef.current;
      if (view) {
        view.dispatch({
          changes: {
            from: 0,
            to: view.state.doc.length,
            insert: tmpl.code,
          },
        });
      }
    },
    [],
  );

  // Run the script, auto-triggering analysis first if needed
  const handleRun = useCallback(async () => {
    setRunning(true);
    setConsoleOutput('');
    setConsoleError(null);

    const collectedFindings: Finding[] = [];
    setFindings([]);

    try {
      // Always re-run analysis to ensure results match current source code
      const output = await onAnalyze();
      if (!output) {
        setConsoleError('Analysis failed — check the error banner above.');
        return;
      }

      const pyodide = await initPyodide();
      setPyodideStatus('ready');

      await setupSafBridge(
        pyodide,
        output.results,
        (finding) => { collectedFindings.push(finding); },
        output.airJson,
        output.wasmConfig,
      );

      const scriptResult: ScriptResult = await runUserScript(
        pyodide,
        codeRef.current,
      );

      setConsoleOutput(scriptResult.stdout);
      setConsoleError(scriptResult.error);
      setFindings(collectedFindings);

      // Auto-switch to findings tab if there are findings and no errors
      if (collectedFindings.length > 0 && !scriptResult.error) {
        setOutputTab('findings');
      }
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setConsoleError(message);
    } finally {
      setRunning(false);
    }
  }, [onAnalyze]);

  const pyodideLabel =
    pyodideStatus === 'ready'
      ? 'Pyodide ready'
      : pyodideStatus === 'loading'
        ? 'Loading Pyodide...'
        : 'Pyodide idle';

  return (
    <div className="analyzer-panel">
      {/* Toolbar */}
      <div className="analyzer-toolbar">
        <button
          className="btn-run"
          onClick={handleRun}
          disabled={running || pyodideStatus === 'loading'}
        >
          {running ? 'Running...' : 'Run'}
        </button>

        <select onChange={handleTemplateChange} defaultValue="list_free">
          {Object.entries(TEMPLATES).map(([key, tmpl]) => (
            <option key={key} value={key}>
              {tmpl.label}
            </option>
          ))}
        </select>

        <button
          className="btn-help"
          onClick={() => setShowHelp((v) => !v)}
          title="Keyboard shortcuts"
          aria-label="Keyboard shortcuts"
        >
          ?
        </button>

        <div className="pyodide-status">
          <span className={`pyodide-dot ${pyodideStatus}`} />
          <span className="pyodide-status-text">{pyodideLabel}</span>
        </div>
      </div>

      {showHelp && (
        <div className="help-popover">
          <div className="help-popover-header">
            <span>Keyboard Shortcuts</span>
            <button className="help-popover-close" onClick={() => setShowHelp(false)}>&times;</button>
          </div>
          <table className="help-popover-table">
            <tbody>
              <tr><td><kbd>Tab</kbd></td><td>Indent line / selection</td></tr>
              <tr><td><kbd>Shift</kbd>+<kbd>Tab</kbd></td><td>Unindent line / selection</td></tr>
              <tr><td><kbd>{isMac ? 'Cmd' : 'Ctrl'}</kbd>+<kbd>/</kbd></td><td>Toggle line comment</td></tr>
              <tr><td><kbd>{isMac ? 'Cmd' : 'Ctrl'}</kbd>+<kbd>D</kbd></td><td>Select next occurrence</td></tr>
              <tr><td><kbd>{isMac ? 'Cmd' : 'Ctrl'}</kbd>+<kbd>F</kbd></td><td>Find &amp; replace</td></tr>
              <tr><td><kbd>{isMac ? 'Cmd' : 'Ctrl'}</kbd>+<kbd>Z</kbd></td><td>Undo</td></tr>
              <tr><td><kbd>{isMac ? 'Cmd' : 'Ctrl'}</kbd>+<kbd>Shift</kbd>+<kbd>Z</kbd></td><td>Redo</td></tr>
              <tr><td><kbd>Ctrl</kbd>+<kbd>Space</kbd></td><td>Trigger autocomplete</td></tr>
            </tbody>
          </table>
          <div className="help-popover-hint">
            Type <code>saf.</code> or <code>result.</code> for SAF API completions.
            The editor uses <strong>tabs for indentation</strong>.
          </div>
        </div>
      )}

      {/* Editor */}
      <div className="analyzer-editor" ref={editorRef} />

      {/* Output pane */}
      <div className="analyzer-output">
        <div className="analyzer-output-tabs">
          <button
            className={`analyzer-output-tab ${outputTab === 'console' ? 'active' : ''}`}
            onClick={() => setOutputTab('console')}
          >
            Console
          </button>
          <button
            className={`analyzer-output-tab ${outputTab === 'findings' ? 'active' : ''}`}
            onClick={() => setOutputTab('findings')}
          >
            Findings{findings.length > 0 ? ` (${findings.length})` : ''}
          </button>
        </div>

        <div className="analyzer-output-content">
          {outputTab === 'console' && (
            <>
              {consoleOutput || (!consoleError && (
                <span className="analyzer-output-placeholder">
                  Run a script to see output here.
                </span>
              ))}
              {consoleError && (
                <div className="error-text">{consoleError}</div>
              )}
            </>
          )}

          {outputTab === 'findings' && (
            <>
              {findings.length === 0 ? (
                <span className="analyzer-output-placeholder">
                  No findings reported. Use saf.report() in your script.
                </span>
              ) : (
                <div className="findings-list">
                  {findings.map((f, i) => (
                    <div
                      key={i}
                      className={`finding-card severity-${f.severity}`}
                    >
                      <div className="finding-severity">{f.severity}</div>
                      <div className="finding-message">{f.message}</div>
                      <div className="finding-node">{f.nodeId}</div>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
