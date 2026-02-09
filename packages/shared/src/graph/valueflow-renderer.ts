/** Value-flow renderer: converts a PropertyGraph (graph_type "valueflow") to Cytoscape elements. */

import type { ElementDefinition } from 'cytoscape';
import type { PropertyGraph, HumanLabel } from '../types';
import { tryResolveDisplayBatch } from './resolve-helpers';

/** Edge type to CSS class mapping. */
const EDGE_CLASS: Record<string, string> = {
  'Store': 'store',
  'Load': 'load',
  'DefUse': 'define',
};

/**
 * Render a valueflow PropertyGraph as Cytoscape element definitions.
 */
export function renderValueFlow(
  graph: PropertyGraph,
): ElementDefinition[] {
  const elements: ElementDefinition[] = [];

  // Collect all node IDs for batch resolution
  const nodeIds = graph.nodes.map(n => n.id);
  const labelMap = tryResolveDisplayBatch(nodeIds);

  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const kind = props.kind as string | undefined;

    // Use WASM DisplayResolver for human-readable labels
    const resolved: HumanLabel | undefined = labelMap.get(node.id);
    const label = resolved?.short_name
      ?? (props.name as string)
      ?? node.id;

    const classes: string[] = [];
    if (kind === 'Value') {
      classes.push('value-node');
    } else if (kind === 'Location') {
      classes.push('location-node');
    } else if (kind === 'UnknownMem') {
      classes.push('unknown-mem');
    }

    const sourceLoc = resolved?.source_loc;
    elements.push({
      data: {
        id: node.id,
        label,
        kind,
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: classes.join(' '),
    });
  }

  for (const edge of graph.edges) {
    const cls = EDGE_CLASS[edge.edge_type];
    const classes: string[] = cls ? [cls] : [];

    elements.push({
      data: {
        id: `${edge.src}-${edge.edge_type}->${edge.dst}`,
        source: edge.src,
        target: edge.dst,
        label: edge.edge_type,
      },
      classes: classes.join(' '),
    });
  }

  return elements;
}

/** Recommended layout options for value-flow graphs. */
export const valueflowLayoutOptions = {
  name: 'cose',
  animate: false,
  nodeDimensionsIncludeLabels: true,
  idealEdgeLength: 100,
  nodeOverlap: 20,
  padding: 30,
};
