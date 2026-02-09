import { useCallback, useEffect, useState } from 'react';
import {
  loadSpecManifest,
  loadSpec,
  getCategories,
  type SpecManifest,
  type SpecEntry,
} from '@saf/web-shared/analysis';

export function SpecViewer() {
  const [manifest, setManifest] = useState<SpecManifest | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedSpec, setSelectedSpec] = useState<SpecEntry | null>(null);
  const [yamlContent, setYamlContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSpecManifest()
      .then(setManifest)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : String(err)),
      );
  }, []);

  const handleCategoryClick = useCallback((category: string) => {
    setSelectedCategory((prev) => (prev === category ? null : category));
    setSelectedSpec(null);
    setYamlContent(null);
  }, []);

  const handleSpecClick = useCallback((entry: SpecEntry) => {
    setSelectedSpec(entry);
    setYamlContent(null);
    setLoading(true);
    loadSpec(entry.path)
      .then((yaml) => {
        setYamlContent(yaml);
        setLoading(false);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
        setLoading(false);
      });
  }, []);

  if (error) {
    return (
      <div className="spec-viewer">
        <div className="spec-error">Failed to load specs: {error}</div>
      </div>
    );
  }

  if (!manifest) {
    return (
      <div className="spec-viewer">
        <div className="placeholder">
          <p>Loading specs...</p>
        </div>
      </div>
    );
  }

  const categories = getCategories(manifest);
  const specsInCategory = selectedCategory
    ? manifest.specs.filter((s) => s.category === selectedCategory)
    : [];

  return (
    <div className="spec-viewer">
      <div className="spec-sidebar">
        <div className="spec-categories">
          {categories.map((cat) => (
            <div key={cat} className="spec-category-group">
              <button
                className={`spec-category ${selectedCategory === cat ? 'active' : ''}`}
                onClick={() => handleCategoryClick(cat)}
              >
                <span className="spec-category-arrow">
                  {selectedCategory === cat ? '\u25BE' : '\u25B8'}
                </span>
                {cat}
                <span className="spec-count">
                  {manifest.specs.filter((s) => s.category === cat).length}
                </span>
              </button>
              {selectedCategory === cat && (
                <div className="spec-list">
                  {specsInCategory.map((entry) => (
                    <button
                      key={entry.path}
                      className={`spec-item ${selectedSpec?.path === entry.path ? 'active' : ''}`}
                      onClick={() => handleSpecClick(entry)}
                    >
                      {entry.name}.yaml
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
      <div className="spec-content">
        {loading && (
          <div className="placeholder">
            <p>Loading...</p>
          </div>
        )}
        {!loading && yamlContent && selectedSpec && (
          <div className="spec-yaml-view">
            <div className="spec-yaml-header">
              {selectedSpec.category}/{selectedSpec.name}.yaml
            </div>
            <pre className="spec-yaml">{yamlContent}</pre>
          </div>
        )}
        {!loading && !yamlContent && (
          <div className="placeholder">
            <p>
              Select a category and spec file to view function specifications
              used by SAF analyses (PTA, taint, nullness)
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
