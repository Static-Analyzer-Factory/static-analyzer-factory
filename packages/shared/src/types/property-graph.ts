/** PropertyGraph format matching SAF's unified graph export. */

export interface PropertyGraph {
  schema_version: string;
  graph_type: 'cfg' | 'callgraph' | 'defuse' | 'valueflow';
  metadata: Record<string, unknown>;
  nodes: PgNode[];
  edges: PgEdge[];
}

export interface PgNode {
  id: string;
  labels: string[];
  properties?: Record<string, unknown>;
}

export interface PgEdge {
  src: string;
  dst: string;
  edge_type: string;
  properties?: Record<string, unknown>;
}

/** PTA result — not PropertyGraph format. */
export interface PtaResult {
  points_to: Array<{
    value: string;
    locations: string[];
  }>;
}

// Re-export old names for backwards compatibility with existing imports
export type GraphNode = PgNode;
export type GraphEdge = PgEdge;
