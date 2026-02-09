/**
 * CST-to-AIR converter.
 *
 * Converts a tree-sitter-llvm CST (from web-tree-sitter) into an AIR JSON
 * bundle that matches SAF's AIR JSON schema (air_json_schema.rs).
 *
 * The converter walks the CST produced by tree-sitter-llvm and emits
 * AirBundle JSON with hex-string IDs.
 */

import type { Node, Tree } from 'web-tree-sitter';
import type {
  AirBundle,
  AirModule,
  AirFunction,
  AirParam,
  AirBlock,
  AirInstruction,
  AirGlobal,
  AirFieldPath,
  AirConstant,
} from '../types';

type SyntaxNode = Node;

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

let idCounter = 0;

/** Generate a monotonically increasing hex ID (u128 format). */
function makeId(): string {
  idCounter++;
  return '0x' + idCounter.toString(16).padStart(32, '0');
}

/** Reset the ID counter (called at the start of each conversion). */
function resetIds(): void {
  idCounter = 0;
}

// ---------------------------------------------------------------------------
// Value resolution context
// ---------------------------------------------------------------------------

/** Tracks SSA value names to their assigned ValueIds within a function. */
class ValueContext {
  private localValues = new Map<string, string>();
  private globalValues: Map<string, string>;
  private functionIds: Map<string, string>;

  constructor(
    globalValues: Map<string, string>,
    functionIds: Map<string, string>,
  ) {
    this.globalValues = globalValues;
    this.functionIds = functionIds;
  }

  /** Resolve a local %name to a ValueId, creating one if needed. */
  resolveLocal(name: string): string {
    const existing = this.localValues.get(name);
    if (existing) return existing;
    const id = makeId();
    this.localValues.set(name, id);
    return id;
  }

  /** Resolve a global @name to a ValueId. */
  resolveGlobal(name: string): string {
    // If this is a function name, return its function ID directly.
    // PTA creates Addr constraints using ValueId::new(func.id.raw()),
    // so function address references must use the same ID as the function
    // for indirect call resolution to work.
    const funcId = this.functionIds.get(name);
    if (funcId) return funcId;

    const existing = this.globalValues.get(name);
    if (existing) return existing;
    const id = makeId();
    this.globalValues.set(name, id);
    return id;
  }

  /** Look up a function ID by name. */
  resolveFunctionId(name: string): string | undefined {
    return this.functionIds.get(name);
  }

  /** Get all local register → AIR ID mappings. */
  getLocalValues(): Map<string, string> {
    return this.localValues;
  }
}

// ---------------------------------------------------------------------------
// Top-level entry point
// ---------------------------------------------------------------------------

/** Result of CST-to-AIR conversion. */
export interface ConvertResult {
  air: AirBundle;
  /**
   * Maps LLVM register name (e.g., "%5") to AIR hex ID.
   * Collected from all function contexts during conversion.
   */
  registerMap: Map<string, string>;
}

/**
 * Convert a tree-sitter-llvm CST into an AIR JSON bundle.
 *
 * @param tree - The parsed tree from web-tree-sitter
 * @param instSourceLines - Optional map from positional key ("func:block:idx")
 *   to C source line number. When provided, AIR instructions get `span` fields
 *   populated so source line info flows through the analysis pipeline.
 * @returns An AirBundle matching the SAF AIR JSON schema, plus register mapping
 */
export function convertToAIR(
  tree: Tree,
  instSourceLines?: Record<string, number>,
): ConvertResult {
  resetIds();

  const root = tree.rootNode;
  const functions: AirFunction[] = [];
  const globals: AirGlobal[] = [];
  const globalValues = new Map<string, string>();
  const functionIds = new Map<string, string>();

  // First pass: collect function and global names so we can resolve cross-references.
  for (const child of root.children) {
    const t = child.type;
    if (t === 'fn_define' || t === 'define') {
      const name = extractFunctionName(child);
      if (name) {
        const id = makeId();
        functionIds.set(name, id);
      }
    } else if (t === 'declare') {
      const name = extractDeclareName(child);
      if (name) {
        const id = makeId();
        functionIds.set(name, id);
      }
    } else if (t === 'global_global' || t === 'global_variable') {
      const name = extractGlobalName(child);
      if (name) {
        const valId = makeId();
        globalValues.set(name, valId);
      }
    }
  }

  // Second pass: convert definitions.
  // Re-iterate to get a stable ordering. Use the pre-assigned IDs.
  const fnNameOrder: string[] = [];
  const declNameOrder: string[] = [];
  const globalNameOrder: string[] = [];
  const nodesByName = new Map<string, SyntaxNode>();

  for (const child of root.children) {
    const t = child.type;
    if (t === 'fn_define' || t === 'define') {
      const name = extractFunctionName(child);
      if (name) {
        fnNameOrder.push(name);
        nodesByName.set('fn:' + name, child);
      }
    } else if (t === 'declare') {
      const name = extractDeclareName(child);
      if (name) {
        declNameOrder.push(name);
        nodesByName.set('decl:' + name, child);
      }
    } else if (t === 'global_global' || t === 'global_variable') {
      const name = extractGlobalName(child);
      if (name) {
        globalNameOrder.push(name);
        nodesByName.set('global:' + name, child);
      }
    }
  }

  // Collect register → AIR ID mappings from all function contexts
  const registerMap = new Map<string, string>();

  for (const name of fnNameOrder) {
    const node = nodesByName.get('fn:' + name)!;
    const result = convertFunction(node, functionIds.get(name)!, globalValues, functionIds);
    if (result) {
      functions.push(result.func);
      // Collect register mappings (prefix with % for LLVM convention)
      for (const [reg, airId] of result.ctx.getLocalValues()) {
        registerMap.set(`%${reg}`, airId);
      }
    }
  }

  for (const name of declNameOrder) {
    const node = nodesByName.get('decl:' + name)!;
    const fn = convertDeclaration(node, functionIds.get(name)!);
    if (fn) functions.push(fn);
  }

  for (const name of globalNameOrder) {
    const node = nodesByName.get('global:' + name)!;
    const g = convertGlobal(node, globalValues, functionIds);
    if (g) globals.push(g);
  }

  // Populate span fields from instSourceLines if provided.
  // Walk all instructions and set span using positional key "func:block:idx".
  if (instSourceLines) {
    for (const func of functions) {
      if (func.is_declaration) continue;
      for (const block of func.blocks) {
        const blockLabel = block.label || 'entry';
        for (let i = 0; i < block.instructions.length; i++) {
          const posKey = `${func.name}:${blockLabel}:${i}`;
          const srcLine = instSourceLines[posKey];
          if (srcLine !== undefined) {
            block.instructions[i].span = {
              file_id: '0x00000000000000000000000000000000',
              byte_start: 0,
              byte_end: 0,
              line_start: srcLine,
              col_start: 0,
              line_end: srcLine,
              col_end: 0,
            };
          }
        }
      }
    }
  }

  const moduleId = makeId();
  const module: AirModule = {
    id: moduleId,
    name: null,
    functions,
    globals,
    source_files: [],
  };

  return {
    air: {
      frontend_id: 'air-json',
      schema_version: '0.1.0',
      module,
    },
    registerMap,
  };
}

