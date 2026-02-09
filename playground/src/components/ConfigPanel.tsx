/** Compact settings bar for analysis configuration. */

export interface AnalysisConfig {
  mem2reg: boolean;
  vf_mode: 'fast' | 'precise';
  pta_solver: 'worklist' | 'datalog';
  pta_max_iterations: number;
  max_refinement_iters: number;
  enable_specs: boolean;
}

export const defaultConfig: AnalysisConfig = {
  mem2reg: true,
  vf_mode: 'precise',
  pta_solver: 'worklist',
  pta_max_iterations: 10_000,
  max_refinement_iters: 3,
  enable_specs: true,
};

interface ConfigPanelProps {
  config: AnalysisConfig;
  onChange: (config: AnalysisConfig) => void;
  disabled: boolean;
}

export function ConfigPanel({ config, onChange, disabled }: ConfigPanelProps) {
  return (
    <div className="config-bar">
      <label className="config-field config-checkbox">
        <input
          type="checkbox"
          checked={config.mem2reg}
          onChange={(e) =>
            onChange({ ...config, mem2reg: e.target.checked })
          }
          disabled={disabled}
        />
        <span className="config-label">mem2reg</span>
      </label>

      <label className="config-field">
        <span className="config-label">VF Mode</span>
        <select
          value={config.vf_mode}
          onChange={(e) =>
            onChange({ ...config, vf_mode: e.target.value as 'fast' | 'precise' })
          }
          disabled={disabled}
        >
          <option value="precise">Precise</option>
          <option value="fast">Fast</option>
        </select>
      </label>

      <label className="config-field">
        <span className="config-label">PTA Solver</span>
        <select
          value={config.pta_solver}
          onChange={(e) =>
            onChange({ ...config, pta_solver: e.target.value as 'worklist' | 'datalog' })
          }
          disabled={disabled}
        >
          <option value="worklist">Worklist</option>
          <option value="datalog">Datalog</option>
        </select>
      </label>

      <label className="config-field">
        <span className="config-label">PTA Iters</span>
        <input
          type="number"
          value={config.pta_max_iterations}
          onChange={(e) =>
            onChange({ ...config, pta_max_iterations: Number(e.target.value) || 10_000 })
          }
          disabled={disabled}
          min={100}
          max={1_000_000}
          step={1000}
        />
      </label>

      <label className="config-field">
        <span className="config-label">CG Iters</span>
        <input
          type="number"
          value={config.max_refinement_iters}
          onChange={(e) =>
            onChange({ ...config, max_refinement_iters: Number(e.target.value) || 3 })
          }
          disabled={disabled}
          min={1}
          max={20}
        />
      </label>

      <label className="config-field config-checkbox">
        <input
          type="checkbox"
          checked={config.enable_specs}
          onChange={(e) =>
            onChange({ ...config, enable_specs: e.target.checked })
          }
          disabled={disabled}
        />
        <span className="config-label">Specs</span>
      </label>
    </div>
  );
}
