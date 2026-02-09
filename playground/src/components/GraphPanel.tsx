import { useCallback, useEffect, useRef } from 'react';
import type { Core, ElementDefinition } from 'cytoscape';
import type { AnalysisResults } from '@saf/web-shared/types';
import {
  createCyInstance,
  renderCFG, cfgLayoutOptions,
  renderCallGraph, callgraphLayoutOptions,
  renderDefUse, defuseLayoutOptions,
  renderValueFlow, valueflowLayoutOptions,
  renderPTA, ptaLayoutOptions,
} from '@saf/web-shared/graph';

type GraphType = 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';

const GRAPH_LABELS: Record<GraphType, string> = {
  cfg: 'CFG',
  callgraph: 'Call Graph',
  defuse: 'Def-Use',
  valueflow: 'Value Flow',
  pta: 'PTA',
};

interface GraphPanelProps {
  activeGraph: GraphType;
  onGraphChange: (graph: GraphType) => void;
  results: AnalysisResults | null;
  selectedFunction: string | null;
  onFunctionSelect: (fn: string | null) => void;
  onSourceHighlight?: (lines: [number, number] | null) => void;
}

const EMPTY_GRAPH = { schema_version: '0.1.0', graph_type: '', nodes: [], edges: [], metadata: {} };
const EMPTY_PTA = { points_to: [] };

function getElementsAndLayout(
  activeGraph: GraphType,
  results: AnalysisResults,
  selectedFunction: string | null,
): { elements: ElementDefinition[]; layout: Record<string, unknown> } {
  switch (activeGraph) {
    case 'cfg':
      return {
        elements: renderCFG(results.cfg ?? EMPTY_GRAPH, selectedFunction),
        layout: cfgLayoutOptions,
      };
    case 'callgraph':
      return {
        elements: renderCallGraph(results.callgraph ?? EMPTY_GRAPH),
        layout: callgraphLayoutOptions,
      };
    case 'defuse':
      return {
        elements: renderDefUse(results.defuse ?? EMPTY_GRAPH, selectedFunction),
        layout: defuseLayoutOptions,
      };
    case 'valueflow':
      return {
        elements: renderValueFlow(results.valueflow ?? EMPTY_GRAPH),
        layout: valueflowLayoutOptions,
      };
    case 'pta':
      return {
        elements: renderPTA(results.pta ?? EMPTY_PTA),
        layout: ptaLayoutOptions,
      };
  }
}

export function GraphPanel({
  activeGraph,
  onGraphChange,
  results,
  selectedFunction,
  onFunctionSelect,
  onSourceHighlight,
}: GraphPanelProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);

  // Node click handler — select function from callgraph or CFG nodes
  const handleNodeClick = useCallback(
    (fn: string | null) => {
      onFunctionSelect(fn);
    },
    [onFunctionSelect],
  );

  useEffect(() => {
    if (!containerRef.current || !results) return;

    // Destroy previous instance
    if (cyRef.current) {
      cyRef.current.destroy();
      cyRef.current = null;
    }

    const { elements, layout } = getElementsAndLayout(
      activeGraph,
      results,
      selectedFunction,
    );

    if (elements.length === 0) return;

    const cy = createCyInstance(
      containerRef.current,
      elements,
      layout.name as string,
      layout,
    );
    cyRef.current = cy;

    // Fit after layout completes
    cy.on('layoutstop', () => {
      cy.fit(undefined, 30);
    });

    // Node click: emit function name for filtering + source line highlighting
    cy.on('tap', 'node', (evt) => {
      const node = evt.target;
      const fn =
        (node.data('function') as string) ||
        (node.data('kind') === 'defined' ? (node.data('label') as string) : null);
      if (fn) {
        handleNodeClick(fn);
      }

      // Highlight source lines for clicked nodes
      const lineStart = node.data('lineStart') as number | undefined;
      const lineEnd = node.data('lineEnd') as number | undefined;
      if (lineStart != null && lineEnd != null && onSourceHighlight) {
        onSourceHighlight([lineStart, lineEnd]);
      }
    });

    // Click on background clears highlight
    cy.on('tap', (evt) => {
      if (evt.target === cy && onSourceHighlight) {
        onSourceHighlight(null);
      }
    });

    return () => {
      cy.destroy();
      cyRef.current = null;
      // Clear container so React's reconciliation doesn't choke on stale children
      if (containerRef.current) {
        containerRef.current.innerHTML = '';
      }
    };
  }, [results, activeGraph, selectedFunction, handleNodeClick, onSourceHighlight]);

  const graphTypes: GraphType[] = [
    'cfg',
    'callgraph',
    'defuse',
    'valueflow',
    'pta',
  ];

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>Analysis</h2>
        {results?.functions && results.functions.length > 0 && (
          <select
            value={selectedFunction || ''}
            onChange={(e) => onFunctionSelect(e.target.value || null)}
          >
            <option value="">All functions</option>
            {results.functions.map((fn) => (
              <option key={fn} value={fn}>
                {fn}
              </option>
            ))}
          </select>
        )}
      </div>
      <div className="tab-bar">
        {graphTypes.map((g) => (
          <button
            key={g}
            className={`tab ${activeGraph === g ? 'active' : ''}`}
            onClick={() => onGraphChange(g)}
          >
            {GRAPH_LABELS[g]}
          </button>
        ))}
      </div>
      <div className="panel-content">
        <div
          className="graph-container"
          ref={containerRef}
          style={{ display: results ? 'block' : 'none' }}
        />
        {!results && (
          <div className="placeholder">
            <p>
              Run analysis to see graph visualizations for CFG, call graph,
              def-use chains, value flow, and points-to analysis
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
