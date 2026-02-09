/**
 * SAF WASM bridge.
 *
 * Loads the real saf-wasm module at runtime.
 * If the WASM module is not available, analysis will fail with a
 * message directing the user to build it with `make wasm`.
 */

import type { AnalysisResults, HumanLabel, QueryResponse } from '../types';

let initialized = false;
let usingRealWasm = false;
let analyzeFunc: ((airJson: string, configJson: string) => string) | null = null;
let queryFunc: ((requestJson: string) => string) | null = null;
let resolveDisplayFunc: ((id: string) => string) | null = null;
let resolveDisplayBatchFunc: ((idsJson: string) => string) | null = null;

export async function initWasm(): Promise<void> {
  if (initialized) return;

  try {
    // Dynamic import of the WASM module built by `wasm-pack build`.
    // Path held in a variable so Rollup skips static resolution.
    // The WASM files are built by `wasm-pack` and gitignored; they only
    // exist after running `make wasm` or in CI.
    const wasmPath = '../wasm/saf_wasm.js';
    const wasm = await import(/* @vite-ignore */ wasmPath);
    await wasm.default();
    analyzeFunc = wasm.analyze;
    queryFunc = wasm.query;
    resolveDisplayFunc = wasm.resolve_display;
    resolveDisplayBatchFunc = wasm.resolve_display_batch;
    usingRealWasm = true;
  } catch {
    console.warn('[saf-wasm] WASM module not available. Run `make wasm` to build it.');
  }
  initialized = true;
}

/** Whether real WASM analysis is loaded. */
export function isWasmLoaded(): boolean {
  return usingRealWasm;
}

/** Whether a ProgramDatabase is ready for queries (analyze() was called). */
export function isDatabaseReady(): boolean {
  return usingRealWasm && queryFunc !== null;
}

/** Config object matching the Rust WasmConfig struct. */
export interface WasmAnalysisConfig {
  vf_mode?: string;
  pta_solver?: string;
  pta_max_iterations?: number;
  max_refinement_iters?: number;
  spec_yamls?: string[];
}

export function runAnalysis(
  airJson: string,
  config?: WasmAnalysisConfig,
): AnalysisResults {
  if (!analyzeFunc) {
    throw new Error(
      'SAF WASM module not available. Run `make wasm` to build it, then reload the page.',
    );
  }
  const configJson = config ? JSON.stringify(config) : '{}';
  const resultStr = analyzeFunc(airJson, configJson);
  const parsed = JSON.parse(resultStr);
  if (parsed.error) {
    throw new Error(`SAF analysis error: ${parsed.error}`);
  }
  return parsed as AnalysisResults;
}

/** Send a JSON protocol request to the ProgramDatabase. */
export function runQuery(requestJson: string): QueryResponse {
  if (!queryFunc) {
    throw new Error(
      'SAF WASM module not available. Run `make wasm` to build it, then reload the page.',
    );
  }
  const resultStr = queryFunc(requestJson);
  return JSON.parse(resultStr) as QueryResponse;
}

/**
 * Resolve a single hex ID to a human-readable label.
 *
 * Requires that `analyze()` has been called first to build the
 * `ProgramDatabase`. Returns the resolved `HumanLabel` or throws
 * if the WASM module is not loaded or the database is not ready.
 *
 * @param id - Hex ID string (e.g., "0x1a2b..." or "1a2b...")
 * @returns The resolved human-readable label.
 */
export function resolveDisplay(id: string): HumanLabel {
  if (!resolveDisplayFunc) {
    throw new Error(
      'SAF WASM module not available. Run `make wasm` to build it, then reload the page.',
    );
  }
  const resultStr = resolveDisplayFunc(id);
  const parsed = JSON.parse(resultStr);
  if (parsed.error) {
    throw new Error(`resolveDisplay error: ${parsed.error}`);
  }
  return parsed as HumanLabel;
}

/**
 * Resolve a batch of hex IDs to human-readable labels.
 *
 * More efficient than calling `resolveDisplay()` in a loop because
 * the `DisplayResolver` is constructed once for the entire batch.
 *
 * @param ids - Array of hex ID strings.
 * @returns Array of resolved labels in the same order as the input.
 */
export function resolveDisplayBatch(ids: string[]): HumanLabel[] {
  if (!resolveDisplayBatchFunc) {
    throw new Error(
      'SAF WASM module not available. Run `make wasm` to build it, then reload the page.',
    );
  }
  const resultStr = resolveDisplayBatchFunc(JSON.stringify(ids));
  const parsed = JSON.parse(resultStr);
  if (parsed.error) {
    throw new Error(`resolveDisplayBatch error: ${parsed.error}`);
  }
  return parsed as HumanLabel[];
}
