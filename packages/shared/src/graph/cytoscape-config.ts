/** Shared Cytoscape.js configuration for SAF graph rendering (Morandi theme). */

import cytoscape from 'cytoscape';
import cytoscapeDagre from 'cytoscape-dagre';
import type { Core, ElementDefinition, StylesheetStyle } from 'cytoscape';

// Register dagre layout plugin (hierarchical layout that handles cycles properly)
cytoscape.use(cytoscapeDagre);

export const COLORS = {
  bg: '#f1eeeb',
  nodeBg: '#faf9f7',
  nodeText: '#2c2c2e',
  edgeDefault: '#c0bab4',
  entry: '#3d9b8f',
  exit: '#c75050',
  external: '#c49a3c',
  selected: '#3d9b8f',
  trueBranch: '#3d9b8f',
  falseBranch: '#c75050',
  store: '#c49a3c',
  load: '#5088b5',
  defineEdge: '#3d9b8f',
  useEdge: '#5088b5',
  value: '#5088b5',
  location: '#3d9b8f',
  unknownMem: '#c75050',
} as const;

export const stylesheet: StylesheetStyle[] = [
  // -- Node defaults --
  {
    selector: 'node',
    style: {
      'background-color': COLORS.nodeBg,
      'label': 'data(label)',
      'color': COLORS.nodeText,
      'text-valign': 'center',
      'text-halign': 'center',
      'font-size': '11px',
      'font-family': 'monospace',
      'border-width': 1,
      'border-color': '#d4cac0',
      'width': 'label',
      'height': 'label',
      'padding': '10px',
      'shape': 'roundrectangle',
      'text-wrap': 'wrap',
      'text-max-width': '200px',
      'text-outline-width': 2,
      'text-outline-color': COLORS.bg,
    },
  },
  // -- Edge defaults --
  {
    selector: 'edge',
    style: {
      'width': 2,
      'line-color': COLORS.edgeDefault,
      'target-arrow-color': COLORS.edgeDefault,
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
      'font-size': '9px',
      'color': '#7a6f66',
      'label': 'data(label)',
      'text-outline-width': 2,
      'text-outline-color': COLORS.bg,
    },
  },
  // -- Entry block (green border) --
  {
    selector: 'node.entry',
    style: {
      'border-color': COLORS.entry,
      'border-width': 2,
    },
  },
  // -- Exit block (red border) --
  {
    selector: 'node.exit',
    style: {
      'border-color': COLORS.exit,
      'border-width': 2,
    },
  },
  // -- External function (dashed amber border) --
  {
    selector: 'node.external',
    style: {
      'border-color': COLORS.external,
      'border-width': 2,
      'border-style': 'dashed',
    },
  },
  // -- Selected state (purple glow) --
  {
    selector: 'node.selected',
    style: {
      'border-color': COLORS.selected,
      'border-width': 3,
      'background-color': 'rgba(61, 155, 143, 0.12)',
    },
  },
  {
    selector: 'node:selected',
    style: {
      'border-color': COLORS.selected,
      'border-width': 3,
      'background-color': 'rgba(61, 155, 143, 0.12)',
    },
  },
  {
    selector: 'edge:selected',
    style: {
      'line-color': COLORS.selected,
      'target-arrow-color': COLORS.selected,
      'width': 3,
    },
  },
  // -- True branch (green) --
  {
    selector: 'edge.true-branch',
    style: {
      'line-color': COLORS.trueBranch,
      'target-arrow-color': COLORS.trueBranch,
      'label': 'T',
    },
  },
  // -- False branch (red) --
  {
    selector: 'edge.false-branch',
    style: {
      'line-color': COLORS.falseBranch,
      'target-arrow-color': COLORS.falseBranch,
      'label': 'F',
    },
  },
  // -- Back edge (loop, dashed amber going upward) --
  {
    selector: 'edge.back-edge',
    style: {
      'line-color': '#c49a3c',
      'target-arrow-color': '#c49a3c',
      'line-style': 'dashed',
      'width': 2,
      'curve-style': 'unbundled-bezier',
    },
  },
  // -- Store edge (amber) --
  {
    selector: 'edge.store',
    style: {
      'line-color': COLORS.store,
      'target-arrow-color': COLORS.store,
      'line-style': 'dashed',
    },
  },
  // -- Load edge (blue) --
  {
    selector: 'edge.load',
    style: {
      'line-color': COLORS.load,
      'target-arrow-color': COLORS.load,
      'line-style': 'dashed',
    },
  },
  // -- Define edge (green) --
  {
    selector: 'edge.define',
    style: {
      'line-color': COLORS.defineEdge,
      'target-arrow-color': COLORS.defineEdge,
    },
  },
  // -- Use edge (blue) --
  {
    selector: 'edge.use',
    style: {
      'line-color': COLORS.useEdge,
      'target-arrow-color': COLORS.useEdge,
    },
  },
  // -- Value node (blue, ellipse) --
  {
    selector: 'node.value-node',
    style: {
      'background-color': 'rgba(80, 136, 181, 0.08)',
      'border-color': COLORS.value,
      'shape': 'ellipse',
    },
  },
  // -- Location node (green, diamond) --
  {
    selector: 'node.location-node',
    style: {
      'background-color': 'rgba(61, 155, 143, 0.08)',
      'border-color': COLORS.location,
      'shape': 'diamond',
    },
  },
  // -- UnknownMem node (red) --
  {
    selector: 'node.unknown-mem',
    style: {
      'background-color': 'rgba(199, 80, 80, 0.08)',
      'border-color': COLORS.unknownMem,
      'border-style': 'dashed',
    },
  },
  // -- Instruction node (rectangle) --
  {
    selector: 'node.instruction',
    style: {
      'shape': 'rectangle',
      'background-color': 'rgba(80, 136, 181, 0.06)',
      'border-color': '#5088b5',
    },
  },
  // -- PTA value pointer node --
  {
    selector: 'node.pta-value',
    style: {
      'shape': 'roundrectangle',
      'background-color': 'rgba(80, 136, 181, 0.08)',
      'border-color': COLORS.value,
      'border-width': 2,
    },
  },
  // -- PTA location target node --
  {
    selector: 'node.pta-location',
    style: {
      'shape': 'ellipse',
      'background-color': 'rgba(61, 155, 143, 0.08)',
      'border-color': COLORS.location,
      'border-width': 2,
    },
  },
];

export function createCyInstance(
  container: HTMLElement,
  elements: ElementDefinition[],
  layoutName: string = 'preset',
  layoutOptions: Record<string, unknown> = {},
): Core {
  const cy = cytoscape({
    container,
    elements,
    style: stylesheet,
    layout: { name: layoutName, ...layoutOptions },
    userZoomingEnabled: true,
    userPanningEnabled: true,
    boxSelectionEnabled: false,
    minZoom: 0.1,
    maxZoom: 8,
    wheelSensitivity: 0.3,
  });

  return cy;
}
