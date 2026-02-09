/** CFG renderer: converts a PropertyGraph (graph_type "cfg") to Cytoscape elements. */

import type { ElementDefinition } from 'cytoscape';
import type { PropertyGraph, HumanLabel } from '../types';
import { tryResolveDisplayBatch } from './resolve-helpers';

/**
 * Render a CFG PropertyGraph as Cytoscape element definitions.
 * Optionally filter to a single function.
 */
export function renderCFG(
  graph: PropertyGraph,
  selectedFunction?: string | null,
): ElementDefinition[] {
  const elements: ElementDefinition[] = [];

  // Collect node IDs for batch resolution
  const nodeIds: string[] = [];
  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const fn = props.function as string | undefined;
    if (selectedFunction && fn && fn !== selectedFunction) continue;
    nodeIds.push(node.id);
  }

  // Batch resolve all node IDs to human-readable labels
  const labelMap = tryResolveDisplayBatch(nodeIds);

  // Build a set of node IDs in the selected function (if any)
  const filteredNodes = new Set<string>();
  // Track entry nodes for back-edge detection
  let entryNodeId: string | null = null;

  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const fn = props.function as string | undefined;

    // Filter by function if specified
    if (selectedFunction && fn && fn !== selectedFunction) {
      continue;
    }

    filteredNodes.add(node.id);

    if (node.labels.includes('Entry')) {
      entryNodeId = node.id;
    }

    // Use WASM DisplayResolver for human-readable block labels
    const resolved: HumanLabel | undefined = labelMap.get(node.id);
    const label = resolved?.short_name
      ?? (props.name as string)
      ?? node.id;

    const classes: string[] = [];
    if (node.labels.includes('Entry')) {
      classes.push('entry');
    }

    // Heuristic: blocks ending with ret/unreachable are exit blocks
    const blockName = label.toLowerCase();
    if (
      blockName.includes('return') ||
      blockName.includes('exit') ||
      blockName.includes('unreachable')
    ) {
      classes.push('exit');
    }

    // Attach source line range from WASM resolver
    const sourceLoc = resolved?.source_loc;

    elements.push({
      data: {
        id: node.id,
        label,
        function: fn,
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: classes.join(' '),
    });
  }

  // Build adjacency list for back-edge detection
  const successors = new Map<string, string[]>();
  const filteredEdges: typeof graph.edges = [];
  for (const edge of graph.edges) {
    if (!filteredNodes.has(edge.src) || !filteredNodes.has(edge.dst)) {
      continue;
    }
    filteredEdges.push(edge);
    let succs = successors.get(edge.src);
    if (!succs) {
      succs = [];
      successors.set(edge.src, succs);
    }
    succs.push(edge.dst);
  }

  // Detect back-edges via DFS from the entry node
  const backEdges = detectBackEdges(entryNodeId, successors);

  for (const edge of filteredEdges) {
    const classes: string[] = [];
    const props = edge.properties ?? {};
    const condition = props.condition as string | undefined;
    const edgeLabel = (props.label as string | undefined) ?? condition ?? '';
    if (condition === 'true') {
      classes.push('true-branch');
    } else if (condition === 'false') {
      classes.push('false-branch');
    }

    const edgeKey = `${edge.src}->${edge.dst}`;
    if (backEdges.has(edgeKey)) {
      classes.push('back-edge');
    }

    elements.push({
      data: {
        id: edgeKey,
        source: edge.src,
        target: edge.dst,
        label: edgeLabel,
      },
      classes: classes.join(' '),
    });
  }

  return elements;
}

/**
 * Detect back-edges in a directed graph via DFS.
 * A back-edge is an edge from a node to an ancestor in the DFS tree.
 * Returns a set of edge keys "src->dst".
 */
function detectBackEdges(
  entry: string | null,
  successors: Map<string, string[]>,
): Set<string> {
  const backEdges = new Set<string>();
  if (!entry) return backEdges;

  const WHITE = 0, GRAY = 1, BLACK = 2;
  const color = new Map<string, number>();

  // Initialize all nodes as WHITE
  for (const node of successors.keys()) {
    color.set(node, WHITE);
  }

  // Iterative DFS using an explicit stack
  const stack: { node: string; succIdx: number }[] = [];
  color.set(entry, GRAY);
  stack.push({ node: entry, succIdx: 0 });

  while (stack.length > 0) {
    const frame = stack[stack.length - 1];
    const succs = successors.get(frame.node) ?? [];

    if (frame.succIdx >= succs.length) {
      // Done with this node
      color.set(frame.node, BLACK);
      stack.pop();
      continue;
    }

    const succ = succs[frame.succIdx];
    frame.succIdx++;

    const succColor = color.get(succ) ?? WHITE;
    if (succColor === GRAY) {
      // Edge to an ancestor in the DFS tree → back-edge
      backEdges.add(`${frame.node}->${succ}`);
    } else if (succColor === WHITE) {
      color.set(succ, GRAY);
      stack.push({ node: succ, succIdx: 0 });
    }
  }

  return backEdges;
}

/** Recommended layout options for CFG (dagre handles cycles properly). */
export const cfgLayoutOptions = {
  name: 'dagre',
  rankDir: 'TB',
  nodeSep: 40,
  rankSep: 60,
  edgeSep: 20,
};
