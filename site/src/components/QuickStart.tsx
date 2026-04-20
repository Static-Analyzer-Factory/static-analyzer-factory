import { motion } from 'motion/react';
import { useState } from 'react';

const COMMANDS = [
  'git clone https://github.com/Static-Analyzer-Factory/static-analyzer-factory.git && cd static-analyzer-factory',
  'make shell',
];

const FULL_INSTALL = COMMANDS.join(' && ');

export default function QuickStart() {
  const [copied, setCopied] = useState(false);

  const onCopy = () => {
    navigator.clipboard.writeText(FULL_INSTALL).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };

  return (
    <section className="quickstart" id="quickstart">
      <div className="section-container">
        <motion.div
          className="quickstart-terminal"
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-60px' }}
          transition={{ duration: 0.5 }}
        >
          <div className="quickstart-header">
            <span className="quickstart-label">QUICK INSTALL:</span>
            <button
              type="button"
              className="quickstart-copy"
              onClick={onCopy}
              aria-label="Copy install commands"
            >
              {copied ? 'Copied' : 'Copy'}
            </button>
          </div>

          <div className="quickstart-commands">
            {COMMANDS.map((cmd, i) => (
              <div key={cmd}>
                <div className="quickstart-command">
                  <span className="quickstart-prompt">$</span>
                  <div className="quickstart-cmd-text">
                    <code>{cmd}</code>
                  </div>
                </div>
                {i < COMMANDS.length - 1 && (
                  <div className="quickstart-separator">&amp;&amp;</div>
                )}
              </div>
            ))}
          </div>

          <p className="quickstart-note">
            Docker does the rest — LLVM 18 and the <code>saf</code> Python SDK auto-install on first run.
            Then try <code>saf --help</code>.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
