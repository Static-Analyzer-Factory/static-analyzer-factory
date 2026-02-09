/**
 * QueryPanel — Interactive query builder for ProgramDatabase.
 *
 * Two sub-tabs:
 * - Checks: dropdown of named checks, Run/Run All, findings list
 * - Raw JSON: freeform JSON request textarea, raw response display
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { indentWithTab } from '@codemirror/commands';
import { basicSetup } from 'codemirror';
import { json as jsonLang } from '@codemirror/lang-json';
import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';
import type { CatalogEntry, QueryFinding, QueryResponse } from '@saf/web-shared/types';
import { runQuery } from '@saf/web-shared/analysis';
import './QueryPanel.css';

type SubTab = 'checks' | 'raw';

/** A preset raw JSON query with a label and usage hint. */
interface RawPreset {
  label: string;
  /** Which example program this works best with. */
  hint: string;
  json: object;
}

const RAW_PRESETS: RawPreset[] = [
  {
    label: 'Schema — discover API',
    hint: 'any program',
    json: { action: 'schema' },
  },
  {
    label: 'Run all checks',
    hint: 'try with Use-After-Free or Memory Leak',
    json: { action: 'check_all' },
  },
  {
    label: 'Check: use-after-free',
    hint: 'Use-After-Free example',
    json: { action: 'check', name: 'use_after_free' },
  },
  {
    label: 'Check: memory leak',
    hint: 'Memory Leak example',
    json: { action: 'check', name: 'memory_leak' },
  },
  {
    label: 'Check: double free',
    hint: 'Double Free example',
    json: { action: 'check', name: 'double_free' },
  },
  {
    label: 'Check: null dereference',
    hint: 'Null Dereference example',
    json: { action: 'check', name: 'null_deref' },
  },
  {
    label: 'Taint: source() → sink()',
    hint: 'Taint Flow example',
    json: {
      action: 'query',
      type: 'taint_flow',
      params: {
        sources: [{ kind: 'function_return', function: 'source' }],
        sinks: [{ kind: 'arg_to', callee: 'sink', index: 0 }],
      },
    },
  },
  {
    label: 'Flows: malloc → free',
    hint: 'Use-After-Free or Library Modeling example',
    json: {
      action: 'query',
      type: 'flows',
      params: {
        sources: [{ kind: 'call_to', callee: 'malloc' }],
        sinks: [{ kind: 'arg_to', callee: 'free', index: 0 }],
      },
    },
  },
  {
    label: 'Flows: fopen → fclose',
    hint: 'File Descriptor Leak example',
    json: {
      action: 'query',
      type: 'flows',
      params: {
        sources: [{ kind: 'call_to', callee: 'fopen' }],
        sinks: [{ kind: 'arg_to', callee: 'fclose', index: 0 }],
      },
    },
  },
  {
    label: 'Taint: all params → printf',
    hint: 'Taint Flow example',
    json: {
      action: 'query',
      type: 'taint_flow',
      params: {
        sources: [{ kind: 'function_param', function: '*' }],
        sinks: [{ kind: 'arg_to', callee: 'printf' }],
      },
    },
  },
];

interface QueryPanelProps {
  hasResults: boolean;
}

