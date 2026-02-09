export interface TutorialStep {
  title: string;
  text: string;
  code?: string;
  graph: 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';
  highlightLines?: number[];
  prompt?: string;
}

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  steps: TutorialStep[];
}

export interface TutorialIndexEntry {
  id: string;
  title: string;
  difficulty: string;
}
