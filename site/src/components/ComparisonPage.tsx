import { Fragment } from 'react';
import { motion } from 'motion/react';
import { TOOLS, ROWS, type CellKind } from '../data/comparison';

const cellClass = (kind: CellKind, isSaf: boolean) => {
  const base = 'comparison-cell';
  const kindClass = `comparison-cell--${kind}`;
  return isSaf ? `${base} ${kindClass} comparison-cell--saf` : `${base} ${kindClass}`;
};

const capabilityRows = ROWS.filter((r) => r.group === 'capability');
const differentiationRows = ROWS.filter((r) => r.group === 'differentiation');

function ComparisonGroup({
  title,
  intro,
  rows,
}: {
  title: string;
  intro: string;
  rows: typeof ROWS;
}) {
  return (
    <div className="comparison-group">
      <h2 className="comparison-group-title">{title}</h2>
      <p className="comparison-group-intro">{intro}</p>
      <div className="comparison-table-wrap">
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
            {rows.map((row) => (
              <Fragment key={row.dimension}>
                <tr>
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
                {row.note && (
                  <tr className="comparison-note-row">
                    <td colSpan={TOOLS.length + 1} className="comparison-note-cell">
                      {row.note}
                    </td>
                  </tr>
                )}
              </Fragment>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default function ComparisonPage() {
  return (
    <main className="comparison-page">
      <section className="comparison-page-hero">
        <div className="section-container">
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            SAF vs SVF, Phasar, Lotus, CodeQL, Infer
          </motion.h1>
          <motion.p
            className="comparison-page-lede"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            How SAF, the closest LLVM-IR program-analysis frameworks (SVF, Phasar,
            Lotus), and two widely used source-level analyzers (CodeQL, Infer)
            handle the dimensions framework users compare on most often. Each tool
            has its own strengths; this page surfaces the differences so you can
            pick the right one for the job.
          </motion.p>
        </div>
      </section>

      <section className="comparison-page-body">
        <div className="section-container">
          <ComparisonGroup
            title="Capabilities"
            intro="The technical dimensions program-analysis users compare on most often. CodeQL and Infer don't operate on LLVM IR, so some rows read as 'different paradigm' — they're still listed because users do compare across this boundary when picking a static analyzer."
            rows={capabilityRows}
          />

          <ComparisonGroup
            title="Where SAF stands out"
            intro="The dimensions where SAF's design choices show up most clearly: a Python-first SDK, two simultaneous LLVM toolchains, a browser playground, byte-deterministic output as a contract, and shipped AI-agent skills. These aren't claims that peers fall short — they're the design choices that distinguish SAF."
            rows={differentiationRows}
          />
        </div>
      </section>
    </main>
  );
}
