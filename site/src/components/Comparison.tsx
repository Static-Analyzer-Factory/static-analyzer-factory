import { motion } from 'motion/react';
import { TOOLS, TEASER_ROWS, type CellKind } from '../data/comparison';

const cellClass = (kind: CellKind, isSaf: boolean) => {
  const base = 'comparison-cell';
  const kindClass = `comparison-cell--${kind}`;
  return isSaf ? `${base} ${kindClass} comparison-cell--saf` : `${base} ${kindClass}`;
};

export default function Comparison() {
  return (
    <section className="comparison-teaser" id="comparison-teaser">
      <div className="section-container">
        <motion.h2
          className="section-title"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
        >
          How SAF compares
        </motion.h2>
        <motion.p
          className="comparison-teaser-lede"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          Five design choices that distinguish SAF from the closest LLVM-IR
          program-analysis frameworks and the source-level analyzers you may already
          know. See the full comparison for the complete capability matrix.
        </motion.p>
        <motion.div
          className="comparison-table-wrap"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <table className="comparison-table">
            <thead>
              <tr>
                <th scope="col" className="comparison-dimension">Dimension</th>
                {TOOLS.map((t) => (
                  <th
                    key={t.key}
                    scope="col"
                    className={t.key === 'saf' ? 'comparison-cell--saf' : undefined}
                  >
                    {t.name}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {TEASER_ROWS.map((row) => (
                <tr key={row.dimension}>
                  <th scope="row" className="comparison-dimension">{row.dimension}</th>
                  {TOOLS.map((t) => {
                    const v = row.values[t.key];
                    return (
                      <td key={t.key} className={cellClass(v.kind, t.key === 'saf')}>
                        {v.text}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </motion.div>
        <motion.div
          className="comparison-teaser-cta"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <a href="#comparison" className="cta cta-primary">See the full comparison</a>
        </motion.div>
      </div>
    </section>
  );
}
