import type { Pseudocode } from '../content/trace-types';
import './PseudocodeRail.css';

interface PseudocodeRailProps {
  pseudocode: Pseudocode;
  activeLines: number[];
  visitedLines: number[];
}

export default function PseudocodeRail({ pseudocode, activeLines, visitedLines }: PseudocodeRailProps) {
  return (
    <div className="pseudocode-rail">
      <div className="pseudocode-title">{pseudocode.title}</div>
      <div className="pseudocode-lines">
        {pseudocode.lines.map((line, i) => {
          const isActive = activeLines.includes(i);
          const isVisited = visitedLines.includes(i) && !isActive;
          const cls = [
            'pseudocode-line',
            isActive ? 'pseudocode-line-active' : '',
            isVisited ? 'pseudocode-line-visited' : '',
          ].filter(Boolean).join(' ');
          return (
            <div key={i} className={cls}>
              <span className="pseudocode-gutter">
                {isVisited ? <span className="pseudocode-check">{'\u2713'}</span>
                           : <span className="pseudocode-lineno">{i + 1}</span>}
              </span>
              <span className="pseudocode-text">{line.text}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
