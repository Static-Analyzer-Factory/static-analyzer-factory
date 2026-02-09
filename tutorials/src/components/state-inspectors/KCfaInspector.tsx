import { AnimatePresence, motion } from 'motion/react';
import type { KCfaState, TraceDiff } from '../../content/trace-types';

interface KCfaInspectorProps {
  state: KCfaState;
  diff: TraceDiff;
}

export default function KCfaInspector({ state, diff }: KCfaInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Current Context</h4>
        {state.currentContext.length === 0 ? (
          <p className="inspector-empty">Global (context-insensitive)</p>
        ) : (
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flexWrap: 'wrap' }}>
            {state.currentContext.map((ctx, i) => (
              <span key={i} style={{ display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                {i > 0 && (
                  <span style={{ color: '#888', fontSize: '0.85em' }}>&rarr;</span>
                )}
                <span
                  style={{
                    fontFamily: 'monospace',
                    fontSize: '0.85em',
                    background: '#e8f0fe',
                    color: '#1a56db',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontWeight: 500,
                  }}
                >
                  [{ctx}]
                </span>
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Worklist</h4>
        {state.worklist.length === 0 ? (
          <p className="inspector-empty">Empty — fixed point reached</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
            <AnimatePresence>
              {state.worklist.map((item, i) => (
                <motion.div
                  key={`${item.context.join(',')}-${item.variable}-${i}`}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    fontFamily: 'monospace',
                    fontSize: '0.85em',
                    padding: '3px 0',
                  }}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2, delay: i * 0.05 }}
                >
                  <span style={{ color: '#888' }}>
                    {item.context.length === 0
                      ? '[]'
                      : item.context.map(c => `[${c}]`).join(' \u2192 ')}
                  </span>
                  <span style={{ color: '#555' }}>,</span>
                  <span style={{ fontWeight: 600 }}>{item.variable}</span>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Points-To Sets (per context)</h4>
        {Object.keys(state.pointsTo).length === 0 ? (
          <p className="inspector-empty">No points-to information yet</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {Object.entries(state.pointsTo).map(([context, variables]) => (
              <div key={context}>
                <div
                  style={{
                    fontFamily: 'monospace',
                    fontSize: '0.8em',
                    color: '#1a56db',
                    background: '#e8f0fe',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    display: 'inline-block',
                    marginBottom: '4px',
                    fontWeight: 500,
                  }}
                >
                  {context}
                </div>
                <div className="pts-table">
                  <AnimatePresence>
                    {Object.entries(variables).map(([variable, locations]) => (
                      <motion.div
                        key={`${context}-${variable}`}
                        className={`pts-row ${changedNodes.has(variable) ? 'changed' : ''}`}
                        layout
                        initial={{ opacity: 0, x: -20 }}
                        animate={{ opacity: 1, x: 0 }}
                        transition={{ duration: 0.3 }}
                      >
                        <span className="pts-var">{variable}</span>
                        <span className="pts-arrow">&rarr;</span>
                        <span className="pts-locs">{`{${locations.join(', ')}}`}</span>
                        {changedNodes.has(variable) && <span className="pts-badge">changed</span>}
                      </motion.div>
                    ))}
                  </AnimatePresence>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Context Count</h4>
        <span
          style={{
            display: 'inline-block',
            fontFamily: 'monospace',
            fontSize: '0.9em',
            background: state.contextCount > 0 ? '#e8f0fe' : '#f3f4f6',
            color: state.contextCount > 0 ? '#1a56db' : '#888',
            padding: '3px 10px',
            borderRadius: '12px',
            fontWeight: 600,
          }}
        >
          {state.contextCount} {state.contextCount === 1 ? 'context' : 'contexts'}
        </span>
      </div>
    </div>
  );
}
