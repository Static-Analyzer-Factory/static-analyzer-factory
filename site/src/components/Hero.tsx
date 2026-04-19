import { motion } from 'motion/react';

/* Animated CFG-like SVG — draws paths sequentially */
function CfgGraphic() {
  const draw = {
    hidden: { pathLength: 0, opacity: 0 },
    visible: (i: number) => ({
      pathLength: 1,
      opacity: 1,
      transition: { pathLength: { delay: 0.3 + i * 0.15, duration: 0.8, ease: 'easeInOut' as const }, opacity: { delay: 0.3 + i * 0.15, duration: 0.2 } },
    }),
  };

  const nodeAppear = {
    hidden: { scale: 0, opacity: 0 },
    visible: (i: number) => ({
      scale: 1,
      opacity: 1,
      transition: { delay: 0.2 + i * 0.12, duration: 0.4, ease: 'backOut' as const },
    }),
  };

  return (
    <motion.svg
      viewBox="0 0 280 320"
      className="hero-cfg"
      initial="hidden"
      animate="visible"
      aria-label="Animated control flow graph"
    >
      {/* Edges */}
      <motion.line x1="140" y1="50" x2="80" y2="120" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={0} />
      <motion.line x1="140" y1="50" x2="200" y2="120" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={1} />
      <motion.line x1="80" y1="120" x2="80" y2="200" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={2} />
      <motion.line x1="200" y1="120" x2="200" y2="200" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={3} />
      <motion.line x1="80" y1="200" x2="140" y2="270" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={4} />
      <motion.line x1="200" y1="200" x2="140" y2="270" stroke="#3d9b8f" strokeWidth="2" variants={draw} custom={5} />

      {/* Nodes */}
      <motion.rect x="115" y="25" width="50" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="2" variants={nodeAppear} custom={0} />
      <motion.rect x="55" y="105" width="50" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="2" variants={nodeAppear} custom={1} />
      <motion.rect x="175" y="105" width="50" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="2" variants={nodeAppear} custom={2} />
      <motion.rect x="55" y="185" width="50" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="2" variants={nodeAppear} custom={3} />
      <motion.rect x="175" y="185" width="50" height="30" rx="6" fill="#faf9f7" stroke="#c75050" strokeWidth="2" variants={nodeAppear} custom={4} />
      <motion.rect x="115" y="255" width="50" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="2" variants={nodeAppear} custom={5} />

      {/* Labels */}
      <motion.text x="140" y="45" textAnchor="middle" fill="#2c2c2e" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={0}>entry</motion.text>
      <motion.text x="80" y="125" textAnchor="middle" fill="#2c2c2e" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={1}>bb1</motion.text>
      <motion.text x="200" y="125" textAnchor="middle" fill="#2c2c2e" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={2}>bb2</motion.text>
      <motion.text x="80" y="205" textAnchor="middle" fill="#2c2c2e" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={3}>bb3</motion.text>
      <motion.text x="200" y="205" textAnchor="middle" fill="#c75050" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={4}>bug</motion.text>
      <motion.text x="140" y="275" textAnchor="middle" fill="#2c2c2e" fontSize="11" fontFamily="monospace" variants={nodeAppear} custom={5}>exit</motion.text>
    </motion.svg>
  );
}

export default function Hero() {
  return (
    <section className="hero">
      <div className="hero-content">
        <div className="hero-text">
          <motion.h1
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: 'easeOut' as const }}
          >
            Static Analyzer <span className="accent">Factory</span>
          </motion.h1>
          <motion.p
            className="hero-tagline"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.15, ease: 'easeOut' as const }}
          >
            Build program analysis tools. Understand code deeply.
          </motion.p>
          <motion.p
            className="hero-sub"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.25, ease: 'easeOut' as const }}
          >
            Browser-based static analysis powered by Rust + WebAssembly.
            Visualize control flow, pointer aliasing, and value-flow graphs instantly.
          </motion.p>
          <motion.div
            className="hero-ctas"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.4, ease: 'easeOut' as const }}
          >
            <a href="./playground/" className="cta cta-primary">Try the Playground</a>
            <a href="./docs/" className="cta cta-secondary">Read the Docs</a>
            <a
              href="https://github.com/Static-Analyzer-Factory/static-analyzer-factory"
              target="_blank"
              rel="noopener noreferrer"
              className="cta cta-secondary"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" className="cta-github-icon" aria-hidden="true">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.3 3.438 9.8 8.205 11.387.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.694.825.576C20.565 21.795 24 17.295 24 12 24 5.37 18.63 0 12 0z" />
              </svg>
              View on GitHub
            </a>
          </motion.div>
        </div>
        <div className="hero-graphic">
          <CfgGraphic />
        </div>
      </div>
    </section>
  );
}
