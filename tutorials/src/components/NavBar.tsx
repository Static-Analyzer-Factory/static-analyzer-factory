import SiteNav from '@saf/web-shared/ui/SiteNav';
import './NavBar.css';

export default function NavBar({ onMenuToggle }: { onMenuToggle?: () => void }) {
  const base = import.meta.env.BASE_URL;

  return (
    <SiteNav
      active="tutorials"
      siteRoot={`${base}../`}
      leftSlot={onMenuToggle ? (
        <button className="hamburger" onClick={onMenuToggle} aria-label="Toggle sidebar">
          &#9776;
        </button>
      ) : undefined}
    />
  );
}
