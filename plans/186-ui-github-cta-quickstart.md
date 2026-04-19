# Plan 186 — Landing-page GitHub CTA + Quick Start

**Date:** 2026-04-19
**Area:** frontend (site)
**Status:** approved
**Branch:** `ui/github-cta-and-quickstart`

## Motivation

Today the landing page (`site/`) has two hero CTAs (Playground, Docs). The GitHub link is tucked into `SiteNav` and `Footer` only. First-time visitors who want to jump to source, star, or fork have to scan the nav bar. Separately, the page has no copy-pasteable install block — users must leave for the README to figure out how to run SAF locally.

## Goals

1. Give first-time visitors a prominent "View on GitHub" CTA in the hero.
2. Show a self-contained Quick Start block with Docker install commands on the landing page, so visitors can go from "arrived" to "running SAF locally" without leaving the site.

## Non-goals

- No changes to `SiteNav`, `Footer`, playground, tutorials, docs, or rustdoc.
- No hosted interactive install wizard — a single static code block is enough.
- No new tutorial or docs content.

## Design

### 1. Hero GitHub CTA (`site/src/components/Hero.tsx`)

Add a third button in `hero-ctas`, after "Read the Docs":

```tsx
<a
  href="https://github.com/Static-Analyzer-Factory/static-analyzer-factory"
  target="_blank"
  rel="noopener noreferrer"
  className="cta cta-secondary cta-github"
>
  <svg viewBox="0 0 24 24" fill="currentColor" className="cta-github-icon" aria-hidden="true">
    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.3 3.438 9.8 8.205 11.387…" />
  </svg>
  View on GitHub
</a>
```

- Reuse the GitHub icon SVG path from `site/src/components/Footer.tsx`.
- Style piggybacks on existing `.cta-secondary`. Add `.cta-github-icon { width: 18px; height: 18px; margin-right: 0.5rem; }` in `App.css`.
- Keeps the animated `motion.div` wrapper unchanged — third child inside `hero-ctas`.

### 2. Quick Start section (new `site/src/components/QuickStart.tsx`)

New component inserted between `Features` and `Personas` in `site/src/App.tsx`.

```tsx
import { motion } from 'motion/react';
import { useState } from 'react';

const INSTALL = `git clone https://github.com/Static-Analyzer-Factory/static-analyzer-factory.git
cd static-analyzer-factory
make shell`;

export default function QuickStart() {
  const [copied, setCopied] = useState(false);
  const onCopy = () => {
    navigator.clipboard.writeText(INSTALL).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };
  return (
    <section className="quickstart" id="quickstart">
      <div className="section-container">
        <motion.h2
          className="section-title"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
        >
          Get Started in 30 Seconds
        </motion.h2>
        <motion.div
          className="quickstart-card"
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-60px' }}
          transition={{ duration: 0.5 }}
        >
          <div className="quickstart-code-wrap">
            <pre className="quickstart-code"><code>{INSTALL}</code></pre>
            <button type="button" className="quickstart-copy" onClick={onCopy} aria-label="Copy install commands">
              {copied ? 'Copied' : 'Copy'}
            </button>
          </div>
          <p className="quickstart-note">
            Docker does the rest — LLVM 18 and the <code>saf</code> Python SDK are auto-installed inside the shell on
            first run. Try <code>saf --help</code> or <code>python3 -c 'import saf'</code>.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
```

### 3. App wiring (`site/src/App.tsx`)

Add `import QuickStart from './components/QuickStart';` and render it between `<Features />` and `<Personas />` on the home route. Architecture route unchanged.

### 4. CSS additions (`site/src/App.css`)

Follow the existing section pattern (alternating `--color-bg` / `--color-surface`).

```css
/* ── Quick Start ── */
.quickstart {
  padding: 5rem 0;
  background: var(--color-bg);
}

.quickstart-card {
  max-width: 760px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.quickstart-code-wrap {
  position: relative;
  background: var(--color-surface-editor);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 1.25rem 1.5rem;
}

.quickstart-code {
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 0.9rem;
  line-height: 1.7;
  color: var(--color-text-secondary);
  white-space: pre;
  overflow-x: auto;
}

.quickstart-copy {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  background: var(--color-surface-raised);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 0.3rem 0.65rem;
  font-size: 0.75rem;
  cursor: pointer;
  transition: color 0.15s ease, border-color 0.15s ease;
}

.quickstart-copy:hover {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.quickstart-note {
  font-size: 0.9rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
  text-align: center;
}

.quickstart-note code {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 0.85em;
  background: var(--color-surface-raised);
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
}

.cta-github-icon {
  width: 18px;
  height: 18px;
  margin-right: 0.5rem;
}
```

Mobile: `.quickstart-copy` stays positioned; existing responsive rules don't need changes. If the install text is too wide on very narrow screens, `overflow-x: auto` handles it.

## Files touched

| File | Change |
|------|--------|
| `site/src/components/Hero.tsx` | Add third CTA (GitHub) with icon |
| `site/src/components/QuickStart.tsx` | NEW component |
| `site/src/App.tsx` | Import + render `<QuickStart />` between Features and Personas |
| `site/src/App.css` | Add `.quickstart*` rules + `.cta-github-icon` rule |
| `plans/PROGRESS.md` | Index + session log |

## Risks

- `navigator.clipboard` requires a secure context. GitHub Pages serves over HTTPS, and localhost counts as secure — fine.
- Adding a section slightly lengthens the landing page. Acceptable for the target persona (devs).

## Test plan

1. `cd site && npm install && npm run build` — succeeds with no TS errors.
2. Run `npm run dev`, open in browser:
   - Hero shows three CTAs. "View on GitHub" opens the repo in a new tab.
   - New Quick Start section appears between Features and Personas. Copy button copies the 3-line block.
   - Scroll to footer — existing links still work.
3. Responsive check at 375 px width: CTAs wrap to new lines; quick-start code block scrolls horizontally; nothing overflows the viewport.

## Rollout

- Branch: `ui/github-cta-and-quickstart` off `main`.
- Commits: one for the feature, one for the PROGRESS.md update.
- Merge via PR when the user approves the visible result.
