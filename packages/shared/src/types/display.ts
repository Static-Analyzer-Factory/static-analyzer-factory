/**
 * Human-readable display label types.
 *
 * These types mirror the Rust types in
 * `crates/saf-analysis/src/display.rs`.
 */

/** The kind of AIR entity an ID refers to. */
export type EntityKind =
  | 'function'
  | 'block'
  | 'instruction'
  | 'value'
  | 'global'
  | 'global_obj'
  | 'source_file'
  | 'location'
  | 'svfg_node'
  | 'unknown';

/** Source location information resolved from a span. */
export interface SourceLoc {
  /** File path. */
  file: string;
  /** Start line (1-based). */
  line: number;
  /** Start column (1-based). */
  col: number;
  /** End line (1-based), if different from start. */
  end_line?: number;
  /** End column (1-based), if different from start. */
  end_col?: number;
}

/**
 * A human-readable label for an AIR entity.
 *
 * Contains both a short display name and optional context like source
 * location, full qualified name, and the containing function.
 */
export interface HumanLabel {
  /** The kind of entity this label describes. */
  kind: EntityKind;
  /** Short display name (e.g., "main", "call @printf", "%p"). */
  short_name: string;
  /** Longer descriptive name with context. */
  long_name?: string;
  /** Source location, if available. */
  source_loc?: SourceLoc;
  /** Name of the containing function, if applicable. */
  containing_function?: string;
}