// ---------------------------------------------------------------------------
// Name extraction helpers
// ---------------------------------------------------------------------------

/** Extract the function name from a fn_define / define node. */
function extractFunctionName(node: SyntaxNode): string | null {
  // The function name is a global_var child like @main
  const nameNode = findChildByType(node, 'global_var');
  if (nameNode) return stripGlobalPrefix(nameNode.text);

  // Fallback: look for function_header child, then global_var inside it
  const header = findChildByType(node, 'function_header');
  if (header) {
    const gv = findChildByType(header, 'global_var');
    if (gv) return stripGlobalPrefix(gv.text);
  }

  // Last resort: scan text with regex
  const match = node.text.match(/define\s+[\w\s*]*@([\w.$]+)/);
  return match ? match[1] : null;
}

/** Extract the function name from a declare node. */
function extractDeclareName(node: SyntaxNode): string | null {
  const nameNode = findChildByType(node, 'global_var');
  if (nameNode) return stripGlobalPrefix(nameNode.text);

  const match = node.text.match(/declare\s+[\w\s*]*@([\w.$]+)/);
  return match ? match[1] : null;
}

/** Extract the global variable name from a global_global node. */
function extractGlobalName(node: SyntaxNode): string | null {
  const nameNode = findChildByType(node, 'global_var');
  if (nameNode) return stripGlobalPrefix(nameNode.text);

  const match = node.text.match(/@([\w.$]+)\s*=/);
  return match ? match[1] : null;
}

/** Strip the @ prefix from a global variable name. */
function stripGlobalPrefix(text: string): string {
  return text.startsWith('@') ? text.slice(1) : text;
}

/** Strip the % prefix from a local variable name. */
function stripLocalPrefix(text: string): string {
  return text.startsWith('%') ? text.slice(1) : text;
}

// ---------------------------------------------------------------------------
// Function conversion
// ---------------------------------------------------------------------------

function convertFunction(
  node: SyntaxNode,
  funcId: string,
  globalValues: Map<string, string>,
  functionIds: Map<string, string>,
): { func: AirFunction; ctx: ValueContext } | null {
  const name = extractFunctionName(node);
  if (!name) return null;

  const ctx = new ValueContext(globalValues, functionIds);

  // Extract parameters
  const params = extractParams(node, ctx);

  // Extract basic blocks from function_body
  const bodyNode = findChildByType(node, 'function_body');
  const blocks: AirBlock[] = [];
  const blockIdMap = new Map<string, string>(); // label -> BlockId

  if (bodyNode) {
    // First pass: assign block IDs
    const blockNodes = collectBlocks(bodyNode);
    for (const bn of blockNodes) {
      const label = extractBlockLabel(bn);
      const blockId = makeId();
      blockIdMap.set(label, blockId);
    }

    // Second pass: convert instructions
    for (const bn of blockNodes) {
      const label = extractBlockLabel(bn);
      const block = convertBlock(bn, label, blockIdMap.get(label)!, blockIdMap, ctx);
      blocks.push(block);
    }
  }

  const entryBlock = blocks.length > 0 ? blocks[0].id ?? null : null;

  return {
    func: {
      id: funcId,
      name,
      params,
      blocks,
      entry_block: entryBlock,
      is_declaration: false,
      span: null,
      symbol: null,
    },
    ctx,
  };
}

function convertDeclaration(
  node: SyntaxNode,
  funcId: string,
): AirFunction | null {
  const name = extractDeclareName(node);
  if (!name) return null;

  // Extract parameters from the declaration
  const params: AirParam[] = [];
  const paramTypes = extractDeclareParamTypes(node);
  for (let i = 0; i < paramTypes.length; i++) {
    params.push({
      id: makeId(),
      name: null,
      index: i,
    });
  }

  return {
    id: funcId,
    name,
    params,
    blocks: [],
    entry_block: null,
    is_declaration: true,
    span: null,
    symbol: null,
  };
}

function extractDeclareParamTypes(node: SyntaxNode): string[] {
  // Try to parse parameter types from the declare text
  const match = node.text.match(/\(([^)]*)\)/);
  if (!match) return [];
  const inner = match[1].trim();
  if (!inner || inner === '...') return [];
  return inner.split(',').map((s: string) => s.trim()).filter((s: string) => s && s !== '...');
}

// ---------------------------------------------------------------------------
// Parameter extraction
// ---------------------------------------------------------------------------

function extractParams(node: SyntaxNode, ctx: ValueContext): AirParam[] {
  const params: AirParam[] = [];

  // Look for argument / argument_list nodes
  const argList = findChildByType(node, 'argument_list')
    ?? findChildByType(node, 'func_params');

  if (argList) {
    let index = 0;
    for (const child of argList.children) {
      if (child.type === 'argument' || child.type === 'func_param') {
        const paramName = extractParamName(child);
        const paramId = paramName
          ? ctx.resolveLocal(paramName)
          : makeId();
        params.push({
          id: paramId,
          name: paramName ?? null,
          index,
        });
        index++;
      } else if (child.type === 'type_and_value') {
        // Some grammars inline type+value for params
        const paramName = extractValueName(child);
        const paramId = paramName
          ? ctx.resolveLocal(paramName)
          : makeId();
        params.push({
          id: paramId,
          name: paramName ?? null,
          index,
        });
        index++;
      }
    }
  }

  // Fallback: parse from text if no structured children found
  if (params.length === 0) {
    const headerText = node.text;
    const parenMatch = headerText.match(/\(([^)]*)\)/);
    if (parenMatch) {
      const inner = parenMatch[1].trim();
      if (inner && inner !== '...') {
        const parts = inner.split(',');
        let index = 0;
        for (const part of parts) {
          const trimmed = part.trim();
          if (trimmed === '...' || !trimmed) continue;
          // Try to extract %name from "type %name"
          const nameMatch = trimmed.match(/%([\w.$]+)/);
          const paramName = nameMatch ? nameMatch[1] : null;
          const paramId = paramName
            ? ctx.resolveLocal(paramName)
            : makeId();
          params.push({
            id: paramId,
            name: paramName,
            index,
          });
          index++;
        }
      }
    }
  }

  return params;
}

function extractParamName(node: SyntaxNode): string | null {
  const localVar = findChildByType(node, 'local_var')
    ?? findChildByType(node, 'var');
  if (localVar) return stripLocalPrefix(localVar.text);

  const match = node.text.match(/%([\w.$]+)/);
  return match ? match[1] : null;
}

