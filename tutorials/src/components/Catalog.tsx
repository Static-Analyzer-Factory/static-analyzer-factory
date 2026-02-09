import { useState } from 'react';
import type { Difficulty } from '../content/types';
import { CATEGORIES, TUTORIALS } from '../content/registry';
import './Catalog.css';

interface CatalogProps {
  onNavigate: (tutorialId: string) => void;
}

export default function Catalog({ onNavigate }: CatalogProps) {
  const [filterCategory, setFilterCategory] = useState<string | null>(null);
  const [filterDifficulty, setFilterDifficulty] = useState<Difficulty | 'all'>('all');

  const filtered = TUTORIALS.filter((t) => {
    if (filterCategory && t.category !== filterCategory) return false;
    if (filterDifficulty !== 'all' && t.difficulty !== filterDifficulty) return false;
    return true;
  });

  const difficultyColor = (d: string) => {
    switch (d) {
      case 'beginner': return '#3d9b8f';
      case 'intermediate': return '#c49a3c';
      case 'advanced': return '#c75050';
      default: return '#a0aec0';
    }
  };

  const modeLabel = (m: string) => {
    switch (m) {
      case 'browser': return 'Browser';
      case 'local': return 'Local';
      case 'both': return 'Both';
      default: return m;
    }
  };

  return (
    <div className="catalog">
      {/* Hero */}
      <section className="catalog-hero">
        <h1>Learn Static Analysis</h1>
        <p>
          Interactive tutorials for mastering program analysis with the SAF toolkit.
          Explore control flow, value flow, pointer analysis, and more.
        </p>
      </section>

      {/* Learning path cards */}
      <section className="catalog-paths">
        <h2>Learning Paths</h2>
        <div className="path-cards">
          {CATEGORIES.map((cat) => (
            <button
              key={cat.id}
              className={`path-card ${filterCategory === cat.id ? 'active' : ''}`}
              onClick={() => setFilterCategory(filterCategory === cat.id ? null : cat.id)}
            >
              <span className="path-icon">{cat.icon}</span>
              <span className="path-title">{cat.title}</span>
              <span className="path-desc">{cat.description}</span>
            </button>
          ))}
        </div>
      </section>

      {/* Filter bar */}
      <section className="catalog-filters">
        <label>
          Difficulty:
          <select
            value={filterDifficulty}
            onChange={(e) => setFilterDifficulty(e.target.value as Difficulty | 'all')}
          >
            <option value="all">All</option>
            <option value="beginner">Beginner</option>
            <option value="intermediate">Intermediate</option>
            <option value="advanced">Advanced</option>
          </select>
        </label>
        {filterCategory && (
          <button className="clear-filter" onClick={() => setFilterCategory(null)}>
            Clear category filter
          </button>
        )}
      </section>

      {/* Tutorial grid */}
      <section className="catalog-grid">
        {filtered.length === 0 ? (
          <p className="no-results">No tutorials match your filters.</p>
        ) : (
          filtered.map((t) => (
            <button
              key={t.id}
              className="tutorial-card"
              onClick={() => onNavigate(t.id)}
            >
              <div className="card-badges">
                <span
                  className="badge difficulty"
                  style={{ backgroundColor: difficultyColor(t.difficulty) }}
                >
                  {t.difficulty}
                </span>
                <span className="badge mode">{modeLabel(t.mode)}</span>
              </div>
              <h3>{t.title}</h3>
              <p>{t.description}</p>
            </button>
          ))
        )}
      </section>
    </div>
  );
}
