import type { Tutorial, TutorialIndexEntry } from './types';

const BASE = import.meta.env.BASE_URL;

export async function loadTutorial(id: string): Promise<Tutorial> {
  const res = await fetch(`${BASE}tutorials/${id}.json`);
  if (!res.ok) throw new Error(`Tutorial not found: ${id}`);
  return res.json();
}

export async function loadTutorialIndex(): Promise<TutorialIndexEntry[]> {
  const res = await fetch(`${BASE}tutorials/index.json`);
  if (!res.ok) throw new Error('Tutorial index not found');
  return res.json();
}