// ---------------------------------------------------------------------------
// Block extraction
// ---------------------------------------------------------------------------

/**
 * Collect basic block groupings from a function_body.
 *
 * In tree-sitter-llvm, the function body contains label nodes and
 * instruction nodes. Instructions following a label belong to that block.
 * Instructions before any label belong to the implicit entry block.
 */
function collectBlocks(bodyNode: SyntaxNode): SyntaxNode[][] {
  const blocks: SyntaxNode[][] = [];
  let current: SyntaxNode[] = [];
  let hasLabel = false;

  for (const child of bodyNode.children) {
    if (child.type === '{' || child.type === '}') continue;

    if (child.type === 'label') {
      if (current.length > 0 || hasLabel) {
        blocks.push(current);
      }
      current = [child];
      hasLabel = true;
    } else if (child.type.startsWith('instruction_') || child.type === 'instruction') {
      if (!hasLabel && current.length === 0) {
        // Implicit entry block; add a synthetic label marker
        hasLabel = true;
      }
      current.push(child);
    } else {
      // Other nodes (comments, metadata) -- skip or include
      current.push(child);
    }
  }

  if (current.length > 0 || hasLabel) {
    blocks.push(current);
  }

  // If no blocks found but there are children, create a single implicit block
  if (blocks.length === 0) {
    const instrs = bodyNode.children.filter(
      c => c.type.startsWith('instruction_') || c.type === 'instruction',
    );
    if (instrs.length > 0) {
      blocks.push(instrs);
    }
  }

  return blocks;
}

function extractBlockLabel(blockNodes: SyntaxNode[]): string {
  if (blockNodes.length === 0) return 'entry';

  const first = blockNodes[0];
  if (first.type === 'label') {
    // Label text is like "name:" or "%name:"
    const text = first.text.replace(/:$/, '').trim();
    return stripLocalPrefix(text);
  }

  return 'entry';
}

function convertBlock(
  blockNodes: SyntaxNode[],
  label: string,
  blockId: string,
  blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirBlock {
  const instructions: AirInstruction[] = [];

  for (const node of blockNodes) {
    if (node.type === 'label') continue; // Already handled
    if (node.type.startsWith('instruction_') || node.type === 'instruction') {
      const inst = convertInstruction(node, blockIdMap, ctx);
      if (inst) instructions.push(inst);
    }
  }

  return {
    id: blockId,
    label,
    instructions,
  };
}

// ---------------------------------------------------------------------------
// Instruction conversion
// ---------------------------------------------------------------------------

function convertInstruction(
  node: SyntaxNode,
  blockIdMap: Map<string, string>,
  ctx: ValueContext,
  outerDst?: string | null,
): AirInstruction | null {
  const instType = node.type;

  // Check for destination (result) assignment: %name = ...
  // If outerDst is provided (from a wrapper `instruction` node), use it
  // instead of re-extracting — the inner node may not have the `%name = ` prefix.
  const dst = outerDst !== undefined ? outerDst : extractInstructionDst(node, ctx);

  switch (instType) {
    case 'instruction_alloca':
      return convertAlloca(node, dst, ctx);
    case 'instruction_load':
      return convertLoad(node, dst, ctx);
    case 'instruction_store':
      return convertStore(node, ctx);
    case 'instruction_getelementptr':
      return convertGep(node, dst, ctx);
    case 'instruction_call':
      return convertCall(node, dst, blockIdMap, ctx);
    case 'instruction_ret':
      return convertRet(node, ctx);
    case 'instruction_br':
      return convertBr(node, blockIdMap, ctx);
    case 'instruction_switch':
      return convertSwitch(node, blockIdMap, ctx);
    case 'instruction_phi':
      return convertPhi(node, dst, blockIdMap, ctx);
    case 'instruction_select':
      return convertSelect(node, dst, ctx);
    case 'instruction_icmp':
      return convertICmp(node, dst, ctx);
    case 'instruction_fcmp':
      return convertFCmp(node, dst, ctx);
    case 'instruction_bin_op':
      return convertBinOp(node, dst, ctx);
    case 'instruction_cast':
      return convertCast(node, dst, ctx);
    case 'instruction_freeze':
      return convertFreeze(node, dst, ctx);
    case 'instruction_invoke':
      return convertInvoke(node, dst, blockIdMap, ctx);
    case 'instruction_landingpad':
      // Exception landing pad → opaque value (PTA doesn't track exceptions)
      return { id: makeId(), op: 'copy', operands: [], dst: dst ?? makeId() };
    case 'instruction_resume':
      // Exception resume → unreachable (doesn't return normally)
      return { id: makeId(), op: 'unreachable', operands: [], dst: null };
    case 'instruction_unreachable':
      return { id: makeId(), op: 'unreachable', operands: [], dst: null };
    case 'instruction_extractvalue':
      return convertExtractValue(node, dst, ctx);
    case 'instruction_insertvalue':
      return convertInsertValue(node, dst, ctx);
    case 'instruction': {
      // Generic instruction wrapper -- look at first meaningful child.
      // Pass the dst we extracted from the outer node so it isn't lost
      // (the inner node may not include the `%name = ` prefix).
      const inner = node.children.find(c => c.type.startsWith('instruction_'));
      if (inner) return convertInstruction(inner, blockIdMap, ctx, dst);
      return null;
    }
    default:
      // Unsupported instruction -- emit a copy with no operands
      if (instType.startsWith('instruction_')) {
        console.warn(`[cst-to-air] Unsupported instruction type: ${instType}`);
      }
      return null;
  }
}

// ---------------------------------------------------------------------------
// Instruction destination (LHS of assignment)
// ---------------------------------------------------------------------------

function extractInstructionDst(
  node: SyntaxNode,
  ctx: ValueContext,
): string | null {
  // Pattern: %name = instruction ...
  // In tree-sitter-llvm, the local_var is typically a child of the instruction node
  // preceding the '=' token.
  const text = node.text;
  const match = text.match(/^\s*%([\w.$]+)\s*=/);
  if (match) {
    return ctx.resolveLocal(match[1]);
  }

  // Also check for unnamed temps like %0, %1
  const numMatch = text.match(/^\s*%(\d+)\s*=/);
  if (numMatch) {
    return ctx.resolveLocal(numMatch[1]);
  }

  return null;
}

// ---------------------------------------------------------------------------
// Individual instruction converters
// ---------------------------------------------------------------------------

function convertAlloca(
  node: SyntaxNode,
  dst: string | null,
  _ctx: ValueContext,
): AirInstruction {
  // Parse the allocated type to compute size_bytes
  const sizeBytes = allocaTypeSize(node.text);

  return {
    id: makeId(),
    op: 'alloca',
    operands: [],
    dst: dst ?? makeId(),
    size_bytes: sizeBytes,
  };
}

/** Compute allocation size in bytes from an alloca instruction's type. */
function allocaTypeSize(text: string): number | undefined {
  // Match: alloca <type>, ... or alloca inalloca <type>, ...
  const m = text.match(/alloca\s+(?:inalloca\s+)?(.+)/);
  if (!m) return undefined;
  // Take the type part before any comma (align, count, etc.)
  const rest = m[1];
  // The type ends at the first comma (which separates align/count) or end of string
  const commaIdx = rest.indexOf(',');
  const typeStr = (commaIdx >= 0 ? rest.slice(0, commaIdx) : rest).trim();
  return typeSize(typeStr);
}

/** Compute the size in bytes for an LLVM IR type. */
function typeSize(t: string): number | undefined {
  const s = t.trim();
  // Primitive types
  const primitives: Record<string, number> = {
    i1: 1, i8: 1, i16: 2, i32: 4, i64: 8, i128: 16,
    float: 4, double: 8, ptr: 8,
  };
  if (s in primitives) return primitives[s];
  // Array type: [N x T]
  const arrMatch = s.match(/^\[(\d+)\s+x\s+(.+)\]$/);
  if (arrMatch) {
    const count = parseInt(arrMatch[1], 10);
    const elemSize = typeSize(arrMatch[2]);
    return elemSize !== undefined ? count * elemSize : undefined;
  }
  // Struct/unknown types: return undefined (unknown size)
  return undefined;
}

function convertLoad(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  // `load type, ptr %p` — after resolvePartOperand, the type filters to null
  // and the pointer is the first (and typically only) resolved operand.
  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'load',
    operands: operands.length > 0 ? [operands[0]] : [],
    dst: dst ?? makeId(),
  };
}

