import { motion } from 'motion/react';

const highlights = [
  { label: 'Multi-LLVM', detail: 'LLVM 18 + 22 in Docker' },
  { label: 'Python + Rust', detail: 'Scriptable; native or in-browser' },
  { label: 'Deterministic', detail: 'Byte-identical outputs' },
  { label: 'Open Source', detail: 'MIT License' },
];

export default function TechHighlights() {
  return (
    <section className="tech-highlights">
      <div className="section-container">
        <div className="highlights-strip">
          {highlights.map((h, i) => (
            <motion.div
              key={h.label}
              className="highlight-item"
              initial={{ opacity: 0, scale: 0.9 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true, margin: '-40px' }}
              transition={{ delay: i * 0.1, duration: 0.4, ease: 'easeOut' as const }}
            >
              <span className="highlight-label">{h.label}</span>
              <span className="highlight-detail">{h.detail}</span>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
