export type AlgorithmType =
  | 'andersen-pta' | 'ifds-taint' | 'interval-absint' | 'memory-ssa'
  | 'kcfa-pta' | 'sparse-vf' | 'dominator-tree' | 'callgraph-construction';

export interface TraceNode {
  id: string;
  label: string;
  type: string;
  properties?: Record<string, unknown>;
}

export interface TraceEdge {
  src: string;
  dst: string;
  label?: string;
  type: string;
}

export interface TraceGraphState {
  nodes: TraceNode[];
  edges: TraceEdge[];
}

export interface TraceDiff {
  added: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  removed: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  changed: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
}

export interface PtaState {
  worklist: Array<{ rank: number; values: string[] }>;
  pointsTo: Record<string, string[]>;
  constraints: Array<{
    type: 'addr' | 'copy' | 'load' | 'store' | 'gep';
    from: string;
    to: string;
    processed: boolean;
  }>;
}

export interface IfdsState {
  worklist: Array<{ func: string; d1: string; inst: string; d2: string }>;
  summaryEdges: Array<{ func: string; entryFact: string; exitFact: string }>;
  factsAt: Record<string, string[]>;
}

export interface IntervalState {
  currentBlock: string;
  iteration: number;
  variables: Record<string, { lo: number | '-inf'; hi: number | '+inf' }>;
  operation?: { type: 'join' | 'widen' | 'narrow'; description: string };
}

export interface MssaState {
  query: string;
  walkChain: Array<{
    inst: string;
    type: 'load' | 'store' | 'phi';
    aliasQuery?: { ptr1: string; ptr2: string; result: 'may' | 'must' | 'no' };
  }>;
  pointsToContext: Record<string, string[]>;
}

export interface KCfaState {
  currentContext: string[];
  worklist: Array<{ context: string[]; variable: string }>;
  pointsTo: Record<string, Record<string, string[]>>;
  contextCount: number;
}

export interface SvfState {
  currentNode: string;
  vfEdges: Array<{ src: string; dst: string; kind: 'direct' | 'store' | 'load'; processed: boolean }>;
  facts: Record<string, string[]>;
}

export interface DomState {
  processed: string[];
  idom: Record<string, string>;
  domFrontier: Record<string, string[]>;
  loopHeaders: string[];
  backEdges: Array<{ src: string; dst: string }>;
}

export interface CgState {
  algorithm: 'cha' | 'rta' | 'vta';
  resolvedCalls: Array<{ callsite: string; targets: string[]; precision: 'exact' | 'over-approx' }>;
  reachableMethods: string[];
  unresolvedCalls: string[];
}

export type AlgorithmState =
  | PtaState | IfdsState | IntervalState | MssaState
  | KCfaState | SvfState | DomState | CgState;

export interface PseudocodeLine {
  text: string;
}

export interface Pseudocode {
  title: string;
  lines: PseudocodeLine[];
}

export interface TraceStep {
  id: number;
  action: string;
  explanation: string;
  highlights: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  graph: TraceGraphState;
  diff: TraceDiff;
  algorithmState: AlgorithmState;
  phase: number;          // index into AlgorithmTrace.phases
  activeLines: number[];  // indices into AlgorithmTrace.pseudocode.lines
}

export interface AlgorithmTrace {
  algorithm: AlgorithmType;
  title: string;
  example: { code: string; language: string };
  phases: string[];          // e.g. ["Extract", "Process Rank 0", ...]
  pseudocode: Pseudocode;    // algorithm pseudocode for rail display
  steps: TraceStep[];
}
