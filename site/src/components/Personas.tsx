import { motion } from 'motion/react';

const personas = [
  {
    title: 'Students & Learners',
    description: 'Interactive tutorials on real vulnerabilities. Visualize CFG, value-flow, and points-to analysis step by step.',
    link: './tutorials/',
    linkText: 'Start Learning',
    color: '#3d9b8f',
  },
  {
    title: 'Security Researchers',
    description: 'Author custom analyzers in Python. Detect use-after-free, command injection, and more.',
    link: './playground/',
    linkText: 'Open Playground',
    color: '#3d9b8f',
  },
  {
    title: 'AI Agent Developers',
    description: 'Schema-driven API for programmatic analysis. Build agents that reason about code structure and data flow.',
    link: './docs/api-reference/python-sdk.html',
    linkText: 'View API Docs',
    color: '#c49a3c',
  },
];

export default function Personas() {
  return (
    <section className="personas">
      <div className="section-container">
        <motion.h2
          className="section-title"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
        >
          Built for You
        </motion.h2>
        <div className="personas-grid">
          {personas.map((p, i) => (
            <motion.a
              key={p.title}
              href={p.link}
              className="persona-card"
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ delay: i * 0.12, duration: 0.5, ease: 'easeOut' as const }}
              whileHover={{ y: -6, boxShadow: `0 12px 40px ${p.color}22` }}
              style={{ '--persona-accent': p.color } as React.CSSProperties}
            >
              <div className="persona-bar" />
              <h3>{p.title}</h3>
              <p>{p.description}</p>
              <span className="persona-link">{p.linkText} &rarr;</span>
            </motion.a>
          ))}
        </div>
      </div>
    </section>
  );
}
