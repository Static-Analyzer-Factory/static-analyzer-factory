import { motion } from 'motion/react';

/** Static CFG mockup SVG — looks like a control-flow graph from the playground. */
function CfgPreview() {
  return (
    <svg viewBox="0 0 320 260" fill="none" xmlns="http://www.w3.org/2000/svg" className="feature-preview">
      {/* entry */}
      <rect x="120" y="8" width="80" height="28" rx="4" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="160" y="26" textAnchor="middle" fill="#a0aec0" fontSize="11" fontFamily="monospace">entry</text>
      {/* edge entry → while.cond */}
      <line x1="160" y1="36" x2="160" y2="56" stroke="#dbd6d0" strokeWidth="1" />
      <polygon points="156,54 164,54 160,60" fill="#dbd6d0" />
      {/* while.cond */}
      <rect x="110" y="60" width="100" height="28" rx="4" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="160" y="78" textAnchor="middle" fill="#a0aec0" fontSize="11" fontFamily="monospace">while.cond</text>
      {/* branches */}
      <line x1="130" y1="88" x2="70" y2="118" stroke="#3d9b8f" strokeWidth="1" />
      <polygon points="66,116 74,116 70,122" fill="#3d9b8f" />
      <line x1="190" y1="88" x2="250" y2="118" stroke="#c75050" strokeWidth="1" />
      <polygon points="246,116 254,116 250,122" fill="#c75050" />
      {/* if.then */}
      <rect x="30" y="122" width="80" height="28" rx="4" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="70" y="140" textAnchor="middle" fill="#a0aec0" fontSize="11" fontFamily="monospace">if.then</text>
      {/* if.else */}
      <rect x="210" y="122" width="80" height="28" rx="4" fill="#faf9f7" stroke="#c75050" strokeWidth="1.5" />
      <text x="250" y="140" textAnchor="middle" fill="#a0aec0" fontSize="11" fontFamily="monospace">if.else</text>
      {/* merge → while.body */}
      <line x1="70" y1="150" x2="160" y2="180" stroke="#dbd6d0" strokeWidth="1" />
      <line x1="250" y1="150" x2="160" y2="180" stroke="#dbd6d0" strokeWidth="1" />
      <polygon points="156,178 164,178 160,184" fill="#dbd6d0" />
      {/* while.body */}
      <rect x="110" y="184" width="100" height="28" rx="4" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="160" y="202" textAnchor="middle" fill="#a0aec0" fontSize="11" fontFamily="monospace">while.body</text>
      {/* back-edge */}
      <path d="M 210 198 Q 280 198 280 74 Q 280 74 210 74" stroke="#c49a3c" strokeWidth="1" strokeDasharray="4 3" fill="none" />
      <polygon points="212,70 212,78 206,74" fill="#c49a3c" />
      {/* exit */}
      <line x1="160" y1="212" x2="160" y2="232" stroke="#dbd6d0" strokeWidth="1" />
      <polygon points="156,230 164,230 160,236" fill="#dbd6d0" />
      <rect x="120" y="236" width="80" height="22" rx="4" fill="#faf9f7" stroke="#dbd6d0" strokeWidth="1.5" />
      <text x="160" y="251" textAnchor="middle" fill="#718096" fontSize="10" fontFamily="monospace">return</text>
    </svg>
  );
}

/** Static PTA mockup SVG — looks like a points-to graph. */
function PtaPreview() {
  return (
    <svg viewBox="0 0 320 260" fill="none" xmlns="http://www.w3.org/2000/svg" className="feature-preview">
      {/* Variables */}
      <circle cx="60" cy="50" r="20" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="60" y="54" textAnchor="middle" fill="#c4b5fd" fontSize="12" fontFamily="monospace">p</text>
      <circle cx="160" cy="50" r="20" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="160" y="54" textAnchor="middle" fill="#c4b5fd" fontSize="12" fontFamily="monospace">q</text>
      <circle cx="260" cy="50" r="20" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="260" y="54" textAnchor="middle" fill="#c4b5fd" fontSize="12" fontFamily="monospace">r</text>
      {/* Heap objects */}
      <rect x="30" y="150" width="60" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="60" y="169" textAnchor="middle" fill="#6ee7b7" fontSize="10" fontFamily="monospace">obj₁</text>
      <rect x="130" y="150" width="60" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="160" y="169" textAnchor="middle" fill="#6ee7b7" fontSize="10" fontFamily="monospace">obj₂</text>
      <rect x="230" y="150" width="60" height="30" rx="6" fill="#faf9f7" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="260" y="169" textAnchor="middle" fill="#6ee7b7" fontSize="10" fontFamily="monospace">obj₃</text>
      {/* Points-to edges */}
      <line x1="60" y1="70" x2="60" y2="150" stroke="#3d9b8f" strokeWidth="1.5" />
      <polygon points="56,148 64,148 60,154" fill="#3d9b8f" />
      <line x1="160" y1="70" x2="60" y2="150" stroke="#3d9b8f" strokeWidth="1.5" strokeDasharray="4 3" />
      <polygon points="56,148 64,148 60,154" fill="#3d9b8f" />
      <line x1="160" y1="70" x2="160" y2="150" stroke="#3d9b8f" strokeWidth="1.5" />
      <polygon points="156,148 164,148 160,154" fill="#3d9b8f" />
      <line x1="260" y1="70" x2="260" y2="150" stroke="#3d9b8f" strokeWidth="1.5" />
      <polygon points="256,148 264,148 260,154" fill="#3d9b8f" />
      {/* Field edges between objects */}
      <path d="M 90 165 Q 110 200 130 165" stroke="#c49a3c" strokeWidth="1" strokeDasharray="4 3" fill="none" />
      <polygon points="128,161 128,169 134,165" fill="#c49a3c" />
      <path d="M 190 165 Q 210 200 230 165" stroke="#c49a3c" strokeWidth="1" strokeDasharray="4 3" fill="none" />
      <polygon points="228,161 228,169 234,165" fill="#c49a3c" />
      {/* Legend */}
      <text x="60" y="220" fill="#718096" fontSize="9" fontFamily="monospace">q aliases p → obj₁</text>
      <line x1="40" y1="235" x2="60" y2="235" stroke="#3d9b8f" strokeWidth="1.5" />
      <text x="65" y="238" fill="#718096" fontSize="9" fontFamily="monospace">points-to</text>
      <line x1="140" y1="235" x2="160" y2="235" stroke="#c49a3c" strokeWidth="1" strokeDasharray="4 3" />
      <text x="165" y="238" fill="#718096" fontSize="9" fontFamily="monospace">field</text>
    </svg>
  );
}

