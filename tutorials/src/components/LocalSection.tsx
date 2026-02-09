import { useState } from 'react';

interface LocalSectionProps {
  cmd?: string;
  script?: string;
  tutorialId?: string;
}

export default function LocalSection({ cmd, script, tutorialId }: LocalSectionProps) {
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    if (cmd) {
      navigator.clipboard.writeText(cmd);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const base = import.meta.env.BASE_URL;

  return (
    <div className="local-section">
      <button className="local-toggle" onClick={() => setOpen(!open)}>
        <span className={`arrow ${open ? 'open' : ''}`}>&#9654;</span>
        Run Locally
      </button>
      {open && (
        <div className="local-content">
          {cmd && (
            <div className="local-cmd">
              <div className="local-cmd-header">
                <span>Terminal</span>
                <button onClick={handleCopy} className="copy-btn">
                  {copied ? 'Copied!' : 'Copy'}
                </button>
              </div>
              <pre><code>{cmd}</code></pre>
            </div>
          )}
          {script && tutorialId && (
            <a
              href={`${base}content/${tutorialId}/scripts/${script}`}
              download
              className="download-link"
            >
              Download {script}
            </a>
          )}
        </div>
      )}
    </div>
  );
}
