export default function PlaygroundEmbed({ url }: { url: string }) {
  const base = import.meta.env.BASE_URL;
  const fullUrl = `${base}../playground/${url}`;
  return (
    <div className="playground-embed">
      <div className="playground-embed-header">
        <span>Interactive Playground</span>
        <a href={fullUrl} target="_blank" rel="noopener noreferrer">Open in Playground &#8599;</a>
      </div>
      <iframe
        src={fullUrl}
        title="SAF Playground"
        sandbox="allow-scripts allow-same-origin"
        style={{ width: '100%', height: '400px', border: 'none', borderRadius: '8px' }}
      />
    </div>
  );
}
