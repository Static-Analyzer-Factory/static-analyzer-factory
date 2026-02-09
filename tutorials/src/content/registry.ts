import type { Category, Subcategory, TutorialMeta } from './types';

export interface CategoryInfo {
  id: Category;
  title: string;
  description: string;
  icon: string;
}

export const CATEGORIES: CategoryInfo[] = [
  {
    id: 'getting-started',
    title: 'Getting Started',
    description: 'Learn the fundamentals of static analysis and the SAF toolkit.',
    icon: '🚀',
  },
  {
    id: 'memory-safety',
    title: 'Memory Safety',
    description: 'Detect use-after-free, double-free, and memory leak vulnerabilities.',
    icon: '🛡️',
  },
  {
    id: 'information-flow',
    title: 'Information Flow',
    description: 'Track data flow from sources to sinks for taint analysis.',
    icon: '🔍',
  },
  {
    id: 'algorithms',
    title: 'Algorithms',
    description: 'Step through SAF\'s core analysis algorithms and watch them solve.',
    icon: '📊',
  },
  {
    id: 'advanced',
    title: 'Advanced',
    description: 'Build custom analyzers and author analysis specifications.',
    icon: '⚙️',
  },
];

export interface SubcategoryInfo {
  id: Subcategory;
  title: string;
}

export const SUBCATEGORIES: SubcategoryInfo[] = [
  { id: 'pointer-analysis', title: 'Pointer Analysis' },
  { id: 'dataflow-analysis', title: 'Dataflow Analysis' },
  { id: 'memory-modeling', title: 'Memory Modeling' },
  { id: 'graph-foundations', title: 'Graph Foundations' },
];

export const TUTORIALS: TutorialMeta[] = [
  {
    id: 'first-analysis',
    title: 'Your First Analysis',
    description: 'Load a C program and explore its control flow and value flow graphs.',
    difficulty: 'beginner',
    mode: 'browser',
    category: 'getting-started',
  },
  {
    id: 'understanding-graphs',
    title: 'Understanding Analysis Graphs',
    description: 'Learn the five graph types SAF produces: CFG, Call Graph, Def-Use, Value Flow, and Points-To.',
    difficulty: 'beginner',
    mode: 'browser',
    category: 'getting-started',
  },
  {
    id: 'uaf-detection',
    title: 'Detecting Use-After-Free',
    description: 'Trace a use-after-free vulnerability through CFG, value-flow, and points-to analysis.',
    difficulty: 'beginner',
    mode: 'both',
    category: 'memory-safety',
  },
  {
    id: 'double-free',
    title: 'Detecting Double Free',
    description: 'Find double-free bugs caused by pointer aliasing using PTA and value-flow analysis.',
    difficulty: 'intermediate',
    mode: 'both',
    category: 'memory-safety',
  },
  {
    id: 'memory-leak',
    title: 'Detecting Memory Leaks',
    description: 'Track allocation flow paths to find missing deallocations.',
    difficulty: 'intermediate',
    mode: 'local',
    category: 'memory-safety',
  },
  {
    id: 'taint-basics',
    title: 'Taint Analysis Basics',
    description: 'Learn source-sink analysis by tracing data flow from user input to output.',
    difficulty: 'beginner',
    mode: 'browser',
    category: 'information-flow',
  },
  {
    id: 'command-injection',
    title: 'Detecting Command Injection',
    description: 'Find command injection vulnerabilities by tracing untrusted input to system calls.',
    difficulty: 'intermediate',
    mode: 'both',
    category: 'information-flow',
  },
  {
    id: 'pta-andersen',
    title: 'Pointer Analysis (Andersen\'s)',
    description: 'Watch inclusion-based constraint solving: worklist processing, points-to set propagation, and cycle detection.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'pointer-analysis',
  },
  {
    id: 'ifds-taint',
    title: 'IFDS Taint Analysis',
    description: 'Follow path edges through the IFDS tabulation algorithm as tainted data flows between functions.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'dataflow-analysis',
  },
  {
    id: 'interval-analysis',
    title: 'Interval Abstract Interpretation',
    description: 'See how abstract intervals widen at loop headers and narrow to precise bounds.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'dataflow-analysis',
  },
  {
    id: 'memory-ssa',
    title: 'Memory SSA & Clobber Walking',
    description: 'Trace demand-driven backward walks through memory definitions with alias queries.',
    difficulty: 'advanced',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'memory-modeling',
  },
  {
    id: 'pta-kcfa',
    title: 'Context-Sensitive PTA (k-CFA)',
    description: 'Compare context-insensitive and k-CFA pointer analysis to see how call-site sensitivity improves precision.',
    difficulty: 'advanced',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'pointer-analysis',
  },
  {
    id: 'sparse-valueflow',
    title: 'Sparse Value-Flow Analysis',
    description: 'Watch value-flow edges propagate facts along def-use chains, skipping irrelevant statements.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'dataflow-analysis',
  },
  {
    id: 'dominator-tree',
    title: 'Dominator Tree & Loop Detection',
    description: 'Build the dominator tree incrementally and identify loop headers via back edges.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'graph-foundations',
  },
  {
    id: 'callgraph-construction',
    title: 'Call Graph Construction',
    description: 'Compare CHA, RTA, and VTA call graph algorithms on function pointers to see precision improve.',
    difficulty: 'intermediate',
    mode: 'browser',
    category: 'algorithms',
    subcategory: 'graph-foundations',
  },
  {
    id: 'custom-analyzer',
    title: 'Building a Custom Analyzer',
    description: 'Write a Python script that uses the SAF SDK to detect vulnerabilities programmatically.',
    difficulty: 'advanced',
    mode: 'local',
    category: 'advanced',
  },
  {
    id: 'specs-authoring',
    title: 'Authoring Analysis Specs',
    description: 'Create YAML specification files to model library functions for precise analysis.',
    difficulty: 'advanced',
    mode: 'local',
    category: 'advanced',
  },
];
