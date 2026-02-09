/** Compiler Explorer (Godbolt) API integration for compiling C to LLVM IR. */

const GODBOLT_CLANG = 'https://godbolt.org/api/compiler/cclang1810/compile';
const GODBOLT_OPT = 'https://godbolt.org/api/compiler/opt1810/compile';

/** Result of compilation including optional variable name metadata. */
export interface CompileResult {
  ir: string;
  /**
   * Maps LLVM register name (e.g., "%1") to C variable name(s) (e.g., "p").
   * Extracted from debug info before metadata is stripped.
   * For mem2reg IR, maps post-mem2reg registers to variable names
   * by tracing store chains from the -O0 IR.
   */
  varNames: Record<string, string[]>;
  /**
   * Maps "funcName:blockLabel" to [minLine, maxLine] source line ranges.
   * Extracted from LLVM debug metadata before stripping.
   */
  blockSourceLines: Record<string, [number, number]>;
  /**
   * Maps "funcName:blockLabel:instIdx" to the C source line number.
   * instIdx is the 0-based position among non-debug instructions in each block,
   * matching what tree-sitter sees in the stripped IR.
   */
  instSourceLines: Record<string, number>;
}

export async function compileToLLVM(
  source: string,
  mem2reg = true,
): Promise<CompileResult> {
  // Step 1: Compile C to LLVM IR at -O0 with debug info for variable names.
  const clangResp = await fetch(GODBOLT_CLANG, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
    body: JSON.stringify({
      source,
      options: {
        userArguments: '-S -emit-llvm -O0 -g',
        compilerOptions: { skipAsm: false },
        filters: { binary: false, execute: false, directives: false },
      },
    }),
  });

  if (!clangResp.ok) {
    throw new Error(
      `Compiler Explorer returned ${clangResp.status}: ${clangResp.statusText}`,
    );
  }

  const clangData = await clangResp.json();
  if (clangData.code !== 0) {
    const stderr =
      clangData.stderr
        ?.map((l: { text: string }) => l.text)
        .join('\n') || 'Unknown compilation error';
    throw new Error(stderr);
  }

  let ir: string = clangData.asm
    .map((l: { text: string }) => l.text)
    .join('\n');

  // Extract variable name mappings from debug metadata (before stripping)
  const debugInfo = extractDebugVarNames(ir);

  if (!mem2reg) {
    // Without mem2reg, extract source lines and strip debug for the parser
    const { blockLines: blockSourceLines, instLines: instSourceLines } = extractSourceLines(ir);
    ir = stripDebugMetadata(ir);
    return { ir, varNames: debugInfo.allocaVarNames, blockSourceLines, instSourceLines };
  }

  // Step 2: Strip `optnone` attribute (added by -O0) so passes can run,
  // then run mem2reg via Godbolt's opt tool.
  // Keep debug metadata intact — opt handles it natively, and we need it
  // to extract block→source line mappings with correct post-mem2reg labels.
  ir = ir.replace(/ optnone/g, '');

  const optResp = await fetch(GODBOLT_OPT, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
    body: JSON.stringify({
      source: ir,
      options: {
        userArguments: '-passes=mem2reg -S',
        compilerOptions: { skipAsm: false },
        filters: { binary: false, execute: false, directives: false },
      },
    }),
  });

  if (!optResp.ok) {
    throw new Error(
      `opt mem2reg failed: ${optResp.status} ${optResp.statusText}`,
    );
  }

  const optData = await optResp.json();
  if (optData.code !== 0) {
    const stderr =
      optData.stderr
        ?.map((l: { text: string }) => l.text)
        .join('\n') || 'opt mem2reg failed';
    throw new Error(stderr);
  }

  const postIrWithDebug: string = optData.asm
    .map((l: { text: string }) => l.text)
    .join('\n');

  // Extract block→source line ranges and per-instruction source lines from
  // post-mem2reg IR (block labels match what tree-sitter parser will see)
  const { blockLines: blockSourceLines, instLines: instSourceLines } = extractSourceLines(postIrWithDebug);

  // Strip debug metadata for the tree-sitter parser
  const postIr = stripDebugMetadata(postIrWithDebug);

  // Map pre-mem2reg value registers to post-mem2reg registers by matching
  // instruction patterns (instruction text minus LHS register name).
  const varNames = mapVarNamesToPostMem2reg(
    debugInfo.valueVarNames,
    debugInfo.valueDefPatterns,
    postIr,
  );

  return { ir: postIr, varNames, blockSourceLines, instSourceLines };
}

