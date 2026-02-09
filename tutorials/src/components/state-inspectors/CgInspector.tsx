import { AnimatePresence, motion } from 'motion/react';
import type { CgState, TraceDiff } from '../../content/trace-types';

interface CgInspectorProps {
  state: CgState;
  diff: TraceDiff;
}

const algorithmColors: Record<CgState['algorithm'], { background: string; color: string }> = {
  cha: { background: '#dbeafe', color: '#1e40af' },
  rta: { background: '#fef3c7', color: '#92400e' },
  vta: { background: '#d1fae5', color: '#065f46' },
};

const precisionColors: Record<string, { background: string; color: string }> = {
  exact: { background: '#d1fae5', color: '#065f46' },
  'over-approx': { background: '#fef3c7', color: '#92400e' },
};

export default function CgInspector({ state, diff }: CgInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);
  const algoStyle = algorithmColors[state.algorithm];

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Algorithm</h4>
        <span
          style={{
            display: 'inline-block',
            padding: '4px 12px',
            borderRadius: '6px',
            fontSize: '0.9em',
            fontWeight: 600,
            fontFamily: 'monospace',
            background: algoStyle.background,
            color: algoStyle.color,
            border: `1px solid ${algoStyle.color}40`,
          }}
        >
          {state.algorithm.toUpperCase()}
        </span>
      </div>

      <div className="inspector-section">
        <h4>Resolved Calls</h4>
        {state.resolvedCalls.length === 0 ? (
          <p className="inspector-empty">No calls resolved yet</p>
        ) : (
          <div className="pts-table">
            <AnimatePresence>
              {state.resolvedCalls.map(call => {
                const precStyle = precisionColors[call.precision];
                return (
                  <motion.div
                    key={call.callsite}
                    className={`pts-row ${changedNodes.has(call.callsite) ? 'changed' : ''}`}
                    layout
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3 }}
                  >
                    <span className="pts-var">{call.callsite}</span>
                    <span className="pts-arrow">&rarr;</span>
                    <span style={{ display: 'flex', flexWrap: 'wrap', gap: '4px', alignItems: 'center' }}>
                      {call.targets.map(target => (
                        <span
                          key={target}
                          style={{
                            display: 'inline-block',
                            padding: '2px 8px',
                            borderRadius: '4px',
                            fontSize: '0.8em',
                            fontFamily: 'monospace',
                            background: '#f1f5f9',
                            border: '1px solid #cbd5e1',
                            color: '#1e293b',
                          }}
                        >
                          {target}
                        </span>
                      ))}
                      <span
                        style={{
                          display: 'inline-block',
                          padding: '2px 6px',
                          borderRadius: '4px',
                          fontSize: '0.75em',
                          fontWeight: 600,
                          background: precStyle.background,
                          color: precStyle.color,
                          border: `1px solid ${precStyle.color}40`,
                        }}
                      >
                        {call.precision}
                      </span>
                    </span>
                  </motion.div>
                );
              })}
            </AnimatePresence>
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Reachable Methods</h4>
        {state.reachableMethods.length === 0 ? (
          <p className="inspector-empty">No methods reachable yet</p>
        ) : (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
            {state.reachableMethods.map(method => (
              <span
                key={method}
                style={{
                  display: 'inline-block',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '0.8em',
                  fontFamily: 'monospace',
                  background: changedNodes.has(method) ? '#dbeafe' : '#f1f5f9',
                  border: changedNodes.has(method) ? '1px solid #3b82f6' : '1px solid #cbd5e1',
                  color: '#1e293b',
                }}
              >
                {method}
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Unresolved Calls</h4>
        {state.unresolvedCalls.length === 0 ? (
          <p className="inspector-empty">All calls resolved</p>
        ) : (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
            {state.unresolvedCalls.map(callsite => (
              <span
                key={callsite}
                style={{
                  display: 'inline-block',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  fontSize: '0.8em',
                  fontFamily: 'monospace',
                  background: '#fee2e2',
                  border: '1px solid #fca5a5',
                  color: '#991b1b',
                }}
              >
                {callsite}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
