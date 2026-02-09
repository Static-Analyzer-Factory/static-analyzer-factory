type Status = 'idle' | 'compiling' | 'parsing' | 'converting' | 'analyzing' | 'ready' | 'error';

const STATUS_TEXT: Record<Status, string> = {
  idle: 'Ready',
  compiling: 'Compiling via Compiler Explorer...',
  parsing: 'Parsing LLVM IR with tree-sitter...',
  converting: 'Converting CST to AIR...',
  analyzing: 'Running SAF analysis...',
  ready: 'Analysis complete',
  error: 'Error',
};

interface StatusBarProps {
  status: Status;
  elapsed: number | null;
}

export function StatusBar({ status, elapsed }: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-indicator">
        <div className={`status-dot ${status}`} />
        <span>{STATUS_TEXT[status]}</span>
      </div>
      {elapsed !== null && (
        <span>{elapsed < 1000 ? `${Math.round(elapsed)}ms` : `${(elapsed / 1000).toFixed(1)}s`}</span>
      )}
    </div>
  );
}