function convertStore(
  node: SyntaxNode,
  ctx: ValueContext,
): AirInstruction {
  // store type value, type* ptr → operands[0]=value, operands[1]=pointer
  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'store',
    operands: operands.length >= 2 ? [operands[0], operands[1]] : operands,
    dst: null,
  };
}

function convertGep(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const operands = extractValueOperands(node, ctx);
  const fieldPath = extractFieldPath(node);

  // Include ALL operands: base pointer + index operands.
  // The base pointer is operands[0]; index operands follow.
  // For constant indices not captured by extractValueOperands, they are
  // already represented in the field_path.
  return {
    id: makeId(),
    op: 'gep',
    operands,
    dst: dst ?? makeId(),
    field_path: fieldPath,
  };
}

function convertCall(
  node: SyntaxNode,
  dst: string | null,
  _blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;

  // Try to find the callee function name
  const calleeMatch = text.match(/@([\w.$]+)\s*\(/);

  if (calleeMatch) {
    const calleeName = calleeMatch[1];
    const calleeId = ctx.resolveFunctionId(calleeName);

    // Check for heap allocation functions
    const heapFuncs: Record<string, string> = {
      malloc: 'malloc',
      calloc: 'calloc',
      realloc: 'realloc',
      'operator new': 'new',
      _Znwm: 'new',
      _Znam: 'new[]',
    };

    if (calleeName in heapFuncs) {
      const argOperands = extractCallArgOperands(node, ctx);
      return {
        id: makeId(),
        op: 'heap_alloc',
        operands: argOperands,
        dst: dst ?? makeId(),
        kind: heapFuncs[calleeName],
      };
    }

    // Check for memcpy/memmove/memset
    if (calleeName.startsWith('llvm.memcpy') || calleeName === 'memcpy'
        || calleeName.startsWith('llvm.memmove') || calleeName === 'memmove') {
      const argOperands = extractCallArgOperands(node, ctx);
      return {
        id: makeId(),
        op: 'memcpy',
        operands: argOperands,
        dst: null,
      };
    }
    if (calleeName.startsWith('llvm.memset') || calleeName === 'memset') {
      const argOperands = extractCallArgOperands(node, ctx);
      return {
        id: makeId(),
        op: 'memset',
        operands: argOperands,
        dst: null,
      };
    }

    if (calleeId) {
      const argOperands = extractCallArgOperands(node, ctx);
      return {
        id: makeId(),
        op: 'call_direct',
        operands: argOperands,
        dst: dst ?? null,
        callee: calleeId,
      };
    }
  }

  // Indirect call (function pointer or no recognized callee)
  const argOperands = extractCallArgOperands(node, ctx);
  // Extract the function pointer: the %reg or @name immediately before the
  // opening parenthesis of the argument list.  SAF uses callee-LAST convention
  // (last operand is the function pointer; all preceding are arguments).
  const funcPtrId = extractIndirectCallee(text, ctx);
  const operands = funcPtrId
    ? [...argOperands, funcPtrId]
    : argOperands;

  return {
    id: makeId(),
    op: 'call_indirect',
    operands,
    dst: dst ?? null,
  };
}

function convertInvoke(
  node: SyntaxNode,
  dst: string | null,
  _blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirInstruction {
  // invoke is like call but with exception handling:
  // %res = invoke retty @callee(args) to label %normal unwind label %exception
  // We emit just the call instruction; the branch is implicit from CFG.
  const text = node.text;

  const calleeMatch = text.match(/@([\w.$]+)\s*\(/);
  if (calleeMatch) {
    const calleeName = calleeMatch[1];
    const calleeId = ctx.resolveFunctionId(calleeName);

    if (calleeId) {
      const argOperands = extractCallArgOperands(node, ctx);
      return {
        id: makeId(),
        op: 'call_direct',
        operands: argOperands,
        dst: dst ?? null,
        callee: calleeId,
      };
    }
  }

  // Indirect invoke — same callee-LAST convention as call_indirect
  const argOperands = extractCallArgOperands(node, ctx);
  const funcPtrId = extractIndirectCallee(text, ctx);
  const operands = funcPtrId
    ? [...argOperands, funcPtrId]
    : argOperands;

  return {
    id: makeId(),
    op: 'call_indirect',
    operands,
    dst: dst ?? null,
  };
}

function convertRet(
  node: SyntaxNode,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text.trim();
  // "ret void" has no operand
  if (text.match(/ret\s+void/)) {
    return { id: makeId(), op: 'ret', operands: [], dst: null };
  }

  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'ret',
    operands,
    dst: null,
  };
}

function convertBr(
  node: SyntaxNode,
  blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text.trim();

  // Conditional branch: br i1 %cond, label %then, label %else
  const condMatch = text.match(
    /br\s+i1\s+(%[\w.$]+|true|false|\d+),\s*label\s+%([\w.$]+),\s*label\s+%([\w.$]+)/,
  );
  if (condMatch) {
    const condOperand = resolveTextOperand(condMatch[1], ctx);
    const thenLabel = condMatch[2];
    const elseLabel = condMatch[3];
    const thenId = blockIdMap.get(thenLabel) ?? makeId();
    const elseId = blockIdMap.get(elseLabel) ?? makeId();

    return {
      id: makeId(),
      op: 'cond_br',
      operands: [condOperand],
      dst: null,
      then_target: thenId,
      else_target: elseId,
    };
  }

  // Unconditional branch: br label %target
  const brMatch = text.match(/br\s+label\s+%([\w.$]+)/);
  if (brMatch) {
    const targetLabel = brMatch[1];
    const targetId = blockIdMap.get(targetLabel) ?? makeId();
    return {
      id: makeId(),
      op: 'br',
      operands: [],
      dst: null,
      target: targetId,
    };
  }

  // Fallback
  return { id: makeId(), op: 'br', operands: [], dst: null, target: makeId() };
}

function convertSwitch(
  node: SyntaxNode,
  blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text.trim();

  // switch i32 %val, label %default [ i32 0, label %case0  i32 1, label %case1 ]
  const headerMatch = text.match(
    /switch\s+\w+\s+(%[\w.$]+|\d+),\s*label\s+%([\w.$]+)/,
  );

  const switchOperand = headerMatch
    ? resolveTextOperand(headerMatch[1], ctx)
    : extractValueOperands(node, ctx)[0] ?? makeId();

  const defaultLabel = headerMatch ? headerMatch[2] : 'default';
  const defaultId = blockIdMap.get(defaultLabel) ?? makeId();

  // Extract cases
  const cases: [number, string][] = [];
  const casePattern = /(\w+)\s+(-?\d+),\s*label\s+%([\w.$]+)/g;
  let caseMatch;
  // Skip the first type (it's the switch operand type), iterate rest
  const bracketContent = text.match(/\[([^\]]*)\]/);
  if (bracketContent) {
    while ((caseMatch = casePattern.exec(bracketContent[1])) !== null) {
      const val = parseInt(caseMatch[2], 10);
      const label = caseMatch[3];
      const blockId = blockIdMap.get(label) ?? makeId();
      cases.push([val, blockId]);
    }
  }

  return {
    id: makeId(),
    op: 'switch',
    operands: [switchOperand],
    dst: null,
    default: defaultId,
    cases,
  };
}

function convertPhi(
  node: SyntaxNode,
  dst: string | null,
  blockIdMap: Map<string, string>,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;

  // phi type [ val1, %bb1 ], [ val2, %bb2 ], ...
  // Use nesting-aware parsing so aggregate values like { i32 0, ptr null }
  // don't break on internal commas.
  const incoming: [string, string][] = [];
  const pairs = extractPhiPairs(text);
  for (const [valText, blockLabel] of pairs) {
    const valueId = resolveTextOperand(valText, ctx);
    const blockId = blockIdMap.get(blockLabel) ?? makeId();
    incoming.push([blockId, valueId]);
  }

  return {
    id: makeId(),
    op: 'phi',
    operands: [],
    dst: dst ?? makeId(),
    incoming,
  };
}

function convertSelect(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'select',
    operands,
    dst: dst ?? makeId(),
  };
}