// ---------------------------------------------------------------------------
// Debug info extraction
// ---------------------------------------------------------------------------

interface DebugVarInfo {
  /** Maps alloca register (e.g., "%2") → variable names. For non-mem2reg use. */
  allocaVarNames: Record<string, string[]>;
  /** Maps value register (e.g., "%5") → variable names. For mem2reg use. */
  valueVarNames: Record<string, string[]>;
  /** Maps value register → the instruction text that defines it (for pattern matching). */
  valueDefPatterns: Record<string, string>;
}

/**
 * Extract C variable names from LLVM debug metadata.
 *
 * Parses `#dbg_declare` / `@llvm.dbg.declare` records and `!DILocalVariable`
 * metadata to build alloca→varname and value→varname mappings.
 */
function extractDebugVarNames(ir: string): DebugVarInfo {
  const allocaVarNames: Record<string, string[]> = {};
  const valueVarNames: Record<string, string[]> = {};
  const valueDefPatterns: Record<string, string> = {};

  // 1. Parse metadata: !N = !DILocalVariable(name: "x", ...)
  const metaVars: Record<string, string> = {};
  for (const m of ir.matchAll(/^(!\d+)\s*=\s*!DILocalVariable\(name:\s*"([^"]+)"/gm)) {
    metaVars[m[1]] = m[2];
  }

  // 2. Parse #dbg_declare(ptr %N, !M, !DIExpression())
  for (const m of ir.matchAll(/#dbg_declare\(\s*ptr\s+(%\S+)\s*,\s*(!\d+)/gm)) {
    const reg = m[1].replace(/,\s*$/, '');
    const meta = m[2];
    if (metaVars[meta]) {
      (allocaVarNames[reg] ??= []).push(metaVars[meta]);
    }
  }

  // 3. Also handle old-style: call void @llvm.dbg.declare(metadata ptr %N, metadata !M, ...)
  for (const m of ir.matchAll(
    /call void @llvm\.dbg\.declare\(metadata\s+ptr\s+(%\S+)\s*,\s*metadata\s+(!\d+)/gm,
  )) {
    const reg = m[1].replace(/,\s*$/, '');
    const meta = m[2];
    if (metaVars[meta]) {
      (allocaVarNames[reg] ??= []).push(metaVars[meta]);
    }
  }

  // 4. For each alloca with a variable name, find the first `store val, ptr %alloca`
  //    to determine which value register represents the variable.
  //    Also follow loads to handle `q = p` patterns.
  const allocaToValueReg: Record<string, string> = {};
  const storeRe = /store\s+\S+\s+(%\S+)\s*,\s*ptr\s+(%\S+)/g;
  for (const m of ir.matchAll(storeRe)) {
    const valueReg = m[1].replace(/,\s*$/, '');
    const ptrReg = m[2].replace(/,\s*$/, '');
    // Only record the FIRST store to each alloca
    if (allocaVarNames[ptrReg] && !allocaToValueReg[ptrReg]) {
      allocaToValueReg[ptrReg] = valueReg;
    }
  }

  // 5. Resolve value chains: if a value is a `load ptr, ptr %alloca`, follow it
  const loadMap: Record<string, string> = {};
  for (const m of ir.matchAll(/(%\S+)\s*=\s*load\s+ptr\s*,\s*ptr\s+(%\S+)/g)) {
    loadMap[m[1]] = m[2].replace(/,\s*$/, ''); // %result → %source_alloca
  }

  // 6. Build value→varname map by following the store chain
  for (const [allocaReg, names] of Object.entries(allocaVarNames)) {
    let valueReg = allocaToValueReg[allocaReg];
    if (!valueReg) continue;

    // If the value is a load from another alloca, follow to that alloca's value
    const visited = new Set<string>();
    while (loadMap[valueReg] && allocaToValueReg[loadMap[valueReg]] && !visited.has(valueReg)) {
      visited.add(valueReg);
      valueReg = allocaToValueReg[loadMap[valueReg]];
    }

    for (const name of names) {
      (valueVarNames[valueReg] ??= []).push(name);
    }
  }

  // 7. Record defining instruction text for each value register (for pattern matching)
  for (const m of ir.matchAll(/(%\S+)\s*=\s*(.+)/g)) {
    const reg = m[1];
    if (valueVarNames[reg]) {
      // Store the RHS (instruction text without the LHS register)
      const rhs = m[2].replace(/,?\s*!dbg\s+!\d+/g, '').trim();
      if (!valueDefPatterns[reg]) {
        valueDefPatterns[reg] = rhs;
      }
    }
  }

  return { allocaVarNames, valueVarNames, valueDefPatterns };
}

/**
 * Strip all LLVM debug metadata from IR text.
 */
function stripDebugMetadata(ir: string): string {
  return (
    ir
      // Remove #dbg_declare / #dbg_value record intrinsics (full lines)
      .replace(/^\s*#dbg_\w+\([^)]*\).*$/gm, '')
      // Remove @llvm.dbg.declare / @llvm.dbg.value calls (full lines, with optional tail/musttail)
      .replace(/^\s*(?:(?:tail|musttail|notail)\s+)?call void @llvm\.dbg\.\w+\([^)]*\).*$/gm, '')
      // Remove !dbg !N references from instructions
      .replace(/,?\s*!dbg\s+!\d+/g, '')
      // Remove other instruction-level metadata like !tbaa, !range, !llvm.loop
      .replace(/,?\s*![\w.]+\s+!\d+/g, '')
      // Remove metadata definitions (lines like !0 = ...)
      .replace(/^!\d+\s*=\s*.*$/gm, '')
      // Remove named metadata (like !llvm.dbg.cu = ...)
      .replace(/^!llvm\.\S+\s*=\s*.*$/gm, '')
      // Remove @llvm.dbg.* declarations
      .replace(/^declare\s+void\s+@llvm\.dbg\..*$/gm, '')
      // Remove attributes referencing debug (like "di-forward-declarations")
      // Clean up excess blank lines
      .replace(/\n{3,}/g, '\n\n')
  );
}

/**
 * Map variable names to post-mem2reg registers by matching instruction patterns.
 *
 * After mem2reg, register numbers change. We match instructions by their
 * text pattern (RHS of the assignment, ignoring register numbers) to find
 * which post-mem2reg register corresponds to which pre-mem2reg value.
 */
function mapVarNamesToPostMem2reg(
  valueVarNames: Record<string, string[]>,
  valueDefPatterns: Record<string, string>,
  postIr: string,
): Record<string, string[]> {
  const result: Record<string, string[]> = {};

  // Build post-mem2reg instruction patterns: register → RHS text
  const postPatterns: [string, string][] = [];
  for (const m of postIr.matchAll(/(%\S+)\s*=\s*(.+)/g)) {
    postPatterns.push([m[1], m[2].trim()]);
  }

  // For each pre-mem2reg value with variable names, find matching post-mem2reg instruction
  for (const [preReg, names] of Object.entries(valueVarNames)) {
    const prePattern = valueDefPatterns[preReg];
    if (!prePattern) continue;

    // Normalize pattern: replace register references with wildcards for matching
    const normalized = normalizePattern(prePattern);

    for (const [postReg, postRhs] of postPatterns) {
      if (normalizePattern(postRhs) === normalized) {
        for (const name of names) {
          (result[postReg] ??= []).push(name);
        }
        break; // first match wins
      }
    }
  }

  return result;
}

/**
 * Normalize an instruction pattern for fuzzy matching.
 * Replaces all register references (%5, %call, %struct.Point) and
 * attribute group references (#2, #3) with placeholders so patterns
 * can match between pre- and post-mem2reg IR.
 */
function normalizePattern(text: string): string {
  return text
    .replace(/%[\w.]+/g, '%_')   // %5, %call, %struct.Point → %_
    .replace(/#\d+/g, '#_')      // #2, #3 → #_
    .replace(/\s+/g, ' ')
    .trim();
}

// ---------------------------------------------------------------------------
// Block → source line extraction
// ---------------------------------------------------------------------------

/**
 * Check if an IR line is a debug intrinsic that gets stripped before parsing.
 * These lines are removed by stripDebugMetadata so they don't appear in the
 * tree-sitter-parsed IR, and must be skipped when counting instruction indices.
 */
function isDebugIntrinsic(line: string): boolean {
  const trimmed = line.trim();
  // #dbg_declare(...), #dbg_value(...), etc.
  if (trimmed.startsWith('#dbg_')) return true;
  // call void @llvm.dbg.declare(...), call void @llvm.dbg.value(...), etc.
  // May be prefixed with tail/musttail/notail
  if (/^(?:(?:tail|musttail|notail)\s+)?call void @llvm\.dbg\./.test(trimmed)) return true;
  return false;
}

/**
 * Check if an IR line is an instruction (something that will be parsed by
 * tree-sitter after stripping). Must be indented (not a label, metadata def,
 * or blank line) and not a debug intrinsic.
 */
function isInstruction(line: string): boolean {
  // Must be indented (starts with whitespace) and non-empty
  if (!line.match(/^\s+\S/)) return false;
  // Skip debug intrinsics
  if (isDebugIntrinsic(line)) return false;
  // Skip comment-only lines
  if (line.trim().startsWith(';')) return false;
  return true;
}

/**
 * Extract block-level and instruction-level source line info from LLVM IR.
 *
 * Parses `!DILocation(line: N, ...)` and `!dbg !N` annotations to compute:
 * - Block ranges: min/max C source lines for each basic block
 * - Instruction lines: per-instruction C source line, keyed by
 *   "funcName:blockLabel:instIdx" where instIdx matches tree-sitter order
 *
 * @returns blockLines keyed by "funcName:blockLabel" → [minLine, maxLine],
 *          instLines keyed by "funcName:blockLabel:instIdx" → source line.
 */
function extractSourceLines(ir: string): {
  blockLines: Record<string, [number, number]>;
  instLines: Record<string, number>;
} {
  // 1. Parse !N = !DILocation(line: L, ...) → metaIdToLine
  const metaIdToLine: Record<string, number> = {};
  for (const m of ir.matchAll(/^(!\d+)\s*=\s*!DILocation\(line:\s*(\d+)/gm)) {
    metaIdToLine[m[1]] = parseInt(m[2], 10);
  }

  // 2. Walk through IR, tracking function/block context
  const blockLines: Record<string, [number, number]> = {};
  const instLines: Record<string, number> = {};
  let currentFunc = '';
  let currentBlock = '';
  let blockMin = Infinity;
  let blockMax = -Infinity;
  let instIdx = 0;

  const flushBlock = () => {
    if (currentFunc && currentBlock && blockMin <= blockMax) {
      const key = `${currentFunc}:${currentBlock}`;
      blockLines[key] = [blockMin, blockMax];
    }
    blockMin = Infinity;
    blockMax = -Infinity;
    instIdx = 0;
  };

  for (const line of ir.split('\n')) {
    // Match function definition: define ... @funcName(
    const funcMatch = line.match(/^define\s+.*@([\w.$]+)\s*\(/);
    if (funcMatch) {
      flushBlock();
      currentFunc = funcMatch[1];
      currentBlock = 'entry'; // implicit first block
      continue;
    }

    // Match basic block label at column 0: "name:" or "5:"
    const blockMatch = line.match(/^([\w.]+)\s*:/);
    if (blockMatch && currentFunc) {
      flushBlock();
      currentBlock = blockMatch[1];
      continue;
    }

    // Process instruction lines within a function/block
    if (currentFunc && currentBlock) {
      // Check if this is an instruction that tree-sitter will see
      if (isInstruction(line)) {
        // Extract source line from !dbg annotation if present
        const dbgMatch = line.match(/!dbg\s+(!\d+)/);
        if (dbgMatch && metaIdToLine[dbgMatch[1]] !== undefined) {
          const srcLine = metaIdToLine[dbgMatch[1]];
          if (srcLine > 0) { // skip line 0 (compiler-generated)
            blockMin = Math.min(blockMin, srcLine);
            blockMax = Math.max(blockMax, srcLine);
            // Record per-instruction source line
            const instKey = `${currentFunc}:${currentBlock}:${instIdx}`;
            instLines[instKey] = srcLine;
          }
        }
        instIdx++;
      }
    }

    // End of function
    if (line.trim() === '}') {
      flushBlock();
      currentFunc = '';
      currentBlock = '';
    }
  }

  return { blockLines, instLines };
}
