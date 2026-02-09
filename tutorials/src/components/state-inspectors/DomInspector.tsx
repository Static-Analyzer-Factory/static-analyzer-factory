import { AnimatePresence, motion } from 'motion/react';
import type { DomState, TraceDiff } from '../../content/trace-types';

interface DomInspectorProps {
  state: DomState;
  diff: TraceDiff;
}

export default function DomInspector({ state, diff }: DomInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Processed</h4>
        {state.processed.length === 0 ? (
          <p className="inspector-empty">No blocks processed yet</p>
        ) : (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
            {state.processed.map(block => (
              <span
                key={block}
                style={{
                  display: 'inline-block',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '0.8em',
                  fontFamily: 'monospace',
                  background: changedNodes.has(block) ? '#dbeafe' : '#f1f5f9',
                  border: changedNodes.has(block) ? '1px solid #3b82f6' : '1px solid #cbd5e1',
                  color: '#1e293b',
                }}
              >
                {block}
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Immediate Dominators</h4>
        {Object.keys(state.idom).length === 0 ? (
          <p className="inspector-empty">No dominators computed yet</p>
        ) : (
          <div className="pts-table">
            <AnimatePresence>
              {Object.entries(state.idom).map(([block, dominator]) => (
                <motion.div
                  key={block}
                  className={`pts-row ${changedNodes.has(block) ? 'changed' : ''}`}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3 }}
                >
                  <span className="pts-var">{block}</span>
                  <span className="pts-arrow">&rarr;</span>
                  <span className="pts-locs">idom({block}) = {dominator}</span>
                  {changedNodes.has(block) && (
                    <span className="pts-badge">changed</span>
                  )}
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Dominance Frontier</h4>
        {Object.keys(state.domFrontier).length === 0 ? (
          <p className="inspector-empty">Not yet computed</p>
        ) : (
          <div className="pts-table">
            <AnimatePresence>
              {Object.entries(state.domFrontier).map(([block, frontier]) => (
                <motion.div
                  key={block}
                  className={`pts-row ${changedNodes.has(block) ? 'changed' : ''}`}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3 }}
                >
                  <span className="pts-var">DF({block})</span>
                  <span className="pts-arrow">=</span>
                  <span className="pts-locs">{`{${frontier.join(', ')}}`}</span>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Loop Headers</h4>
        {state.loopHeaders.length === 0 ? (
          <p className="inspector-empty">None detected</p>
        ) : (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
            {state.loopHeaders.map(header => (
              <span
                key={header}
                style={{
                  display: 'inline-block',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '0.8em',
                  fontFamily: 'monospace',
                  background: '#dcfce7',
                  border: '1px solid #22c55e',
                  color: '#166534',
                }}
              >
                {header}
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Back Edges</h4>
        {state.backEdges.length === 0 ? (
          <p className="inspector-empty">None detected</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
            {state.backEdges.map((edge, i) => (
              <div
                key={`${edge.src}-${edge.dst}-${i}`}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  fontFamily: 'monospace',
                  fontSize: '0.85em',
                }}
              >
                <span
                  style={{
                    display: 'inline-block',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    background: '#fef3c7',
                    border: '1px solid #c49a3c',
                    color: '#92400e',
                  }}
                >
                  {edge.src} &rarr; {edge.dst}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
