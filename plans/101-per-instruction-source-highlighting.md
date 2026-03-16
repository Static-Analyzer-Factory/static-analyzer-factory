# Plan 101: Per-Instruction Source Line Highlighting

## Status: done

## Context

The previous implementation (Plan 100) mapped every instruction in a block to the block's `[minLine, maxLine]` range. In single-block functions (like `main` in the Pointer Aliasing example), clicking any Def-Use, Value Flow, or PTA node highlighted the entire function body (lines 9-18). We need per-instruction (statement-level) granularity so clicking a specific `gep` or `load` node highlights only the C line it came from.

## Approach

The LLVM IR has per-instruction `!dbg !N` annotations pointing to `!DILocation(line: L, ...)`. The `extractBlockSourceLines()` function already walks the pre-strip IR line by line. We extract per-instruction source lines keyed by `"funcName:blockLabel:instIdx"` where `instIdx` is the 0-based position among non-debug instructions (matching what tree-sitter sees in the stripped IR).

## Changes

### 1. `compiler-explorer.ts`
- Added `instSourceLines: Record<string, number>` to `CompileResult` interface
- Renamed `extractBlockSourceLines` → `extractSourceLines`, returning both `blockLines` and `instLines`
- Added `isDebugIntrinsic()` and `isInstruction()` helpers to correctly count instruction indices (skipping `#dbg_*`, `@llvm.dbg.*`, comments, blank lines)
- During IR walk, tracks `instIdx` per block, records `"func:block:idx" → srcLine` for each instruction with `!dbg` metadata

### 2. `App.tsx`
- Threads `instSourceLines` from `CompileResult` through to AirBundle stash (`_instSourceLines`)

### 3. `GraphPanel.tsx`
- Extracts `_instSourceLines` from the AirBundle and passes to `buildAirIndex()`

### 4. `air-index.ts`
- Added `instSourceLines?: Record<string, number>` parameter to `buildAirIndex()`
- In the instruction loop, looks up per-instruction source line first; falls back to block range if not found
- Per-instruction lines stored as `[line, line]` (single line) in `instLines` map

## Verification

Playwright verification with Pointer Aliasing example confirmed:
- `malloc` node → line 9 (was 9-18)
- `gep` for `p->x` → line 10 (was 9-18)
- `gep` for `p->y` → line 11 (was 9-18)
- `gep` for `q->x` → line 14 (was 9-18)
- `add` (val) → line 17 (was 9-18)
- `ret` → line 18 (was 9-18)
- Call Graph `main` → lines 9-18 (function-level, unchanged)
- CFG entry block → lines 9-18 (block-level, unchanged)
- Value Flow nodes: per-instruction lines (same as Def-Use)
