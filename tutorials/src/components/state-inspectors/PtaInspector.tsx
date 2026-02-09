import { AnimatePresence, motion } from 'motion/react';
import type { PtaState, TraceDiff } from '../../content/trace-types';

interface PtaInspectorProps {
  state: PtaState;
  diff: TraceDiff;
}

export default function PtaInspector({ state, diff }: PtaInspectorProps) {
  const changedNodes = new Set(diff.changed.nodes);

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Worklist</h4>
        {state.worklist.length === 0 ? (
          <p className="inspector-empty">Empty — fixed point reached</p>
        ) : (
          <div className="worklist-waves">
            {state.worklist.map(wave => (
              <div key={wave.rank} className="worklist-wave">
                <span className="wave-rank">R{wave.rank}:</span>
                <div className="wave-values">
                  {wave.values.map(v => (
                    <span key={v} className="wave-value">{v}</span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Points-To Sets</h4>
        {Object.keys(state.pointsTo).length === 0 ? (
          <p className="inspector-empty">No points-to information yet</p>
        ) : (
          <div className="pts-table">
            <AnimatePresence>
              {Object.entries(state.pointsTo).map(([variable, locations]) => (
                <motion.div
                  key={variable}
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
        )}
      </div>

      <div className="inspector-section">
        <h4>Constraints</h4>
        <div className="constraint-list">
          {state.constraints.map((c, i) => (
            <div key={i} className="constraint-row">
              <span className={`constraint-type ${c.type}`}>{c.type}</span>
              <span>{c.from} &rarr; {c.to}</span>
              {c.processed ? (
                <span className="constraint-check">&#10003;</span>
              ) : (
                <span className="constraint-pending">&#9711;</span>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
