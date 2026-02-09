/**
 * Shared helpers for resolving display labels via the WASM DisplayResolver.
 *
 * Wraps `resolveDisplayBatch` with graceful fallback: if WASM is not loaded
 * or the ProgramDatabase is not ready, returns an empty map so callers can
 * fall back to property-based labels.
 */
import type { HumanLabel } from '../types';
import { isDatabaseReady, resolveDisplayBatch } from '../analysis/saf-wasm';

/**
 * Attempt to resolve a batch of IDs via the WASM DisplayResolver.
 *
 * Returns a Map from ID to `HumanLabel`. If the WASM module is not
 * available or the database is not ready, returns an empty map.
 */
export function tryResolveDisplayBatch(ids: string[]): Map<string, HumanLabel> {
  const result = new Map<string, HumanLabel>();
  if (ids.length === 0 || !isDatabaseReady()) return result;

  try {
    const labels = resolveDisplayBatch(ids);
    for (let i = 0; i < ids.length; i++) {
      if (labels[i]) {
        result.set(ids[i], labels[i]);
      }
    }
  } catch {
    // WASM not available or database not ready — fall back silently
  }

  return result;
}