function convertICmp(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;
  const predMatch = text.match(/icmp\s+(eq|ne|ugt|uge|ult|ule|sgt|sge|slt|sle)/);
  const pred = predMatch ? predMatch[1] : 'eq';

  const binaryOpKind = icmpPredToAir(pred);
  const operands = extractValueOperands(node, ctx);

  return {
    id: makeId(),
    op: 'binary_op',
    operands,
    dst: dst ?? makeId(),
    kind: binaryOpKind,
  };
}

function convertFCmp(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;
  const predMatch = text.match(/fcmp\s+(oeq|one|ogt|oge|olt|ole|ord|uno|ueq|une|ugt|uge|ult|ule|true|false)/);
  const pred = predMatch ? predMatch[1] : 'oeq';

  const binaryOpKind = fcmpPredToAir(pred);
  const operands = extractValueOperands(node, ctx);

  return {
    id: makeId(),
    op: 'binary_op',
    operands,
    dst: dst ?? makeId(),
    kind: binaryOpKind,
  };
}

function convertBinOp(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;
  // Extract the operation: add, sub, mul, udiv, sdiv, urem, srem, etc.
  const opMatch = text.match(
    /\b(add|sub|mul|udiv|sdiv|urem|srem|fadd|fsub|fmul|fdiv|frem|and|or|xor|shl|lshr|ashr)\b/,
  );
  const op = opMatch ? opMatch[1] : 'add';
  const binaryOpKind = llvmBinOpToAir(op);
  const operands = extractValueOperands(node, ctx);

  return {
    id: makeId(),
    op: 'binary_op',
    operands,
    dst: dst ?? makeId(),
    kind: binaryOpKind,
  };
}

function convertCast(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const text = node.text;
  const castMatch = text.match(
    /\b(trunc|zext|sext|fptoui|fptosi|uitofp|sitofp|fptrunc|fpext|ptrtoint|inttoptr|bitcast|addrspacecast)\b/,
  );
  const castOp = castMatch ? castMatch[1] : 'bitcast';
  const castKind = llvmCastToAir(castOp);
  const operands = extractValueOperands(node, ctx);

  return {
    id: makeId(),
    op: 'cast',
    operands,
    dst: dst ?? makeId(),
    kind: castKind,
  };
}

function convertFreeze(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'freeze',
    operands,
    dst: dst ?? makeId(),
  };
}

function convertExtractValue(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  // extractvalue maps to GEP with field path
  const operands = extractValueOperands(node, ctx);
  const fieldPath = extractFieldPathFromIndices(node);

  return {
    id: makeId(),
    op: 'gep',
    operands: operands.length > 0 ? [operands[0]] : [],
    dst: dst ?? makeId(),
    field_path: fieldPath,
  };
}

function convertInsertValue(
  node: SyntaxNode,
  dst: string | null,
  ctx: ValueContext,
): AirInstruction {
  // insertvalue is modeled as a copy for pointer analysis purposes
  const operands = extractValueOperands(node, ctx);
  return {
    id: makeId(),
    op: 'copy',
    operands,
    dst: dst ?? makeId(),
  };
}

// ---------------------------------------------------------------------------
// Operand extraction
// ---------------------------------------------------------------------------

/**
 * Extract value operands from an instruction node **in positional order**.
 *
 * Splits the instruction text by top-level commas, then resolves one
 * value per comma-separated part.  This preserves the order in which
 * operands appear in the LLVM IR text, handling SSA references (%name,
 * @name), literal constants (null, undef, true, false, zeroinitializer,
 * poison), and bare numeric/float constants.
 */
