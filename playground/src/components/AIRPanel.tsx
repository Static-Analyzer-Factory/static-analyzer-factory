interface AIRPanelProps {
  airJSON: object | null;
}

export function AIRPanel({ airJSON }: AIRPanelProps) {
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', borderTop: '1px solid #f1eeeb' }}>
      <div className="panel-header">
        <h2>AIR JSON</h2>
      </div>
      <div className="panel-content">
        {airJSON ? (
          <pre className="json-viewer">{JSON.stringify(airJSON, null, 2)}</pre>
        ) : (
          <div className="placeholder">
            <p>AIR representation will appear here after analysis pipeline is connected</p>
          </div>
        )}
      </div>
    </div>
  );
}
