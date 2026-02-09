/**
 * AIR (Analysis Intermediate Representation) type definitions.
 *
 * These types match the JSON schema defined in saf-frontends/src/air_json_schema.rs.
 * IDs are hex strings like "0x00000000000000000000000000000001".
 */

import type { PropertyGraph } from './property-graph';

// ---------------------------------------------------------------------------
// AIR JSON types (matching JsonAirBundle / JsonModule / etc.)
// ---------------------------------------------------------------------------

export interface AirBundle {
  frontend_id: string;
  schema_version: string;
  module: AirModule;
}

export interface AirModule {
  id?: string;
  name?: string | null;
  functions: AirFunction[];
  globals: AirGlobal[];
  source_files: AirSourceFile[];
  type_hierarchy?: AirTypeHierarchyEntry[];
  constants?: Record<string, AirConstant>;
}

export interface AirFunction {
  id?: string;
  name: string;
  params: AirParam[];
  blocks: AirBlock[];
  entry_block?: string | null;
  is_declaration: boolean;
  span?: AirSpan | null;
  symbol?: AirSymbol | null;
}

export interface AirParam {
  id?: string;
  name?: string | null;
  index?: number;
}

export interface AirBlock {
  id?: string;
  label?: string | null;
  instructions: AirInstruction[];
}

/**
 * Flattened instruction format matching JsonInstruction.
 * The "op" field selects the operation type; operation-specific fields
 * sit at the same level.
 */
export interface AirInstruction {
  id?: string;
  op: string;
  operands: string[];
  dst?: string | null;
  span?: AirSpan | null;
  symbol?: AirSymbol | null;

  // Operation-specific fields (all optional)
  target?: string;              // Br
  then_target?: string;         // CondBr
  else_target?: string;         // CondBr
  default?: string;             // Switch
  cases?: [number, string][];   // Switch: [value, blockId]
  incoming?: [string, string][]; // Phi: [blockId, valueId]
  callee?: string;              // CallDirect
  obj?: string;                 // Global
  kind?: string;                 // BinaryOp / Cast / HeapAlloc sub-kind
  target_bits?: number;         // Cast (optional target bit-width)
  field_path?: AirFieldPath;    // Gep
  size_bytes?: number;          // Alloca
}

export interface AirFieldPath {
  steps: AirFieldStep[];
}

export type AirFieldStep =
  | { kind: 'index' }
  | { kind: 'field'; index: number };

export interface AirGlobal {
  id?: string;
  obj?: string;
  name: string;
  init?: AirConstant | null;
  is_constant: boolean;
  span?: AirSpan | null;
}

export type AirConstant =
  | { kind: 'int'; value: number; bits: number }
  | { kind: 'big_int'; value: string; bits: number }
  | { kind: 'float'; value: number; bits: number }
  | { kind: 'string'; value: string }
  | { kind: 'null' }
  | { kind: 'undef' }
  | { kind: 'zero_init' }
  | { kind: 'aggregate'; elements: AirConstant[] }
  | { kind: 'global_ref'; '0': string };

export interface AirVirtualMethodSlot {
  index: number;
  function?: string | null;
}

export interface AirTypeHierarchyEntry {
  type_name: string;
  base_types?: string[];
  virtual_methods?: AirVirtualMethodSlot[];
}

export interface AirSpan {
  file_id: string;
  byte_start: number;
  byte_end: number;
  line_start: number;
  col_start: number;
  line_end: number;
  col_end: number;
}

export interface AirSymbol {
  display_name: string;
  mangled_name?: string | null;
  namespace_path: string[];
}

export interface AirSourceFile {
  id: number;
  path: string;
  checksum?: string | null;
}

// ---------------------------------------------------------------------------
// Analysis result types (consumed by the UI)
// ---------------------------------------------------------------------------

export interface AnalysisResults {
  cfg: PropertyGraph;
  callgraph: PropertyGraph;
  defuse: PropertyGraph;
  valueflow: PropertyGraph;
  pta: PTAResult;
  functions: string[];
}

export interface PTAResult {
  points_to: PTAEntry[];
  /** Human-readable labels for PTA value IDs (hex to label). */
  value_labels?: Record<string, string>;
  /** Human-readable labels for PTA location IDs (hex to label). */
  location_labels?: Record<string, string>;
}

export interface PTAEntry {
  value: string;
  locations: string[];
}