function extractValueOperands(node: SyntaxNode, ctx: ValueContext): string[] {
  const text = node.text;

  // Strip the destination assignment ("  %x = ...")
  const assignMatch = text.match(/=\s*(.*)/s);
  const rhs = assignMatch ? assignMatch[1] : text;

  // Remove the instruction keyword and optional flags
  // (e.g., "add nsw nuw i32 %a, %b" → " i32 %a, %b")
  const withoutKeyword = rhs.replace(
    /^\s*(alloca|load|store|getelementptr|call|invoke|ret|br|switch|phi|select|icmp|fcmp|add|sub|mul|udiv|sdiv|urem|srem|fadd|fsub|fmul|fdiv|frem|and|or|xor|shl|lshr|ashr|trunc|zext|sext|fptoui|fptosi|uitofp|sitofp|fptrunc|fpext|ptrtoint|inttoptr|bitcast|addrspacecast|freeze|extractvalue|insertvalue|extractelement|insertelement|shufflevector)\b/,
    '',
  );

  // Remove optional flags that appear after the keyword
  // (e.g., " nsw nuw i32 %a, %b" → " i32 %a, %b")
  const withoutFlags = withoutKeyword.replace(
    /^\s*(?:(?:inbounds|volatile|atomic|nsw|nuw|nnan|ninf|nsz|arcp|contract|afn|reassoc|fast|exact)\s+)*/,
    '',
  );

  // Split by top-level commas (respects nesting in {} and [])
  const parts = splitTopLevelCommas(withoutFlags);

  const operands: string[] = [];
  for (const part of parts) {
    const resolved = resolvePartOperand(part.trim(), ctx);
    if (resolved !== null) {
      operands.push(resolved);
    }
  }

  return operands;
}

/**
 * Resolve a single comma-separated part to a ValueId, or null if the
 * part is a bare type / label / keyword with no value.
 *
 * Expected patterns (after keyword removal):
 *   "i32 %x"        → resolveLocal(x)
 *   "ptr @g"         → resolveGlobal(g)
 *   "i32 42"         → makeId()  (numeric constant)
 *   "ptr null"       → makeId()  (literal constant)
 *   "i1 true"        → makeId()  (boolean literal)
 *   "double 1.5e+2"  → makeId()  (float constant)
 *   "label %blk"     → null      (label, not a value)
 *   "%struct.S"      → null      (type reference, not a value)
 */
export function resolvePartOperand(part: string, ctx: ValueContext): string | null {
  if (!part) return null;

  // Skip label operands (e.g., "label %then")
  if (/^\s*label\b/.test(part)) return null;

  // Skip bare type-only parts (e.g., "i32", "ptr") with no trailing value
  // These appear as the type prefix in instructions like "load i32, ptr %p"
  // where "i32" is the result type, not an operand.
  if (/^\s*(i\d+|float|double|half|fp128|x86_fp80|ppc_fp128|void|ptr|opaque)\s*$/.test(part)) return null;

  // Skip bare type references (%struct.*, %class.*, %union.*) that have no
  // trailing value.  A part like "%struct.S %x" contains an actual value (%x)
  // and must NOT be filtered — only bare "%struct.S" or "%struct.S*" without
  // a subsequent SSA name should be skipped.
  if (/^\s*%?(struct|class|union)\.[\w.$*]*\s*$/.test(part)) return null;

  // Strip optional attribute keywords and cast destination types
  // (e.g., "ptr %p nonnull", "ptr noundef %p", "i64 %x to ptr")
  const cleaned = part
    .replace(/\bto\s+(?:i\d+|float|double|half|fp128|x86_fp80|ppc_fp128|ptr|void)\s*$/, '')
    .replace(/\b(nonnull|noundef|signext|zeroext|inreg|byval|sret|align\s+\d+)\b/g, '')
    .trim();

  // Try to find a value reference at the end: "type %name", "type @name",
  // "type 42", "type null", "type 1.5e+2", etc.
  // Pattern: optional type prefix, then the value
  const valueMatch = cleaned.match(
    /(?:.*\s)?(%[\w.$]+|@[\w.$]+|null|undef|true|false|zeroinitializer|poison|-?\d+(?:\.\d+)?(?:e[+-]?\d+)?)\s*$/,
  );

  if (!valueMatch) return null;

  const val = valueMatch[1];

  // Filter out type references that look like values
  if (val.startsWith('%')) {
    const name = stripLocalPrefix(val);
    if (/^(struct|class|union)\./.test(name)) return null;
    return ctx.resolveLocal(name);
  }
  if (val.startsWith('@')) {
    return ctx.resolveGlobal(stripGlobalPrefix(val));
  }

  // Literal or numeric constant → fresh opaque ID
  return makeId();
}

/**
 * Extract the function pointer (callee) from an indirect call/invoke.
 *
 * In LLVM IR, the callee appears immediately before the argument list:
 *   `call i32 %fptr(i32 %x, i32 %y)`  →  callee = %fptr
 *   `call void @thunk(ptr %p)`          →  callee = @thunk (but this is direct)
 *
 * Returns the resolved ValueId, or undefined if no callee is found.
 */
