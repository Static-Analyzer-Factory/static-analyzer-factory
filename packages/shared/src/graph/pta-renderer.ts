/** PTA renderer: converts PTA result to a bipartite Cytoscape graph. */

import type { ElementDefinition } from 'cytoscape';
import type { PTAResult, HumanLabel } from '../types';
import { tryResolveDisplayBatch } from './resolve-helpers';

/**
 * Render a PTA result as a bipartite graph:
 * value/pointer nodes on the left, location/pointee nodes on the right.
 */
export function renderPTA(
  pta: PTAResult,
): ElementDefinition[] {
  const elements: ElementDefinition[] = [];
  const locationNodes = new Set<string>();
  const valLabels = pta.value_labels ?? {};
  const locLabels = pta.location_labels ?? {};

  // Collect unique location nodes first so we don't duplicate
  for (const entry of pta.points_to) {
    for (const loc of entry.locations) {
      locationNodes.add(loc);
    }
  }

  // Collect all IDs for batch resolution: value IDs + location IDs
  const allIds: string[] = [];
  for (const entry of pta.points_to) {
    allIds.push(entry.value);
  }
  for (const loc of locationNodes) {
    allIds.push(loc);
  }
  const labelMap = tryResolveDisplayBatch(allIds);

  // Add value (pointer) nodes
  // Prefer WASM DisplayResolver labels (which include C variable names)
  // over Rust-side PTA labels (which only have instruction info)
  for (const entry of pta.points_to) {
    const resolved: HumanLabel | undefined = labelMap.get(entry.value);
    const label = resolved?.short_name
      ?? valLabels[entry.value]
      ?? shortenId(entry.value);
    const sourceLoc = resolved?.source_loc;
    elements.push({
      data: {
        id: `val:${entry.value}`,
        label,
        nodeType: 'value',
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: 'pta-value',
    });
  }

  // Add location (pointee) nodes
  for (const loc of locationNodes) {
    const resolved: HumanLabel | undefined = labelMap.get(loc);
    const label = resolved?.short_name
      ?? locLabels[loc]
      ?? shortenId(loc);
    const sourceLoc = resolved?.source_loc;
    elements.push({
      data: {
        id: `loc:${loc}`,
        label,
        nodeType: 'location',
        ...(sourceLoc ? { lineStart: sourceLoc.line, lineEnd: sourceLoc.end_line ?? sourceLoc.line } : {}),
      },
      classes: 'pta-location',
    });
  }

  // Add edges from values to locations
  for (const entry of pta.points_to) {
    for (const loc of entry.locations) {
      elements.push({
        data: {
          id: `val:${entry.value}->loc:${loc}`,
          source: `val:${entry.value}`,
          target: `loc:${loc}`,
          label: '',
        },
      });
    }
  }

  return elements;
}

/** Shorten a hex ID like "0x00000000000000000000000000001234" to "...1234". */
function shortenId(id: string): string {
  if (!id.startsWith('0x')) return id;
  const hex = id.slice(2).replace(/^0+/, '');
  return hex.length > 8 ? `...${hex.slice(-8)}` : `0x${hex || '0'}`;
}

/** Recommended layout options for PTA bipartite graphs. */
export const ptaLayoutOptions = {
  name: 'concentric',
  concentric: (node: { data: (key: string) => string }) =>
    node.data('nodeType') === 'value' ? 2 : 1,
  levelWidth: () => 1,
  animate: false,
  padding: 30,
  minNodeSpacing: 40,
};
