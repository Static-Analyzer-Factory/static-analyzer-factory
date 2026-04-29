import { useState, useEffect, lazy, Suspense } from 'react';
import Hero from './components/Hero';
import Features from './components/Features';
import QuickStart from './components/QuickStart';
import Personas from './components/Personas';
import TechHighlights from './components/TechHighlights';
import Comparison from './components/Comparison';
import Footer from './components/Footer';
import SiteNav from '@saf/web-shared/ui/SiteNav';
import './App.css';

const ArchitectureDiagram = lazy(() => import('./components/ArchitectureDiagram/ArchitectureDiagram'));
const ComparisonPage = lazy(() => import('./components/ComparisonPage'));

const HOME_TITLE = 'SAF — Static Analyzer Factory | Program Analysis Framework for LLVM IR';
const COMPARISON_TITLE = 'SAF vs SVF, Phasar, Lotus, CodeQL, Infer — Static Analysis Framework Comparison';
const ARCHITECTURE_TITLE = 'SAF Architecture — Static Analyzer Factory';

export default function App() {
  const [route, setRoute] = useState(window.location.hash);

  useEffect(() => {
    const onHash = () => setRoute(window.location.hash);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  useEffect(() => {
    if (route === '#comparison') {
      document.title = COMPARISON_TITLE;
    } else if (route === '#architecture') {
      document.title = ARCHITECTURE_TITLE;
    } else {
      document.title = HOME_TITLE;
    }
  }, [route]);

  const active =
    route === '#architecture'
      ? ('architecture' as const)
      : route === '#comparison'
        ? ('comparison' as const)
        : ('home' as const);

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

  if (route === '#comparison') {
    return (
      <div className="landing">
        <SiteNav active={active} />
        <Suspense fallback={<div style={{ minHeight: '100vh', background: '#faf9f7' }} />}>
          <ComparisonPage />
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
      <Comparison />
      <Footer />
    </div>
  );
}
