import { motion } from 'motion/react';
import type { IntervalState } from '../../content/trace-types';

interface IntervalInspectorProps {
  state: IntervalState;
}

function formatBound(b: number | '-inf' | '+inf'): string {
  if (b === '-inf') return '-\u221E';
  if (b === '+inf') return '+\u221E';
  return String(b);
}

export default function IntervalInspector({ state }: IntervalInspectorProps) {
  const entries = Object.entries(state.variables);

  // Compute visual range across all variables for consistent bar sizing
  const numericBounds = entries.flatMap(([, v]) => {
    const vals: number[] = [];
    if (typeof v.lo === 'number') vals.push(v.lo);
    if (typeof v.hi === 'number') vals.push(v.hi);
    return vals;
  });
  const rangeMin = numericBounds.length > 0 ? Math.min(...numericBounds, 0) - 10 : -100;
  const rangeMax = numericBounds.length > 0 ? Math.max(...numericBounds, 0) + 10 : 100;
  const totalRange = rangeMax - rangeMin || 1;

  return (
    <div className="state-inspector">
      <div className="inspector-section">
        <h4>Variable Intervals (iteration {state.iteration}, block: {state.currentBlock})</h4>
        <div className="interval-bars">
          {entries.map(([name, interval]) => {
            const lo = typeof interval.lo === 'number' ? interval.lo : rangeMin;
            const hi = typeof interval.hi === 'number' ? interval.hi : rangeMax;
            const left = ((lo - rangeMin) / totalRange) * 100;
            const width = ((hi - lo) / totalRange) * 100;

            return (
              <div key={name} className="interval-bar-row">
                <span className="interval-var-name">{name}</span>
                <div className="interval-bar-track">
                  <motion.div
                    className="interval-bar-fill"
                    animate={{ left: `${left}%`, width: `${Math.max(width, 1)}%` }}
                    transition={{ type: 'spring', stiffness: 100, damping: 15 }}
                  />
                </div>
                <span className="interval-label">
                  [{formatBound(interval.lo)}, {formatBound(interval.hi)}]
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {state.operation && (
        <div className="inspector-section">
          <h4>Operation</h4>
          <div className="interval-op">
            <span className={`interval-op-type ${state.operation.type}`}>
              {state.operation.type}
            </span>
            <span>{state.operation.description}</span>
          </div>
        </div>
      )}
    </div>
  );
}
