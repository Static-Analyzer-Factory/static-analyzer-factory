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
          <motion.img
            src="./saf-logo.png"
            alt="Static Analyzer Factory logo"
            className="hero-logo"
            width={112}
            height={112}
            initial={{ opacity: 0, scale: 0.85 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, ease: 'easeOut' as const }}
          />
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
          </motion.div>
        </div>
        <div className="hero-graphic">
          <CfgGraphic />
        </div>
      </div>
    </section>
  );
}