export function QueryPanel({ hasResults }: QueryPanelProps) {
  const [activeTab, setActiveTab] = useState<SubTab>('checks');
  const [checks, setChecks] = useState<CatalogEntry[]>([]);
  const [selectedCheck, setSelectedCheck] = useState<string>('');
  const [findings, setFindings] = useState<QueryFinding[]>([]);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastElapsed, setLastElapsed] = useState<number | null>(null);
  const [enginesUsed, setEnginesUsed] = useState<string[]>([]);
  const [hasRun, setHasRun] = useState(false);
  const [rawResponse, setRawResponse] = useState<string | null>(null);
  const [rawNotImplemented, setRawNotImplemented] = useState<string | null>(null);
  const [jsonValid, setJsonValid] = useState(true);
  const [jsonError, setJsonError] = useState<string | null>(null);

  // CodeMirror editor for raw JSON
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const rawRequestRef = useRef('{\n  "action": "schema"\n}');

  // Load schema when results become available
  useEffect(() => {
    if (!hasResults) {
      setChecks([]);
      setSelectedCheck('');
      setFindings([]);
      setError(null);
      setLastElapsed(null);
      setEnginesUsed([]);
      setHasRun(false);
      setRawResponse(null);
      setRawNotImplemented(null);
      return;
    }

    try {
      const resp = runQuery('{"action":"schema"}');
      if (resp.status === 'ok' && resp.checks) {
        setChecks(resp.checks);
        if (resp.checks.length > 0) {
          setSelectedCheck(resp.checks[0].name);
        }
      }
    } catch (err) {
      console.warn('[QueryPanel] Failed to load schema:', err);
    }
  }, [hasResults]);

  // Mount CodeMirror JSON editor when the Raw tab becomes visible.
  // The editor div only exists in the DOM when activeTab === 'raw',
  // so we must re-run when activeTab changes.
  useEffect(() => {
    if (activeTab !== 'raw' || !editorRef.current) return;
    // Already mounted — don't recreate
    if (viewRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const text = update.state.doc.toString();
        rawRequestRef.current = text;
        try {
          JSON.parse(text);
          setJsonValid(true);
          setJsonError(null);
        } catch (e) {
          setJsonValid(false);
          setJsonError(e instanceof Error ? e.message : String(e));
        }
      }
    });

    const state = EditorState.create({
      doc: rawRequestRef.current,
      extensions: [
        basicSetup,
        jsonLang(),
        morandiTheme,
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
  }, [activeTab]);

  // Helper to set editor content programmatically
  const setEditorContent = useCallback((text: string) => {
    rawRequestRef.current = text;
    const view = viewRef.current;
    if (view) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: text },
      });
    }
    try {
      JSON.parse(text);
      setJsonValid(true);
      setJsonError(null);
    } catch (e) {
      setJsonValid(false);
      setJsonError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const handleRunCheck = useCallback(() => {
    if (!selectedCheck) return;
    setRunning(true);
    setError(null);
    setFindings([]);
    setLastElapsed(null);

    try {
      const request = JSON.stringify({ action: 'check', name: selectedCheck });
      const resp: QueryResponse = runQuery(request);

      if (resp.status === 'error' && resp.error) {
        setError(`${resp.error.code}: ${resp.error.message}`);
      } else if (resp.findings) {
        setFindings(resp.findings);
      }
      if (resp.metadata?.elapsed_ms != null) {
        setLastElapsed(resp.metadata.elapsed_ms);
      }
      if (resp.metadata?.engines_used) {
        setEnginesUsed(resp.metadata.engines_used);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
      setHasRun(true);
    }
  }, [selectedCheck]);

  const handleRunAll = useCallback(() => {
    setRunning(true);
    setError(null);
    setFindings([]);
    setLastElapsed(null);
    setSelectedCheck('');

    try {
      const resp: QueryResponse = runQuery('{"action":"check_all"}');

      if (resp.status === 'error' && resp.error) {
        setError(`${resp.error.code}: ${resp.error.message}`);
      } else if (resp.findings) {
        setFindings(resp.findings);
      }
      if (resp.metadata?.elapsed_ms != null) {
        setLastElapsed(resp.metadata.elapsed_ms);
      }
      if (resp.metadata?.engines_used) {
        setEnginesUsed(resp.metadata.engines_used);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
      setHasRun(true);
    }
  }, []);

  const handleRawSend = useCallback(() => {
    setRawResponse(null);
    setRawNotImplemented(null);
    const text = rawRequestRef.current;
    try {
      JSON.parse(text); // validate first
    } catch {
      setRawResponse('Error: Invalid JSON — fix syntax errors before sending.');
      return;
    }
    try {
      const resp = runQuery(text);
      if (resp.status === 'error' && resp.error?.code === 'NOT_IMPLEMENTED') {
        setRawNotImplemented(resp.error.message);
      }
      setRawResponse(JSON.stringify(resp, null, 2));
    } catch (err) {
      setRawResponse(`Error: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, []);

  const selectedEntry = checks.find((c) => c.name === selectedCheck);

  if (!hasResults) {
    return (
      <div className="query-panel">
        <div className="query-placeholder">
          <p>Run analysis first to enable queries.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="query-panel">
      {/* Sub-tab bar */}
      <div className="query-tabs">
        <button
          className={`query-tab ${activeTab === 'checks' ? 'active' : ''}`}
          onClick={() => setActiveTab('checks')}
        >
          Checks
        </button>
        <button
          className={`query-tab ${activeTab === 'raw' ? 'active' : ''}`}
          onClick={() => setActiveTab('raw')}
        >
          Raw JSON
        </button>
      </div>

      {activeTab === 'checks' && (
        <div className="query-checks">
          {/* Controls */}
          <div className="query-controls">
            <select
              value={selectedCheck}
              onChange={(e) => setSelectedCheck(e.target.value)}
              disabled={running}
            >
              {checks.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
            <button
              className="query-btn"
              onClick={handleRunCheck}
              disabled={running || !selectedCheck}
            >
              {running ? <><span className="query-spinner" />Running</> : 'Run Check'}
            </button>
            <button
              className="query-btn query-btn-secondary"
              onClick={handleRunAll}
              disabled={running}
            >
              Run All
            </button>
          </div>

          {/* Check info */}
          {selectedEntry && (
            <div className="query-check-info">
              <div className="query-check-meta">
                <span className={`query-badge query-badge-severity ${selectedEntry.severity}`}>
                  {selectedEntry.severity}
                </span>
                {selectedEntry.cwe && (
                  <span className="query-badge query-badge-cwe">
                    CWE-{selectedEntry.cwe}
                  </span>
                )}
                <span className="query-badge query-badge-category">
                  {selectedEntry.category}
                </span>
              </div>
              <div className="query-check-description">
                {selectedEntry.description}
              </div>
            </div>
          )}

          {/* Findings */}
          <div className="query-findings">
            {error && (
              <div className="error-text" style={{ marginBottom: '0.5rem', fontSize: '0.8em' }}>
                {error}
              </div>
            )}

            {findings.length > 0 && (
              <>
                <div className="query-findings-header">
                  <span className="query-findings-title">
                    Findings
                  </span>
                  <span className="query-findings-count">
                    {findings.length} finding{findings.length !== 1 ? 's' : ''}
                    {lastElapsed != null && (
                      <span className="query-findings-elapsed">
                        {' '}({lastElapsed}ms)
                      </span>
                    )}
                  </span>
                </div>
                <div className="query-finding-list">
                  {findings.map((f, i) => (
                    <div
                      key={i}
                      className={`query-finding-card severity-${f.severity}`}
                    >
                      <div className="query-finding-top">
                        <span className="query-finding-severity">{f.severity}</span>
                        <span className="query-finding-check">{f.check}</span>
                        {f.object && (
                          <span className="query-finding-object">{f.object}</span>
                        )}
                        {f.cwe && (
                          <span className="query-finding-cwe">CWE-{f.cwe}</span>
                        )}
                      </div>
                      <div className="query-finding-message">{f.message}</div>
                      {f.path && f.path.length > 0 && (
                        <div className="query-finding-path">
                          {f.path.map((p, j) => (
                            <div key={j} className="query-finding-path-event">
                              <span className="query-path-marker">{p.event}</span>
                              <span className="query-path-location">{p.location}</span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </>
            )}

            {!error && findings.length === 0 && !running && !hasRun && (
              <div className="query-placeholder">
                <p>Select a check and click "Run Check" to scan for bugs.</p>
              </div>
            )}

            {!error && findings.length === 0 && !running && hasRun && (
              <div className="query-unsupported-notice">
                {enginesUsed.includes('absint') ? (
                  <>
                    <div className="query-findings-header">
                      <span className="query-findings-title">Not available in playground</span>
                    </div>
                    <p className="query-unsupported-detail">
                      This check requires the abstract interpretation engine, which is only available in the full SAF Python SDK. Use <code>project.request({`'{"action":"check","name":"..."}'`})</code> in Python.
                    </p>
                  </>
                ) : (
                  <div className="query-findings-header">
                    <span className="query-findings-title">No issues found</span>
                    {lastElapsed != null && (
                      <span className="query-findings-count">
                        ({lastElapsed}ms)
                      </span>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'raw' && (
        <div className="query-raw">
          <div className="query-raw-input">
            <div className="query-raw-presets">
              <select
                onChange={(e) => {
                  const idx = Number(e.target.value);
                  if (!Number.isNaN(idx) && idx >= 0) {
                    setEditorContent(JSON.stringify(RAW_PRESETS[idx].json, null, 2));
                  }
                }}
                defaultValue=""
              >
                <option value="" disabled>
                  Load preset...
                </option>
                {RAW_PRESETS.map((p, i) => (
                  <option key={i} value={i}>
                    {p.label}
                  </option>
                ))}
              </select>
              <span className={`query-raw-validity ${jsonValid ? 'valid' : 'invalid'}`}>
                {jsonValid ? 'Valid JSON' : 'Invalid JSON'}
              </span>
            </div>
            <div className="query-raw-editor" ref={editorRef} />
            {jsonError && (
              <div className="query-raw-json-error">{jsonError}</div>
            )}
            <div className="query-raw-actions">
              <button
                className="query-btn"
                onClick={handleRawSend}
                disabled={!jsonValid}
              >
                Send
              </button>
            </div>
          </div>
          <div className="query-raw-output">
            {rawNotImplemented && (
              <div className="query-unsupported-notice" style={{ marginBottom: '0.75rem' }}>
                <div className="query-findings-header">
                  <span className="query-findings-title">Not available in playground</span>
                </div>
                <p className="query-unsupported-detail">
                  {rawNotImplemented}. Use the full SAF Python SDK for this feature.
                </p>
              </div>
            )}
            {rawResponse ? (
              <pre>{rawResponse}</pre>
            ) : (
              <div className="query-placeholder">
                <p>Send a JSON request to see the response.</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
