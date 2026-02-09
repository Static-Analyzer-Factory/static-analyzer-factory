/**
 * Compare AIR output from the tree-sitter CST-to-AIR converter vs the Rust inkwell frontend.
 *
 * Usage: npx tsx playground/src/analysis/__tests__/compare-air-output.ts <path-to-ll-file>
 *
 * This script:
 * 1. Runs the Rust frontend (via Docker) on a .ll file to get the "reference" AIR JSON
 * 2. Runs the tree-sitter CST-to-AIR converter on the same .ll file
 * 3. Compares the structural output: function names, instruction opcodes, operand counts,
 *    global variable names, etc.
 *
 * Note: IDs will differ (BLAKE3-based vs sequential), so we compare structure, not identity.
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { execSync } from 'child_process';
import { resolve, basename } from 'path';

const projectRoot = resolve(import.meta.dirname!, '../../../../');

interface AirModule {
  functions: AirFunction[];
  globals: AirGlobal[];
}

interface AirFunction {
  name: string;
  is_declaration: boolean;
  params: { name?: string | null; index?: number }[];
  blocks: AirBlock[];
}

interface AirBlock {
  label?: string | null;
  instructions: AirInstruction[];
}

interface AirInstruction {
  op: string;
  operands: string[];
  dst?: string | null;
  callee?: string;
  kind?: string;
  field_path?: { steps: { kind: string; index?: number }[] };
  size_bytes?: number;
}

interface AirGlobal {
  name: string;
  is_constant: boolean;
  init?: { kind: string } | null;
}

interface AirBundle {
  module: AirModule;
}

function getInkwellAir(llFile: string): AirBundle {
  const absPath = resolve(llFile);
  const relPath = absPath.replace(projectRoot + '/', '');
  const outFile = `/workspace/${relPath}.inkwell.air.json`;

  // Run the Rust CLI inside Docker to emit AIR JSON
  const cmd = `docker compose run --rm dev sh -c 'cargo run --release -p saf-cli -- air-dump /workspace/${relPath} > ${outFile} 2>/dev/null'`;
  try {
    execSync(cmd, { cwd: projectRoot, timeout: 120000, stdio: 'pipe' });
  } catch {
    console.error(`Failed to run inkwell frontend on ${llFile}`);
    console.error('Make sure Docker is running and the file exists.');
    process.exit(1);
  }

  const hostOut = resolve(projectRoot, `${relPath}.inkwell.air.json`);
  if (!existsSync(hostOut)) {
    console.error(`Inkwell AIR output not found at ${hostOut}`);
    process.exit(1);
  }
  const json = readFileSync(hostOut, 'utf-8');
  return JSON.parse(json);
}

async function getTreeSitterAir(llFile: string): Promise<AirBundle> {
  // Dynamic import to get the converter
  const { convertToAIR } = await import('@saf/web-shared/analysis');

  // Load tree-sitter WASM parser
  const Parser = (await import('web-tree-sitter')).default;
  await Parser.init();

  // Find the tree-sitter-llvm WASM file
  const wasmPaths = [
    resolve(projectRoot, 'playground/public/tree-sitter-llvm.wasm'),
    resolve(projectRoot, 'playground/node_modules/tree-sitter-llvm/tree-sitter-llvm.wasm'),
  ];

  let wasmPath = '';
  for (const p of wasmPaths) {
    if (existsSync(p)) {
      wasmPath = p;
      break;
    }
  }

  if (!wasmPath) {
    console.error('Could not find tree-sitter-llvm.wasm');
    console.error('Searched:', wasmPaths);
    process.exit(1);
  }

  const parser = new Parser();
  const lang = await Parser.Language.load(wasmPath);
  parser.setLanguage(lang);

  const source = readFileSync(resolve(llFile), 'utf-8');
  const tree = parser.parse(source);
  return convertToAIR(tree);
}

// ---------------------------------------------------------------------------
// Comparison logic
// ---------------------------------------------------------------------------

interface Diff {
  path: string;
  inkwell: string;
  treesitter: string;
}

function compareFunctions(inkwell: AirFunction[], ts: AirFunction[]): Diff[] {
  const diffs: Diff[] = [];

  // Compare by name
  const inkNames = new Set(inkwell.map(f => f.name));
  const tsNames = new Set(ts.map(f => f.name));

  for (const n of inkNames) {
    if (!tsNames.has(n)) diffs.push({ path: `functions`, inkwell: `has "${n}"`, treesitter: 'missing' });
  }
  for (const n of tsNames) {
    if (!inkNames.has(n)) diffs.push({ path: `functions`, inkwell: 'missing', treesitter: `has "${n}"` });
  }

  // Compare matching functions
  const inkByName = new Map(inkwell.map(f => [f.name, f]));
  const tsByName = new Map(ts.map(f => [f.name, f]));

  for (const [name, inkFn] of inkByName) {
    const tsFn = tsByName.get(name);
    if (!tsFn) continue;

    const prefix = `fn "${name}"`;

    // Compare is_declaration
    if (inkFn.is_declaration !== tsFn.is_declaration) {
      diffs.push({ path: `${prefix}.is_declaration`, inkwell: String(inkFn.is_declaration), treesitter: String(tsFn.is_declaration) });
    }

    // Compare param count
    if (inkFn.params.length !== tsFn.params.length) {
      diffs.push({ path: `${prefix}.params.length`, inkwell: String(inkFn.params.length), treesitter: String(tsFn.params.length) });
    }

    // Compare block count
    if (inkFn.blocks.length !== tsFn.blocks.length) {
      diffs.push({ path: `${prefix}.blocks.length`, inkwell: String(inkFn.blocks.length), treesitter: String(tsFn.blocks.length) });
    }

    // Compare instruction opcodes per block
    const minBlocks = Math.min(inkFn.blocks.length, tsFn.blocks.length);
    for (let b = 0; b < minBlocks; b++) {
      const inkBlock = inkFn.blocks[b];
      const tsBlock = tsFn.blocks[b];
      const bPrefix = `${prefix}.block[${b}]`;

      if ((inkBlock.label ?? 'entry') !== (tsBlock.label ?? 'entry')) {
        diffs.push({ path: `${bPrefix}.label`, inkwell: inkBlock.label ?? 'entry', treesitter: tsBlock.label ?? 'entry' });
      }

      if (inkBlock.instructions.length !== tsBlock.instructions.length) {
        diffs.push({
          path: `${bPrefix}.instructions.length`,
          inkwell: String(inkBlock.instructions.length),
          treesitter: String(tsBlock.instructions.length),
        });
      }

      const minInst = Math.min(inkBlock.instructions.length, tsBlock.instructions.length);
      for (let i = 0; i < minInst; i++) {
        const inkInst = inkBlock.instructions[i];
        const tsInst = tsBlock.instructions[i];
        const iPrefix = `${bPrefix}.inst[${i}]`;

        if (inkInst.op !== tsInst.op) {
          diffs.push({ path: `${iPrefix}.op`, inkwell: inkInst.op, treesitter: tsInst.op });
        }

        if (inkInst.operands.length !== tsInst.operands.length) {
          diffs.push({
            path: `${iPrefix}.operands.length`,
            inkwell: String(inkInst.operands.length),
            treesitter: String(tsInst.operands.length),
          });
        }

        // Compare has/doesn't-have dst
        const inkHasDst = inkInst.dst != null;
        const tsHasDst = tsInst.dst != null;
        if (inkHasDst !== tsHasDst) {
          diffs.push({ path: `${iPrefix}.has_dst`, inkwell: String(inkHasDst), treesitter: String(tsHasDst) });
        }

        // Compare kind (for binary_op, cast, heap_alloc)
        if (inkInst.kind !== tsInst.kind && (inkInst.kind || tsInst.kind)) {
          diffs.push({ path: `${iPrefix}.kind`, inkwell: inkInst.kind ?? 'none', treesitter: tsInst.kind ?? 'none' });
        }

        // Compare field_path
        if (inkInst.field_path || tsInst.field_path) {
          const inkSteps = JSON.stringify(inkInst.field_path?.steps ?? []);
          const tsSteps = JSON.stringify(tsInst.field_path?.steps ?? []);
          if (inkSteps !== tsSteps) {
            diffs.push({ path: `${iPrefix}.field_path`, inkwell: inkSteps, treesitter: tsSteps });
          }
        }

        // Compare size_bytes for alloca
        if (inkInst.op === 'alloca' || tsInst.op === 'alloca') {
          if (inkInst.size_bytes !== tsInst.size_bytes) {
            diffs.push({ path: `${iPrefix}.size_bytes`, inkwell: String(inkInst.size_bytes), treesitter: String(tsInst.size_bytes) });
          }
        }
      }
    }
  }

  return diffs;
}

function compareGlobals(inkwell: AirGlobal[], ts: AirGlobal[]): Diff[] {
  const diffs: Diff[] = [];

  const inkNames = new Set(inkwell.map(g => g.name));
  const tsNames = new Set(ts.map(g => g.name));

  for (const n of inkNames) {
    if (!tsNames.has(n)) diffs.push({ path: 'globals', inkwell: `has "${n}"`, treesitter: 'missing' });
  }
  for (const n of tsNames) {
    if (!inkNames.has(n)) diffs.push({ path: 'globals', inkwell: 'missing', treesitter: `has "${n}"` });
  }

  // Compare matching globals
  const inkByName = new Map(inkwell.map(g => [g.name, g]));
  const tsByName = new Map(ts.map(g => [g.name, g]));

  for (const [name, inkG] of inkByName) {
    const tsG = tsByName.get(name);
    if (!tsG) continue;

    if (inkG.is_constant !== tsG.is_constant) {
      diffs.push({ path: `global "${name}".is_constant`, inkwell: String(inkG.is_constant), treesitter: String(tsG.is_constant) });
    }

    const inkInitKind = inkG.init?.kind ?? 'none';
    const tsInitKind = tsG.init?.kind ?? 'none';
    if (inkInitKind !== tsInitKind) {
      diffs.push({ path: `global "${name}".init.kind`, inkwell: inkInitKind, treesitter: tsInitKind });
    }
  }

  return diffs;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.log('Usage: npx tsx playground/src/analysis/__tests__/compare-air-output.ts <path-to-ll-file>');
    console.log('\nExample .ll files:');
    console.log('  tests/fixtures/llvm/memory_ops.ll');
    console.log('  tests/fixtures/llvm/globals.ll');
    console.log('  tests/fixtures/llvm/calls.ll');
    process.exit(0);
  }

  const llFile = args[0];
  console.log(`\n=== AIR Comparison: ${basename(llFile)} ===\n`);

  console.log('1. Running inkwell (Rust) frontend...');
  const inkwellAir = getInkwellAir(llFile);

  console.log('2. Running tree-sitter CST-to-AIR converter...');
  const tsAir = await getTreeSitterAir(llFile);

  console.log('3. Comparing outputs...\n');

  const funcDiffs = compareFunctions(inkwellAir.module.functions, tsAir.module.functions);
  const globalDiffs = compareGlobals(inkwellAir.module.globals, tsAir.module.globals);
  const allDiffs = [...funcDiffs, ...globalDiffs];

  if (allDiffs.length === 0) {
    console.log('No structural differences found!');
  } else {
    console.log(`Found ${allDiffs.length} structural difference(s):\n`);
    for (const d of allDiffs) {
      console.log(`  ${d.path}`);
      console.log(`    inkwell:     ${d.inkwell}`);
      console.log(`    tree-sitter: ${d.treesitter}`);
      console.log();
    }
  }

  // Summary stats
  const inkFuncs = inkwellAir.module.functions;
  const tsFuncs = tsAir.module.functions;
  const inkInsts = inkFuncs.flatMap(f => f.blocks.flatMap(b => b.instructions));
  const tsInsts = tsFuncs.flatMap(f => f.blocks.flatMap(b => b.instructions));

  console.log('\n--- Summary ---');
  console.log(`Functions:    inkwell=${inkFuncs.length}  tree-sitter=${tsFuncs.length}`);
  console.log(`Globals:      inkwell=${inkwellAir.module.globals.length}  tree-sitter=${tsAir.module.globals.length}`);
  console.log(`Instructions: inkwell=${inkInsts.length}  tree-sitter=${tsInsts.length}`);
  console.log(`Differences:  ${allDiffs.length}`);

  // Optionally dump the full AIR JSON for manual inspection
  if (args.includes('--dump')) {
    const inkOut = `${llFile}.inkwell.air.json`;
    const tsOut = `${llFile}.treesitter.air.json`;
    writeFileSync(tsOut, JSON.stringify(tsAir, null, 2));
    console.log(`\nDumped: ${inkOut} (already exists from step 1)`);
    console.log(`Dumped: ${tsOut}`);
  }
}

main().catch(console.error);
