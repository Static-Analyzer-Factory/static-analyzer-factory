import './PhaseBar.css';

interface PhaseBarProps {
  phases: string[];
  currentPhase: number;
}

export default function PhaseBar({ phases, currentPhase }: PhaseBarProps) {
  return (
    <div className="phase-bar">
      {phases.map((phase, i) => {
        const status = i < currentPhase ? 'completed' : i === currentPhase ? 'active' : 'pending';
        return (
          <div key={i} className="phase-item">
            {i > 0 && <span className="phase-arrow">{'\u2192'}</span>}
            <span className={`phase-node phase-${status}`}>
              <span className="phase-dot" />
              <span className="phase-label">{phase}</span>
            </span>
          </div>
        );
      })}
    </div>
  );
}
