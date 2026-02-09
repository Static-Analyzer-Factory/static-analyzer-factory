/**
 * Spec loading utilities.
 *
 * Loads function specification YAML files from the public/specs directory
 * for display in the spec viewer and for passing to the WASM analysis engine.
 */

export interface SpecEntry {
  path: string;
  category: string;
  name: string;
}

export interface SpecManifest {
  specs: SpecEntry[];
}

export interface LoadedSpec extends SpecEntry {
  yaml: string;
}

let cachedManifest: SpecManifest | null = null;
let cachedSpecs: LoadedSpec[] | null = null;

export async function loadSpecManifest(): Promise<SpecManifest> {
  if (cachedManifest) return cachedManifest;
  // Vite serves public/ files at the root. Use import.meta.env.BASE_URL
  // so it works both in dev and production (e.g., GitHub Pages with a base path).
  const base = import.meta.env.BASE_URL ?? './';
  const resp = await fetch(`${base}specs/index.json`);
  if (!resp.ok) throw new Error('Failed to load spec manifest');
  cachedManifest = (await resp.json()) as SpecManifest;
  return cachedManifest;
}

export async function loadSpec(path: string): Promise<string> {
  const base = import.meta.env.BASE_URL ?? './';
  const resp = await fetch(`${base}specs/${path}`);
  if (!resp.ok) throw new Error(`Failed to load spec: ${path}`);
  return resp.text();
}

export async function loadAllSpecs(): Promise<LoadedSpec[]> {
  if (cachedSpecs) return cachedSpecs;
  const manifest = await loadSpecManifest();
  const specs = await Promise.all(
    manifest.specs.map(async (entry) => ({
      ...entry,
      yaml: await loadSpec(entry.path),
    })),
  );
  cachedSpecs = specs;
  return specs;
}

/** Get unique categories from the manifest. */
export function getCategories(manifest: SpecManifest): string[] {
  const seen = new Set<string>();
  for (const s of manifest.specs) {
    seen.add(s.category);
  }
  return [...seen].sort();
}
