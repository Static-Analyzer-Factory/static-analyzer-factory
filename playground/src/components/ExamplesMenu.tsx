import type { Example } from '../examples';

interface ExamplesMenuProps {
  examples: Example[];
  onSelect: (index: number) => void;
  disabled: boolean;
}

export function ExamplesMenu({ examples, onSelect, disabled }: ExamplesMenuProps) {
  return (
    <select
      onChange={(e) => onSelect(Number(e.target.value))}
      disabled={disabled}
      defaultValue=""
    >
      <option value="" disabled>
        Load example...
      </option>
      {examples.map((ex, i) => (
        <option key={ex.name} value={i}>
          {ex.name}
        </option>
      ))}
    </select>
  );
}
