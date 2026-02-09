import { AnimatePresence, motion } from 'motion/react';
import type { IfdsState, TraceDiff } from '../../content/trace-types';

interface IfdsInspectorProps {
  state: IfdsState;
  diff: TraceDiff;
}

export default function IfdsInspector({ state, diff }: IfdsInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);
  const maxVisible = 5;
  const visibleWorklist = state.worklist.slice(0, maxVisible);
  const remaining = state.worklist.length - maxVisible;

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Worklist (Path Edges)</h4>
        {state.worklist.length === 0 ? (
          <p className="inspector-empty">Empty — tabulation complete</p>
        ) : (
          <div className="path-edge-list">
            <AnimatePresence>
              {visibleWorklist.map((edge, i) => (
                <motion.div
                  key={`${edge.func}-${edge.d1}-${edge.inst}-${edge.d2}-${i}`}
                  className="path-edge"
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2, delay: i * 0.05 }}
                >
                  ({edge.func}, {edge.d1}) &rarr; ({edge.inst}, {edge.d2})
                </motion.div>
              ))}
            </AnimatePresence>
            {remaining > 0 && (
              <span className="path-edge-more">+{remaining} more</span>
            )}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Summary Edges</h4>
        {state.summaryEdges.length === 0 ? (
          <p className="inspector-empty">No summaries computed yet</p>
        ) : (
          <div className="summary-edge-list">
            {state.summaryEdges.map((se, i) => (
              <div key={i} className="summary-edge">
                {se.func}: ({se.entryFact}) &rarr; ({se.exitFact})
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Facts at Instructions</h4>
        {Object.keys(state.factsAt).length === 0 ? (
          <p className="inspector-empty">No facts propagated yet</p>
        ) : (
          <div className="facts-list">
            <AnimatePresence>
              {Object.entries(state.factsAt).map(([inst, facts]) => (
                <motion.div
                  key={inst}
                  className={`facts-row ${changedNodes.has(inst) ? 'changed' : ''}`}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3 }}
                >
                  <strong>{inst}:</strong> {`{${facts.join(', ')}}`}
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        )}
      </div>
    </div>
  );
}
