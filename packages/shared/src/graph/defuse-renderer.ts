/** Def-use renderer: converts a PropertyGraph (graph_type "defuse") to Cytoscape elements. */

import type { ElementDefinition } from 'cytoscape';
import type { PropertyGraph, HumanLabel } from '../types';
import { tryResolveDisplayBatch } from './resolve-helpers';

/**
 * Render a defuse PropertyGraph as Cytoscape element definitions.
 * Optionally filter to a single function.
 */
export function renderDefUse(
  graph: PropertyGraph,
  selectedFunction?: string | null,
): ElementDefinition[] {
  const elements: ElementDefinition[] = [];
  const filteredNodes = new Set<string>();

  // Collect node IDs for batch resolution (respecting function filter)
  const nodeIds: string[] = [];
  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const fn = props.function as string | undefined;
    if (selectedFunction && fn && fn !== selectedFunction) continue;
    nodeIds.push(node.id);
  }

  const labelMap = tryResolveDisplayBatch(nodeIds);

  for (const node of graph.nodes) {
    const props = node.properties ?? {};
    const fn = props.function as string | undefined;

    if (selectedFunction && fn && fn !== selectedFunction) {
      continue;
    }

    filteredNodes.add(node.id);

    const isInstruction = node.labels.includes('Instruction');
    const isValue = node.labels.includes('Value');

    // Use WASM DisplayResolver for human-readable labels
    const resolved: HumanLabel | undefined = labelMap.get(node.id);
    const label = resolved?.short_name
      ?? (props.name as string)
      ?? node.id;

    const classes: string[] = [];
    if (isInstruction) {
      classes.push('instruction');
    } else if (isValue) {
      classes.push('value-node');
    }

    const sourceLoc = resolved?.source_loc;
    elements.push({
      data: {
        id: node.id,
        label,
        nodeType: isInstruction ? 'instruction' : 'value',
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: classes.join(' '),
    });
  }

  for (const edge of graph.edges) {
    if (!filteredNodes.has(edge.src) || !filteredNodes.has(edge.dst)) {
      continue;
    }

    const classes: string[] = [];
    if (edge.edge_type === 'DEFINES') {
      classes.push('define');
    } else if (edge.edge_type === 'USED_BY') {
      classes.push('use');
    }

    elements.push({
      data: {
        id: `${edge.src}-${edge.edge_type}->${edge.dst}`,
        source: edge.src,
        target: edge.dst,
        label: edge.edge_type === 'DEFINES' ? 'def' : 'use',
      },
      classes: classes.join(' '),
    });
  }

  return elements;
}

/** Recommended layout options for def-use graphs. */
export const defuseLayoutOptions = {
  name: 'breadthfirst',
  directed: true,
  spacingFactor: 1.5,
  avoidOverlap: true,
};
