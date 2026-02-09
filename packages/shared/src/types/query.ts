/**
 * TypeScript types for the SAF JSON protocol (ProgramDatabase).
 *
 * These types mirror the Rust protocol types in
 * `crates/saf-analysis/src/database/protocol.rs`.
 */

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

export interface SchemaRequest {
  action: 'schema';
}

export interface CheckRequest {
  action: 'check';
  name: string;
  params?: Record<string, unknown>;
}

export interface CheckAllRequest {
  action: 'check_all';
}

export interface AnalyzeRequest {
  action: 'analyze';
  config: {
    name: string;
    severity: string;
    sources: unknown[];
  };
}

export interface QueryPrimitiveRequest {
  action: 'query';
  type: string;
  params: Record<string, unknown>;
}

export type QueryRequest =
  | SchemaRequest
  | CheckRequest
  | CheckAllRequest
  | AnalyzeRequest
  | QueryPrimitiveRequest;

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

/** A path event in a finding's trace. */
export interface PathEvent {
  location: string;
  event: string;
  state?: string;
}

/** A single finding from a check. */
export interface QueryFinding {
  check: string;
  severity: string;
  cwe?: number;
  message: string;
  path?: PathEvent[];
  object?: string;
}

/** Error details in a response. */
export interface QueryErrorDetail {
  code: string;
  message: string;
  suggestions: string[];
}

/** Response metadata. */
export interface QueryResponseMetadata {
  elapsed_ms?: number;
  engines_used?: string[];
}

/** A catalog entry describing a named check. */
export interface CatalogEntry {
  name: string;
  description: string;
  cwe: number | null;
  severity: string;
  category: string;
}

/** The full JSON protocol response. */
export interface QueryResponse {
  status: 'ok' | 'error';
  findings?: QueryFinding[];
  results?: Record<string, unknown>[];
  error?: QueryErrorDetail;
  metadata?: QueryResponseMetadata;
  /** Schema fields (present on schema responses). */
  checks?: CatalogEntry[];
  graphs?: string[];
  queries?: string[];
}
