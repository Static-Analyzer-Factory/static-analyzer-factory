import type { TutorialStep } from './types';

const BASE = import.meta.env.BASE_URL;

export async function loadTutorialSteps(tutorialId: string): Promise<TutorialStep[]> {
  const res = await fetch(`${BASE}content/${tutorialId}/steps.json`);
  if (!res.ok) throw new Error(`Tutorial not found: ${tutorialId}`);
  return res.json();
}