export function extractIndirectCallee(text: string, ctx: ValueContext): string | undefined {
  // Match %localvar or @globalvar immediately before '('
  const m = text.match(/(%[\w.$]+|@[\w.$]+)\s*\(/);
  if (!m) return undefined;

  const callee = m[1];
  if (callee.startsWith('%')) {
    return ctx.resolveLocal(stripLocalPrefix(callee));
  }
  if (callee.startsWith('@')) {
    return ctx.resolveGlobal(stripGlobalPrefix(callee));
  }
  return undefined;
}

/**
 * Extract call argument operands (inside the argument-list parentheses).
 *
 * Uses nesting-aware parenthesis matching so that constant expressions
 * like `bitcast(ptr @bar to ptr)` inside arguments don't break parsing.
 */
function extractCallArgOperands(node: SyntaxNode, ctx: ValueContext): string[] {
  const text = node.text;
  const operands: string[] = [];

  // Find the callee token (@func or %ptr) and the opening paren
  const calleeMatch = text.match(/(?:@[\w.$]+|%[\w.$]+)\s*\(/);
  if (!calleeMatch) return [];

  // Find the start of the argument list (after the opening paren)
  const openIdx = text.indexOf('(', calleeMatch.index!);
  if (openIdx < 0) return [];

  // Walk forward with depth tracking to find the matching close paren
  const argsText = extractBalancedParens(text, openIdx);
  if (!argsText || !argsText.trim()) return [];

  // Split by top-level commas within the argument list
  const args = splitTopLevelCommas(argsText);
  for (const arg of args) {
    const resolved = resolvePartOperand(arg.trim(), ctx);
    if (resolved !== null) {
      operands.push(resolved);
    }
  }

  return operands;
}

/**
 * Extract the content between balanced parentheses starting at `text[openIdx]`.
 * Returns the inner text (excluding the outer parens), or null if unbalanced.
 */
export function extractBalancedParens(text: string, openIdx: number): string | null {
  if (text[openIdx] !== '(') return null;
  let depth = 1;
  let i = openIdx + 1;
  while (i < text.length && depth > 0) {
    if (text[i] === '(') depth++;
    else if (text[i] === ')') depth--;
    if (depth > 0) i++;
  }
  if (depth !== 0) return null;
  return text.slice(openIdx + 1, i);
}

/** LLVM IR literal constants that should be treated as opaque values. */
const LLVM_LITERALS = new Set(['null', 'undef', 'true', 'false', 'zeroinitializer', 'poison']);

/** Resolve a text-based operand reference to a ValueId. */
function resolveTextOperand(text: string, ctx: ValueContext): string {
  const trimmed = text.trim();
  if (trimmed.startsWith('%')) {
    return ctx.resolveLocal(stripLocalPrefix(trimmed));
  }
  if (trimmed.startsWith('@')) {
    return ctx.resolveGlobal(stripGlobalPrefix(trimmed));
  }
  // LLVM IR literal constants → fresh opaque IDs
  if (LLVM_LITERALS.has(trimmed)) {
    return makeId();
  }
  // Numeric constant or other literal
  return makeId();
}

/** Extract a value name from a type_and_value node. */
function extractValueName(node: SyntaxNode): string | null {
  const localVar = findChildByType(node, 'local_var')
    ?? findChildByType(node, 'var');
  if (localVar) return stripLocalPrefix(localVar.text);

  const match = node.text.match(/%([\w.$]+)/);
  return match ? match[1] : null;
}

// ---------------------------------------------------------------------------
// Field path extraction (for GEP)
// ---------------------------------------------------------------------------

function extractFieldPath(node: SyntaxNode): AirFieldPath {
  const text = node.text;
  const steps: AirFieldPath['steps'] = [];

  // GEP indices appear after the base pointer, comma-separated
  // getelementptr inbounds %struct.S, %struct.S* %s, i32 0, i32 1
  // The first index is always an array/pointer index
  // Subsequent indices are struct field indices

  const indices = extractGepIndices(text);
  for (let i = 0; i < indices.length; i++) {
    if (i === 0) {
      // First index is always a pointer/array index
      steps.push({ kind: 'index' });
    } else {
      const idx = indices[i];
      if (typeof idx === 'number') {
        steps.push({ kind: 'field', index: idx });
      } else {
        // Variable index => array index
        steps.push({ kind: 'index' });
      }
    }
  }

  return { steps };
}

function extractGepIndices(text: string): (number | string)[] {
  const indices: (number | string)[] = [];

  // Split by commas only at top level (not inside {} or [] nesting)
  const parts = splitTopLevelCommas(text);
  // Skip first two parts (instruction keyword + type, and base pointer)
  for (let i = 2; i < parts.length; i++) {
    const part = parts[i].trim();
    const numMatch = part.match(/\w+\s+(-?\d+)/);
    if (numMatch) {
      indices.push(parseInt(numMatch[1], 10));
    } else {
      const varMatch = part.match(/%[\w.$]+/);
      if (varMatch) {
        indices.push(varMatch[0]);
      }
    }
  }

  return indices;
}

/** Split text by commas only at the top level (outside {} and [] nesting). */
function splitTopLevelCommas(text: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '{' || ch === '[' || ch === '(') {
      depth++;
    } else if (ch === '}' || ch === ']' || ch === ')') {
      depth--;
    } else if (ch === ',' && depth === 0) {
      parts.push(text.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(text.slice(start));
  return parts;
}

/**
 * Extract PHI incoming pairs with nesting-aware comma handling.
 *
 * For `phi type [ val1, %bb1 ], [ { i32 0, ptr null }, %bb2 ]`, this
 * correctly handles commas inside aggregate values by tracking `{}` depth.
 * Returns an array of [valueText, blockLabel] pairs.
 */
export function extractPhiPairs(text: string): [string, string][] {
  const pairs: [string, string][] = [];

  // Find each [ ... ] pair
  let i = 0;
  while (i < text.length) {
    const openBracket = text.indexOf('[', i);
    if (openBracket < 0) break;

    // Walk forward to find the matching ']' respecting nesting
    let depth = 1;
    let j = openBracket + 1;
    while (j < text.length && depth > 0) {
      if (text[j] === '[') depth++;
      else if (text[j] === ']') depth--;
      if (depth > 0) j++;
    }
    if (depth !== 0) break;

    const inner = text.slice(openBracket + 1, j).trim();

    // Find the comma separating value from block label, respecting {} depth.
    // The block label is always the LAST comma-separated token: %blockname
    let braceDepth = 0;
    let lastTopComma = -1;
    for (let k = 0; k < inner.length; k++) {
      if (inner[k] === '{') braceDepth++;
      else if (inner[k] === '}') braceDepth--;
      else if (inner[k] === ',' && braceDepth === 0) {
        lastTopComma = k;
      }
    }

    if (lastTopComma >= 0) {
      const valText = inner.slice(0, lastTopComma).trim();
      const blockText = inner.slice(lastTopComma + 1).trim();
      const blockMatch = blockText.match(/%([\w.$]+)/);
      if (blockMatch) {
        pairs.push([valText, blockMatch[1]]);
      }
    }

    i = j + 1;
  }

  return pairs;
}

function extractFieldPathFromIndices(node: SyntaxNode): AirFieldPath {
  const text = node.text;
  const steps: AirFieldPath['steps'] = [];

  // extractvalue { type, type } %val, 0, 1
  // Indices are numeric after the value operand
  const parts = text.split(',');
  for (let i = 1; i < parts.length; i++) {
    const numMatch = parts[i].trim().match(/^(\d+)$/);
    if (numMatch) {
      steps.push({ kind: 'field', index: parseInt(numMatch[1], 10) });
    }
  }

  return { steps };
}

// ---------------------------------------------------------------------------
// Global conversion
// ---------------------------------------------------------------------------

function convertGlobal(
  node: SyntaxNode,
  globalValues: Map<string, string>,
  functionIds: Map<string, string>,
): AirGlobal | null {
  const name = extractGlobalName(node);
  if (!name) return null;

  const valId = globalValues.get(name) ?? makeId();
  const objId = makeId();

  const text = node.text;
  const isConstant = /\bconstant\b/.test(text);

  // Extract the initializer portion after `global`/`constant` keyword + type
  const init = parseGlobalInit(text, globalValues, functionIds);

  return {
    id: valId,
    obj: objId,
    name,
    init,
    is_constant: isConstant,
    span: null,
  };
}

/** Parse a global variable initializer from its text. */
function parseGlobalInit(
  text: string,
  globalValues: Map<string, string>,
  functionIds: Map<string, string>,
): AirConstant | null {
  if (/zeroinitializer/.test(text)) {
    return { kind: 'zero_init' };
  }
  if (/\bundef\b/.test(text)) {
    return { kind: 'undef' };
  }

  // Global pointer reference: @gptr = global ptr @target
  // Also handles constant expressions: @gptr = global ptr bitcast(ptr @func to ptr)
  const globalRefMatch = text.match(/=\s*(?:global|constant)\s+ptr\s+(?:@([\w.$]+)|(?:bitcast|inttoptr|getelementptr|addrspacecast)\s*\([^)]*@([\w.$]+)[^)]*\))/);
  if (globalRefMatch) {
    const targetName = globalRefMatch[1] ?? globalRefMatch[2];
    // Use function ID if available (for function pointers), else global value ID
    const targetId = functionIds.get(targetName)
      ?? globalValues.get(targetName)
      ?? makeId();
    return { kind: 'global_ref', '0': targetId };
  }

  // Null pointer
  if (/=\s*(?:global|constant)\s+ptr\s+null\b/.test(text)) {
    return { kind: 'null' };
  }
  // General null
  if (/\bnull\b/.test(text)) {
    return { kind: 'null' };
  }

  // String constant: c"hello\00"
  const strMatch = text.match(/c"([^"]*)"/);
  if (strMatch) {
    // Unescape common LLVM string escapes
    const raw = strMatch[1].replace(/\\00/g, '').replace(/\\0A/g, '\n');
    return { kind: 'string', value: raw };
  }

  // Aggregate initializer: { i32 1, ptr @func, ... }
  const aggMatch = text.match(/=\s*(?:global|constant)\s+(?:%[\w.$]+|{[^}]*})\s*(\{[^}]*\})/);
  if (aggMatch) {
    const elements = parseAggregateElements(aggMatch[1], globalValues, functionIds);
    if (elements.length > 0) {
      return { kind: 'aggregate', elements };
    }
  }

  // Numeric constant
  const numMatch = text.match(/=\s*(?:global|constant)\s+\w+\s+(-?\d+)/);
  if (numMatch) {
    return { kind: 'int', value: parseInt(numMatch[1], 10), bits: 32 };
  }

  return null;
}

