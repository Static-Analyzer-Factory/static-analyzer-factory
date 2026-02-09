import { useState, useEffect } from 'react';
import type { TutorialMeta, TutorialStep } from '../content/types';
import { CATEGORIES, SUBCATEGORIES, TUTORIALS } from '../content/registry';
import './Sidebar.css';

interface SidebarProps {
  currentTutorialId: string | null;
  currentStep: number;
  steps: TutorialStep[];
  onNavigate: (id: string | null, step?: number) => void;
  isOpen: boolean;
  onToggle: () => void;
}

export default function Sidebar({
  currentTutorialId,
  currentStep,
  steps,
  onNavigate,
  isOpen,
  onToggle,
}: SidebarProps) {
  const currentTutorial = currentTutorialId
    ? TUTORIALS.find((t) => t.id === currentTutorialId)
    : null;

  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(() => {
    const initial = new Set<string>();
    if (currentTutorial) {
      initial.add(currentTutorial.category);
    }
    return initial;
  });

  // Auto-expand the current tutorial's category
  useEffect(() => {
    if (currentTutorial) {
      setExpandedCategories((prev) => {
        const next = new Set(prev);
        next.add(currentTutorial.category);
        return next;
      });
    }
  }, [currentTutorial]);

  const toggleCategory = (catId: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(catId)) {
        next.delete(catId);
      } else {
        next.add(catId);
      }
      return next;
    });
  };

  const difficultyColor = (d: string) => {
    switch (d) {
      case 'beginner': return '#3d9b8f';
      case 'intermediate': return '#c49a3c';
      case 'advanced': return '#c75050';
      default: return '#a0aec0';
    }
  };

  const renderTutorialItem = (t: TutorialMeta) => {
    const isActive = t.id === currentTutorialId;
    return (
      <li key={t.id}>
        <button
          className={`sidebar-tutorial-btn ${isActive ? 'active' : ''}`}
          onClick={() => onNavigate(t.id, 0)}
        >
          <span className="tutorial-name">{t.title}</span>
          <span
            className="difficulty-badge"
            style={{ backgroundColor: difficultyColor(t.difficulty) }}
          >
            {t.difficulty}
          </span>
        </button>

        {/* Step list for active tutorial */}
        {isActive && steps.length > 0 && (
          <ol className="sidebar-step-list">
            {steps.map((s, idx) => (
              <li key={idx}>
                <button
                  className={`sidebar-step-btn ${idx === currentStep ? 'active' : ''}`}
                  onClick={() => onNavigate(t.id, idx)}
                >
                  {s.title}
                </button>
              </li>
            ))}
          </ol>
        )}
      </li>
    );
  };

  return (
    <>
      {/* Mobile overlay */}
      {isOpen && <div className="sidebar-overlay" onClick={onToggle} />}

      <aside className={`sidebar ${isOpen ? 'sidebar-open' : ''}`}>
        <div className="sidebar-header">
          <button className="sidebar-title" onClick={() => onNavigate(null)}>
            SAF Tutorials
          </button>
          <button className="sidebar-close" onClick={onToggle}>
            &#10005;
          </button>
        </div>

        <nav className="sidebar-nav">
          {CATEGORIES.map((cat) => {
            const catTutorials = TUTORIALS.filter((t) => t.category === cat.id);
            const isExpanded = expandedCategories.has(cat.id);

            return (
              <div key={cat.id} className="sidebar-category">
                <button
                  className="sidebar-category-btn"
                  onClick={() => toggleCategory(cat.id)}
                >
                  <span className={`arrow ${isExpanded ? 'open' : ''}`}>&#9654;</span>
                  <span className="sidebar-category-icon">{cat.icon}</span>
                  <span>{cat.title}</span>
                </button>

                {isExpanded && cat.id === 'algorithms' ? (
                  <div className="sidebar-subcategories">
                    {SUBCATEGORIES.map((sub) => {
                      const subTutorials = catTutorials.filter((t) => t.subcategory === sub.id);
                      if (subTutorials.length === 0) return null;
                      return (
                        <div key={sub.id} className="sidebar-subcategory">
                          <span className="sidebar-subcategory-label">{sub.title}</span>
                          <ul className="sidebar-tutorial-list">
                            {subTutorials.map((t) => renderTutorialItem(t))}
                          </ul>
                        </div>
                      );
                    })}
                  </div>
                ) : isExpanded ? (
                  <ul className="sidebar-tutorial-list">
                    {catTutorials.map((t) => renderTutorialItem(t))}
                  </ul>
                ) : null}
              </div>
            );
          })}
        </nav>
      </aside>
    </>
  );
}