const features = [
  {
    title: 'Visualize',
    description: 'See how programs actually work. Explore control flow graphs, call graphs, def-use chains, and value-flow graphs interactively in the browser playground.',
    icon: (
      <svg viewBox="0 0 40 40" fill="none" className="feature-icon">
        <rect x="4" y="4" width="12" height="8" rx="2" stroke="#3d9b8f" strokeWidth="2" />
        <rect x="24" y="4" width="12" height="8" rx="2" stroke="#3d9b8f" strokeWidth="2" />
        <rect x="14" y="28" width="12" height="8" rx="2" stroke="#3d9b8f" strokeWidth="2" />
        <line x1="10" y1="12" x2="20" y2="28" stroke="#3d9b8f" strokeWidth="1.5" />
        <line x1="30" y1="12" x2="20" y2="28" stroke="#3d9b8f" strokeWidth="1.5" />
      </svg>
    ),
    preview: <CfgPreview />,
  },
  {
    title: 'Analyze',
    description: 'Whole-program pointer analysis, taint tracking, and value-flow reasoning over LLVM IR. Context-sensitive (k-CFA), flow-sensitive, and demand-driven variants.',
    icon: (
      <svg viewBox="0 0 40 40" fill="none" className="feature-icon">
        <circle cx="12" cy="12" r="6" stroke="#3d9b8f" strokeWidth="2" />
        <circle cx="28" cy="12" r="6" stroke="#3d9b8f" strokeWidth="2" />
        <circle cx="20" cy="30" r="6" stroke="#c49a3c" strokeWidth="2" />
        <line x1="16" y1="16" x2="24" y2="26" stroke="#3d9b8f" strokeWidth="1.5" strokeDasharray="3 2" />
        <line x1="24" y1="16" x2="20" y2="24" stroke="#3d9b8f" strokeWidth="1.5" strokeDasharray="3 2" />
      </svg>
    ),
    preview: <PtaPreview />,
  },
  {
    title: 'Build',
    description: 'Write custom analyzers in Python over a Rust core. Detect use-after-free, taint flows, and null dereferences with a few lines of code — or experiment with new algorithms.',
    icon: (
      <svg viewBox="0 0 40 40" fill="none" className="feature-icon">
        <polyline points="8,12 16,20 8,28" stroke="#3d9b8f" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
        <line x1="20" y1="28" x2="34" y2="28" stroke="#3d9b8f" strokeWidth="2" strokeLinecap="round" />
      </svg>
    ),
    code: `import saf

result = saf.analyze()
vf = result.valueflow()

for edge in vf.edges():
    if edge.edge_type == "Store":
        saf.report(
            node_id=edge.src,
            severity="high",
            message="Potential UAF"
        )`,
  },
];

const cardVariants = {
  hidden: { opacity: 0, y: 40 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { delay: i * 0.15, duration: 0.5, ease: 'easeOut' as const },
  }),
};

export default function Features() {
  return (
    <section className="features" id="features">
      <div className="section-container">
        <motion.h2
          className="section-title"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
        >
          What the framework gives you
        </motion.h2>
        <div className="features-grid">
          {features.map((f, i) => (
            <motion.div
              key={f.title}
              className="feature-card"
              variants={cardVariants}
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true, margin: '-60px' }}
              custom={i}
            >
              <div className="feature-header">
                {f.icon}
                <h3>{f.title}</h3>
              </div>
              <p>{f.description}</p>
              {f.preview && (
                <div className="feature-widget">
                  {f.preview}
                </div>
              )}
              {f.code && (
                <pre className="feature-code"><code>{f.code}</code></pre>
              )}
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
