import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import type { ArchitectureData, Connection } from './types.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const dataPath = join(__dirname, 'architecture.json');
const outputPath = join(__dirname, 'architecture.mmd');

const data: ArchitectureData = JSON.parse(readFileSync(dataPath, 'utf-8'));

const lines: string[] = [];

lines.push('graph TB');
lines.push('');

// --- Actors subgraph ---
lines.push('  subgraph actors["Actors"]');
for (const actor of data.actors) {
  if (actor.icon === 'robot') {
    lines.push(`    ${actor.id}{{"${actor.label}"}}`);
  } else {
    lines.push(`    ${actor.id}(["${actor.label}"])`);
  }
}
lines.push('  end');
lines.push('');

// --- Layer subgraphs ---
for (const layer of data.layers) {
  lines.push(`  subgraph ${layer.id}["${layer.label}"]`);
  if (layer.groups) {
    for (const group of layer.groups) {
      lines.push(`    subgraph ${group.id}["${group.label}"]`);
      if (group.hub) {
        const hubNode = layer.nodes.find((n) => n.id === group.hub);
        if (hubNode) lines.push(`      ${hubNode.id}["${hubNode.label}"]`);
      }
      for (const childId of group.children) {
        const child = layer.nodes.find((n) => n.id === childId);
        if (child) lines.push(`      ${child.id}["${child.label}"]`);
      }
      lines.push('    end');
    }
  } else {
    for (const node of layer.nodes) {
      lines.push(`    ${node.id}["${node.label}"]`);
    }
  }
  lines.push('  end');
  lines.push('');
}

// --- Connections ---
function arrowFor(type: Connection['type']): string {
  switch (type) {
    case 'uses':
      return '-->';
    case 'data':
      return '==>';
    case 'compiles-to':
      return '-.->';
    case 'deploys':
      return '-..->';
    case 'planned':
      return '-.->';
  }
}

for (const conn of data.connections) {
  const arrow = arrowFor(conn.type);
  if (conn.label) {
    // Escape parens/brackets that Mermaid interprets as node shapes
    const safeLabel = conn.label.replace(/[()[\]{}]/g, ' ').trim();
    lines.push(`  ${conn.from} ${arrow}|${safeLabel}| ${conn.to}`);
  } else {
    lines.push(`  ${conn.from} ${arrow} ${conn.to}`);
  }
}
lines.push('');

// --- Style directives ---
const DARK_BG = '#faf9f7';
const TEXT_COLOR = '#2c2c2e';

// Style actors with a neutral color
for (const actor of data.actors) {
  lines.push(`  style ${actor.id} fill:${DARK_BG},stroke:#9b9ba0,color:${TEXT_COLOR}`);
}

// Style each node with its layer color
for (const layer of data.layers) {
  for (const node of layer.nodes) {
    lines.push(`  style ${node.id} fill:${DARK_BG},stroke:${layer.color},color:${TEXT_COLOR}`);
  }
}

lines.push('');

const output = lines.join('\n');
writeFileSync(outputPath, output, 'utf-8');
console.log(`Wrote ${outputPath}`);
