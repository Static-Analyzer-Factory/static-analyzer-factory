import { AnimatePresence, motion } from 'motion/react';
import type { SvfState, TraceDiff } from '../../content/trace-types';

interface SvfInspectorProps {
  state: SvfState;
  diff: TraceDiff;
}

export default function SvfInspector({ state, diff }: SvfInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Current Node</h4>
        {state.currentNode ? (
          <span className="svf-current-node">{state.currentNode}</span>
        ) : (
          <p className="inspector-empty">None</p>
        )}
      </div>

      <div className="inspector-section">
        <h4>Value-Flow Edges</h4>
        <div className="constraint-list">
          {state.vfEdges.map((edge, i) => (
            <div key={i} className="constraint-row">
              <span
                className="constraint-type"
                style={
                  edge.kind === 'direct'
                    ? { background: '#e2e8f0', color: '#dbd6d0' }
                    : edge.kind === 'store'
                      ? { background: '#fef3c7', color: '#92400e' }
                      : { background: '#dbeafe', color: '#1e40af' }
                }
              >
                {edge.kind}
              </span>
              <span>{edge.src} &rarr; {edge.dst}</span>
              {edge.processed ? (
                <span className="constraint-check">&#10003;</span>
              ) : (
                <span className="constraint-pending">&#9711;</span>
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="inspector-section">
        <h4>Facts at Nodes</h4>
        {Object.keys(state.facts).length === 0 ? (
          <p className="inspector-empty">No facts propagated yet</p>
        ) : (
          <div className="pts-table">
            <AnimatePresence>
              {Object.entries(state.facts).map(([node, facts]) => (
                <motion.div
                  key={node}
                  className={`pts-row ${changedNodes.has(node) ? 'changed' : ''}`}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3 }}
                >
                  <span className="pts-var">{node}</span>
                  <span className="pts-arrow">&rarr;</span>
                  <span className="pts-locs">{`{${facts.join(', ')}}`}</span>
                  {changedNodes.has(node) && <span className="pts-badge">changed</span>}
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>
    </div>
  );
}
