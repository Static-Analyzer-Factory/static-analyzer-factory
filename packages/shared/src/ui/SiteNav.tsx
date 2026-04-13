import './theme.css';
import './global-nav.css';

export interface SiteNavProps {
  /** Which nav link is active for this app */
  active: 'home' | 'architecture' | 'tutorials' | 'playground' | 'docs';
  /** Path prefix to the site root (e.g. "./" for site, "../" for playground) */
  siteRoot?: string;
  /** Optional left-side content before the brand (e.g. hamburger button) */
  leftSlot?: React.ReactNode;
}

export default function SiteNav({ active, siteRoot = './', leftSlot }: SiteNavProps) {
  const cls = (link: string) => link === active ? 'global-nav-active' : undefined;

  const brand = (
    <a href={siteRoot} className="global-nav-brand">
      <img
        src={`${siteRoot}saf-logo.png`}
        alt=""
        aria-hidden="true"
        className="global-nav-logo"
        width={28}
        height={28}
      />
      SAF
    </a>
  );

  return (
    <nav className="global-nav">
      {leftSlot ? (
        <div className="global-nav-left">
          {leftSlot}
          {brand}
        </div>
      ) : (
        brand
      )}
      <div className="global-nav-links">
        <a href={siteRoot} className={cls('home')}>Home</a>
        <a href={`${siteRoot}#architecture`} className={cls('architecture')}>Architecture</a>
        <a href={`${siteRoot}tutorials/`} className={cls('tutorials')}>Tutorials</a>
        <a href={`${siteRoot}playground/`} className={cls('playground')}>Playground</a>
        <a href={`${siteRoot}docs/`} className={cls('docs')}>Docs</a>
        <a href={`${siteRoot}rustdoc/saf_core/`} target="_blank" rel="noopener noreferrer">API Docs</a>
        <a href="https://github.com/Static-Analyzer-Factory/static-analyzer-factory" target="_blank" rel="noopener noreferrer">GitHub</a>
      </div>
    </nav>
  );
}