/** Parse aggregate initializer elements like { i32 1, ptr @func }. */
function parseAggregateElements(
  text: string,
  globalValues: Map<string, string>,
  functionIds: Map<string, string>,
): AirConstant[] {
  const elements: AirConstant[] = [];
  // Remove outer braces
  const inner = text.replace(/^\s*\{/, '').replace(/\}\s*$/, '').trim();
  if (!inner) return elements;

  // Split by commas (simple — doesn't handle nested aggregates)
  const parts = inner.split(',');
  for (const part of parts) {
    const trimmed = part.trim();
    // ptr @name → global_ref
    const refMatch = trimmed.match(/ptr\s+@([\w.$]+)/);
    if (refMatch) {
      const targetId = functionIds.get(refMatch[1])
        ?? globalValues.get(refMatch[1])
        ?? makeId();
      elements.push({ kind: 'global_ref', '0': targetId });
      continue;
    }
    // ptr null
    if (/ptr\s+null/.test(trimmed)) {
      elements.push({ kind: 'null' });
      continue;
    }
    // integer: i32 42
    const intMatch = trimmed.match(/i\d+\s+(-?\d+)/);
    if (intMatch) {
      elements.push({ kind: 'int', value: parseInt(intMatch[1], 10), bits: 32 });
      continue;
    }
    // float/double
    const floatMatch = trimmed.match(/(float|double)\s+([\d.eE+-]+)/);
    if (floatMatch) {
      elements.push({ kind: 'float', value: parseFloat(floatMatch[2]), bits: floatMatch[1] === 'float' ? 32 : 64 });
      continue;
    }
    // Unknown element → null placeholder
    elements.push({ kind: 'null' });
  }
  return elements;
}

// ---------------------------------------------------------------------------
// Binary op / cast mappings
// ---------------------------------------------------------------------------

function icmpPredToAir(pred: string): string {
  const map: Record<string, string> = {
    eq: 'i_cmp_eq',
    ne: 'i_cmp_ne',
    ugt: 'i_cmp_ugt',
    uge: 'i_cmp_uge',
    ult: 'i_cmp_ult',
    ule: 'i_cmp_ule',
    sgt: 'i_cmp_sgt',
    sge: 'i_cmp_sge',
    slt: 'i_cmp_slt',
    sle: 'i_cmp_sle',
  };
  return map[pred] ?? 'i_cmp_eq';
}

function fcmpPredToAir(pred: string): string {
  const map: Record<string, string> = {
    // Ordered predicates
    oeq: 'f_cmp_oeq',
    one: 'f_cmp_one',
    ogt: 'f_cmp_ogt',
    oge: 'f_cmp_oge',
    olt: 'f_cmp_olt',
    ole: 'f_cmp_ole',
    // Unordered predicates → map to nearest ordered equivalent
    ueq: 'f_cmp_oeq',
    une: 'f_cmp_one',
    ugt: 'f_cmp_ogt',
    uge: 'f_cmp_oge',
    ult: 'f_cmp_olt',
    ule: 'f_cmp_ole',
    // Ordering predicates
    ord: 'f_cmp_oeq',
    uno: 'f_cmp_one',
    // Boolean predicates
    true: 'f_cmp_oeq',
    false: 'f_cmp_one',
  };
  return map[pred] ?? 'f_cmp_oeq';
}

function llvmBinOpToAir(op: string): string {
  const map: Record<string, string> = {
    add: 'add',
    sub: 'sub',
    mul: 'mul',
    udiv: 'u_div',
    sdiv: 's_div',
    urem: 'u_rem',
    srem: 's_rem',
    fadd: 'f_add',
    fsub: 'f_sub',
    fmul: 'f_mul',
    fdiv: 'f_div',
    frem: 'f_rem',
    and: 'and',
    or: 'or',
    xor: 'xor',
    shl: 'shl',
    lshr: 'l_shr',
    ashr: 'a_shr',
  };
  return map[op] ?? 'add';
}

function llvmCastToAir(cast: string): string {
  const map: Record<string, string> = {
    trunc: 'trunc',
    zext: 'z_ext',
    sext: 's_ext',
    fptoui: 'fp_to_ui',
    fptosi: 'fp_to_si',
    uitofp: 'ui_to_fp',
    sitofp: 'si_to_fp',
    fptrunc: 'fp_trunc',
    fpext: 'fp_ext',
    ptrtoint: 'ptr_to_int',
    inttoptr: 'int_to_ptr',
    bitcast: 'bitcast',
    addrspacecast: 'addr_space_cast',
  };
  return map[cast] ?? 'bitcast';
}

// ---------------------------------------------------------------------------
// Tree-sitter helpers
// ---------------------------------------------------------------------------

/** Find the first child of a node with a given type. */
function findChildByType(node: SyntaxNode, type: string): SyntaxNode | null {
  for (const child of node.children) {
    if (child.type === type) return child;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Exports for testing
// ---------------------------------------------------------------------------

export {
  allocaTypeSize,
  typeSize,
  fcmpPredToAir,
  icmpPredToAir,
  extractGepIndices,
  splitTopLevelCommas,
  resolveTextOperand,
  makeId,
  resetIds,
  ValueContext,
};
