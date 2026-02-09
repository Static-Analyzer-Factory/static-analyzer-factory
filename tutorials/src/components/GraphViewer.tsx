import { useEffect, useRef } from 'react';
import type { Core, ElementDefinition } from 'cytoscape';
import type { PropertyGraph } from '@saf/web-shared/types';
import type { GraphType } from '../content/types';
import {
  createCyInstance,
  renderCFG, cfgLayoutOptions,
  renderCallGraph, callgraphLayoutOptions,
  renderDefUse, defuseLayoutOptions,
  renderValueFlow, valueflowLayoutOptions,
} from '@saf/web-shared/graph';

interface GraphViewerProps {
  graph: PropertyGraph;
  graphType: GraphType;
  onNodeClick?: (lines: [number, number] | null) => void;
  className?: string;
}

export default function GraphViewer({ graph, graphType, onNodeClick, className }: GraphViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Destroy previous instance
    if (cyRef.current) {
      cyRef.current.destroy();
      cyRef.current = null;
    }

    let elements;
    let layoutName: string;
    let layoutOptions: Record<string, unknown> & { name: string };

    switch (graphType) {
      case 'cfg':
        elements = renderCFG(graph);
        layoutName = cfgLayoutOptions.name;
        layoutOptions = cfgLayoutOptions;
        break;
      case 'callgraph':
        elements = renderCallGraph(graph);
        layoutName = callgraphLayoutOptions.name;
        layoutOptions = callgraphLayoutOptions;
        break;
      case 'defuse':
        elements = renderDefUse(graph);
        layoutName = defuseLayoutOptions.name;
        layoutOptions = defuseLayoutOptions;
        break;
      case 'valueflow':
        elements = renderValueFlow(graph);
        layoutName = valueflowLayoutOptions.name;
        layoutOptions = valueflowLayoutOptions;
        break;
      case 'pta':
        elements = renderValueFlow(graph);
        layoutName = 'cose';
        layoutOptions = { name: 'cose', animate: false };
        break;
      case 'algorithm': {
        elements = graph.nodes.map(n => ({
          data: {
            id: n.id,
            label: n.properties?.name || n.id,
            highlighted: n.properties?.highlighted === true,
            nodeType: n.labels?.[0] || 'value',
          },
          classes: [
            n.labels?.[0] === 'location' ? 'alg-location' : 'alg-value',
            n.properties?.highlighted ? 'alg-highlighted' : '',
          ].filter(Boolean).join(' '),
        })) as ElementDefinition[];

        const edgeElements = graph.edges.map((e, i) => ({
          data: {
            id: `e${i}`,
            source: e.src,
            target: e.dst,
            label: e.properties?.label || '',
            highlighted: e.properties?.highlighted === true,
            edgeType: e.edge_type,
          },
          classes: [
            `alg-edge-${(e.edge_type || 'default').replace(/[^a-z0-9-]/gi, '')}`,
            e.properties?.highlighted ? 'alg-edge-highlighted' : '',
          ].filter(Boolean).join(' '),
        })) as ElementDefinition[];

        elements = [...elements, ...edgeElements];
        layoutName = 'dagre';
        layoutOptions = {
          name: 'dagre',
          rankDir: 'TB',
          nodeSep: 50,
          rankSep: 60,
          animate: false,
          fit: false,
          padding: 20,
        };
        break;
      }
    }

    const cy = createCyInstance(
      containerRef.current,
      elements,
      layoutName,
      layoutOptions,
    );

    // Wire up click handlers for source highlighting
    if (onNodeClick) {
      cy.on('tap', 'node', (evt) => {
        const lineStart = evt.target.data('lineStart') as number | undefined;
        const lineEnd = evt.target.data('lineEnd') as number | undefined;
        if (lineStart != null && lineEnd != null) {
          onNodeClick([lineStart, lineEnd]);
        }
      });
      cy.on('tap', (evt) => {
        if (evt.target === cy) {
          onNodeClick(null);
        }
      });
    }

    cyRef.current = cy;

    if (graphType === 'algorithm') {
      cy.style()
        .selector('.alg-value')
        .style({
          'width': 36,
          'height': 36,
          'background-color': '#3b82f6',
          'border-width': 2,
          'border-color': '#60a5fa',
          'color': '#2c2c2e',
          'font-size': '12px',
          'text-valign': 'center',
          'text-halign': 'center',
          'shape': 'round-rectangle',
          'label': 'data(label)',
        })
        .selector('.alg-location')
        .style({
          'width': 36,
          'height': 36,
          'background-color': '#065f46',
          'border-width': 2,
          'border-color': '#3d9b8f',
          'color': '#2c2c2e',
          'font-size': '12px',
          'text-valign': 'center',
          'text-halign': 'center',
          'shape': 'ellipse',
          'label': 'data(label)',
        })
        .selector('.alg-highlighted')
        .style({
          'border-width': 3,
          'border-color': '#c49a3c',
        })
        .selector('edge')
        .style({
          'width': 2,
          'curve-style': 'bezier',
          'target-arrow-shape': 'triangle',
          'arrow-scale': 0.8,
          'font-size': '10px',
          'text-rotation': 'autorotate',
          'label': 'data(label)',
          'color': '#a0aec0',
          'text-background-color': '#f1eeeb',
          'text-background-opacity': 0.8,
          'text-background-padding': '2px',
        })
        .selector('.alg-edge-points-to')
        .style({
          'line-color': '#3d9b8f',
          'target-arrow-color': '#3d9b8f',
        })
        .selector('.alg-edge-constraint')
        .style({
          'line-color': '#dbd6d0',
          'target-arrow-color': '#dbd6d0',
          'line-style': 'dashed',
        })
        .selector('.alg-edge-highlighted')
        .style({
          'line-color': '#c49a3c',
          'target-arrow-color': '#c49a3c',
          'width': 3,
        })
        .update();

      cy.layout(layoutOptions).run();
      // Fit graph to container but cap zoom so small graphs don't blow up
      cy.fit(undefined, 30);
      if (cy.zoom() > 1) {
        cy.zoom(1);
        cy.center();
      }
    }

    return () => {
      cy.destroy();
      cyRef.current = null;
    };
  }, [graph, graphType, onNodeClick]);

  return (
    <div
      ref={containerRef}
      className={`graph-viewer ${className ?? ''}`}
      style={{ minHeight: '300px', width: '100%' }}
    />
  );
}
