import { motion } from 'motion/react';
import type { MssaState } from '../../content/trace-types';

interface MssaInspectorProps {
  state: MssaState;
}

export default function MssaInspector({ state }: MssaInspectorProps) {
  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Clobber Query</h4>
        <p className="mssa-query">{state.query}</p>
      </div>

      <div className="inspector-section">
        <h4>Clobber Walk</h4>
        {state.walkChain.length === 0 ? (
          <p className="inspector-empty">No walk entries yet</p>
        ) : (
          <div className="walk-chain">
            {state.walkChain.map((entry, i) => (
              <motion.div
                key={`${entry.inst}-${i}`}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.2, delay: i * 0.1 }}
              >
                {i > 0 && <div className="walk-arrow">&darr;</div>}
                <div className="walk-entry">
                  <span className={`walk-type-badge ${entry.type}`}>{entry.type}</span>
                  <span>{entry.inst}</span>
                  {entry.aliasQuery && (
                    <span className={`walk-alias ${entry.aliasQuery.result}`}>
                      {entry.aliasQuery.ptr1} vs {entry.aliasQuery.ptr2}: {entry.aliasQuery.result}
                    </span>
                  )}
                </div>
              </motion.div>
            ))}
          </div>
        )}
      </div>

      <div className="inspector-section">
        <h4>Points-To Context</h4>
        {Object.keys(state.pointsToContext).length === 0 ? (
          <p className="inspector-empty">No context loaded</p>
        ) : (
          <div className="pts-table">
            {Object.entries(state.pointsToContext).map(([ptr, locs]) => (
              <div key={ptr} className="pts-row">
                <span className="pts-var">{ptr}</span>
                <span className="pts-arrow">&rarr;</span>
                <span className="pts-locs">{`{${locs.join(', ')}}`}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
