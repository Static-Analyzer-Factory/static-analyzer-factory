export type Difficulty = 'beginner' | 'intermediate' | 'advanced';
export type Mode = 'browser' | 'local' | 'both';
export type Category = 'getting-started' | 'memory-safety' | 'information-flow' | 'algorithms' | 'advanced';
export type Subcategory = 'pointer-analysis' | 'dataflow-analysis' | 'memory-modeling' | 'graph-foundations';
export type GraphType = 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta' | 'algorithm';

export interface TutorialMeta {
  id: string;
  title: string;
  description: string;
  difficulty: Difficulty;
  mode: Mode;
  category: Category;
  subcategory?: Subcategory;
  prerequisites?: string[];
}

export interface TutorialStep {
  title: string;
  content: string;
  code?: string;
  codeLanguage?: 'c' | 'python' | 'bash';
  graphType?: GraphType;
  playground?: string;
  localCmd?: string;
  localScript?: string;
  highlightLines?: number[];
  challenge?: string;
  stepType?: 'prose' | 'interactive' | 'algorithm';
  algorithmTrace?: string;
}

export interface Tutorial {
  meta: TutorialMeta;
  steps: TutorialStep[];
}
