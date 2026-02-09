/** Call graph renderer: converts a PropertyGraph (graph_type "callgraph") to Cytoscape elements. */

import type { ElementDefinition } from 'cytoscape';
import type { PropertyGraph, HumanLabel } from '../types';
import { tryResolveDisplayBatch } from './resolve-helpers';

/**
 * Render a callgraph PropertyGraph as Cytoscape element definitions.
 */
export function renderCallGraph(graph: PropertyGraph): ElementDefinition[] {
  const elements: ElementDefinition[] = [];
  const indirectNodeIds = new Set<string>();

  // First pass: identify indirect nodes (to skip them)
  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const kind = props.kind as string | undefined;
    if (kind === 'indirect') {
      indirectNodeIds.add(node.id);
    }
  }

  // Collect IDs for batch resolution (excluding indirect nodes)
  const nodeIds: string[] = [];
  for (const node of graph.nodes) {
    if (!indirectNodeIds.has(node.id)) {
      nodeIds.push(node.id);
    }
  }

  const labelMap = tryResolveDisplayBatch(nodeIds);

  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const kind = props.kind as string | undefined;

    // Skip resolved indirect call placeholders (visual noise)
    if (kind === 'indirect') continue;

    const classes: string[] = [];
    if (kind === 'external') {
      classes.push('external');
    }

    // Use WASM DisplayResolver for label and source lines
    const resolved: HumanLabel | undefined = labelMap.get(node.id);
    const name = resolved?.short_name
      ?? (props.name as string)
      ?? node.id;
    const sourceLoc = resolved?.source_loc;

    elements.push({
      data: {
        id: node.id,
        label: name,
        kind,
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: classes.join(' '),
    });
  }

  for (const edge of graph.edges) {
    // Skip edges to/from indirect placeholders
    if (indirectNodeIds.has(edge.src) || indirectNodeIds.has(edge.dst)) {
      continue;
    }
    elements.push({
      data: {
        id: `${edge.src}->${edge.dst}`,
        source: edge.src,
        target: edge.dst,
        label: '',
      },
    });
  }

  return elements;
}

/** Recommended layout options for call graphs. */
export const callgraphLayoutOptions = {
  name: 'cose',
  animate: false,
  nodeDimensionsIncludeLabels: true,
  idealEdgeLength: 120,
  nodeOverlap: 20,
  padding: 30,
};
