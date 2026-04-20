import { useState, useEffect, lazy, Suspense } from 'react';
import Hero from './components/Hero';
import Features from './components/Features';
import QuickStart from './components/QuickStart';
import Personas from './components/Personas';
import TechHighlights from './components/TechHighlights';
import Footer from './components/Footer';
import SiteNav from '@saf/web-shared/ui/SiteNav';
import './App.css';

const ArchitectureDiagram = lazy(() => import('./components/ArchitectureDiagram/ArchitectureDiagram'));

export default function App() {
  const [route, setRoute] = useState(window.location.hash);

  useEffect(() => {
    const onHash = () => setRoute(window.location.hash);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  const active = route === '#architecture' ? 'architecture' as const : 'home' as const;

  if (route === '#architecture') {
    return (
      <div className="landing">
        <SiteNav active={active} />
        <Suspense fallback={<div style={{ minHeight: '100vh', background: '#faf9f7' }} />}>
          <ArchitectureDiagram />
        </Suspense>
        <Footer />
      </div>
    );
  }

  return (
    <div className="landing">
      <SiteNav active={active} />
      <Hero />
      <QuickStart />
      <Features />
      <Personas />
      <TechHighlights />
      <Footer />
    </div>
  );
}
