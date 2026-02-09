/**
 * SAF Python API autocompletion for the CodeMirror analyzer editor.
 *
 * Provides context-aware completions for the `saf` module injected by
 * pyodide-bridge.ts.  When the Python API changes, update SAF_API below
 * and completions update everywhere.
 */

import type { CompletionContext, CompletionResult, Completion } from '@codemirror/autocomplete';
import { pythonLanguage } from '@codemirror/lang-python';

// ---------------------------------------------------------------------------
// SAF API schema — single source of truth for completions
// ---------------------------------------------------------------------------

interface FieldDef {
  type: string;
  doc: string;
}

interface MethodDef {
  params: string;
  returns: string;
  doc: string;
}

interface ClassDef {
  doc: string;
  methods: Record<string, MethodDef>;
  fields: Record<string, FieldDef>;
}

interface FunctionDef {
  params: string;
  returns: string;
  doc: string;
}

interface ApiSchema {
  functions: Record<string, FunctionDef>;
  classes: Record<string, ClassDef>;
}

/**
 * Declarative schema of every public symbol in the `saf` Python module.
 *
 * Keep in sync with the SAF_PYTHON_MODULE string in pyodide-bridge.ts.
 */
const SAF_API: ApiSchema = {
  functions: {
    analyze: {
      params: '(*, pta_solver: str = None)',
      returns: 'AnalysisResult',
      doc: 'Load analysis results. For SDK-compatible workflow, use Project() instead.',
    },
    report: {
      params: '(node_id: str, severity: str, message: str)',
      returns: 'None',
      doc: "Report a finding. Severity: 'info', 'low', 'medium', 'high', 'critical'.",
    },
    source_line: {
      params: '(node_id: str)',
      returns: 'int | None',
      doc: 'Look up the C source line number for an instruction/value node ID.',
    },
    function_param: {
      params: '(function: str, index: int = None)',
      returns: 'Selector',
      doc: 'Create a selector for function parameter(s). Matches parameters of the named function.',
    },
    function_return: {
      params: '(function: str)',
      returns: 'Selector',
      doc: 'Create a selector for a function return value. Matches the return of the named function.',
    },
    call: {
      params: '(callee: str)',
      returns: 'Selector',
      doc: 'Create a selector for call results. Matches return values from calls to the named function.',
    },
    arg_to: {
      params: '(callee: str, index: int = None)',
      returns: 'Selector',
      doc: 'Create a selector for arguments to a function call. Matches arguments passed to the named function.',
    },
    global_var: {
      params: '(name: str)',
      returns: 'Selector',
      doc: 'Create a selector for a global variable by name.',
    },
  },
  classes: {
    AnalysisResult: {
      doc: 'Container holding all available analysis graphs.',
      methods: {},
      fields: {
        cfg: { type: 'PropertyGraph', doc: 'Control-flow graph.' },
        callgraph: { type: 'PropertyGraph', doc: 'Call graph.' },
        defuse: { type: 'PropertyGraph', doc: 'Def-use chains.' },
        valueflow: { type: 'PropertyGraph', doc: 'Value-flow graph.' },
        pta: { type: 'dict | None', doc: 'Points-to analysis results (raw JSON).' },
      },
    },
    PropertyGraph: {
      doc: "Wrapper around SAF's PropertyGraph JSON export.",
      methods: {
        find_nodes: {
          params: '(**kwargs)',
          returns: 'list[PgNode]',
          doc: 'Find nodes matching all given property filters (e.g. label="Function", name="main").',
        },
        successors: {
          params: '(node_id: str)',
          returns: 'list[PgNode]',
          doc: 'Return nodes reachable via one outgoing edge from node_id.',
        },
        predecessors: {
          params: '(node_id: str)',
          returns: 'list[PgNode]',
          doc: 'Return nodes with an edge leading to node_id.',
        },
      },
      fields: {
        nodes: { type: 'list[PgNode]', doc: 'All nodes in the graph.' },
        edges: { type: 'list[PgEdge]', doc: 'All edges in the graph.' },
        metadata: { type: 'dict', doc: 'Graph metadata.' },
        graph_type: { type: 'str', doc: 'Graph type identifier (e.g. "cfg", "callgraph").' },
      },
    },
    PgNode: {
      doc: 'A node in a PropertyGraph.',
      methods: {},
      fields: {
        id: { type: 'str', doc: 'Unique node identifier (hex string).' },
        labels: { type: 'list[str]', doc: 'Node labels (e.g. ["Function"], ["Block", "Entry"]).' },
        properties: { type: 'dict', doc: 'Node properties (name, kind, function, etc.).' },
      },
    },
    PgEdge: {
      doc: 'An edge in a PropertyGraph.',
      methods: {},
      fields: {
        src: { type: 'str', doc: 'Source node ID.' },
        dst: { type: 'str', doc: 'Destination node ID.' },
        edge_type: { type: 'str', doc: 'Edge type (e.g. "CALLS", "FLOWS_TO", "DEFINES").' },
        properties: { type: 'dict', doc: 'Edge properties.' },
      },
    },
    Project: {
      doc: 'SDK-compatible entry point for SAF analysis. Use Project() to start.',
      methods: {
        query: {
          params: '()',
          returns: 'Query',
          doc: 'Create a Query object for running taint_flow, flows, points_to, and may_alias queries.',
        },
        graphs: {
          params: '()',
          returns: 'GraphStore',
          doc: 'Get a GraphStore for accessing available analysis graphs.',
        },
        pta_result: {
          params: '()',
          returns: 'PtaResult',
          doc: 'Get the pointer analysis result.',
        },
        call_graph: {
          params: '()',
          returns: 'PropertyGraph',
          doc: 'Get the call graph as a PropertyGraph.',
        },
        value_flow: {
          params: '()',
          returns: 'PropertyGraph',
          doc: 'Get the value flow graph as a PropertyGraph.',
        },
      },
      fields: {},
    },
    Query: {
      doc: 'Query methods for taint analysis, flow analysis, and pointer queries.',
      methods: {
        taint_flow: {
          params: '(sources, sinks, sanitizers=None)',
          returns: 'list[Finding]',
          doc: 'Find taint flows from sources to sinks, excluding paths through sanitizers.',
        },
        flows: {
          params: '(sources, sinks)',
          returns: 'list[Finding]',
          doc: 'Find all data flows from sources to sinks.',
        },
        points_to: {
          params: '(pointer_id: str)',
          returns: 'list[str]',
          doc: 'Query what locations a pointer may point to.',
        },
        may_alias: {
          params: '(p: str, q: str)',
          returns: 'bool',
          doc: 'Query whether two pointers may alias.',
        },
      },
      fields: {},
    },
    Selector: {
      doc: 'A query selector specifying sources, sinks, or sanitizers. Create with factory functions.',
      methods: {
        to_dict: {
          params: '()',
          returns: 'dict',
          doc: 'Convert selector to dictionary for JSON protocol.',
        },
      },
      fields: {
        kind: { type: 'str', doc: 'Selector kind (e.g. "call_to", "arg_to", "function_param").' },
      },
    },
    Finding: {
      doc: 'A finding from a query result with source/sink locations and trace.',
      methods: {},
      fields: {
        check: { type: 'str', doc: 'Check name that produced this finding.' },
        severity: { type: 'str', doc: 'Severity level.' },
        message: { type: 'str', doc: 'Human-readable message.' },
        trace: { type: 'Trace', doc: 'Trace from source to sink.' },
        source_location: { type: 'str', doc: 'Source location string.' },
        sink_location: { type: 'str', doc: 'Sink location string.' },
        cwe: { type: 'int | None', doc: 'CWE ID if applicable.' },
        object: { type: 'str | None', doc: 'Object name if applicable.' },
      },
    },
    Trace: {
      doc: 'A trace showing the path from source to sink in a finding.',
      methods: {
        pretty: {
          params: '()',
          returns: 'str',
          doc: 'Format trace as a human-readable string with arrows.',
        },
      },
      fields: {
        steps: { type: 'list[TraceStep]', doc: 'Individual steps in the trace.' },
      },
    },
    TraceStep: {
      doc: 'A single step in a finding trace.',
      methods: {},
      fields: {
        location: { type: 'str', doc: 'Location description.' },
        event: { type: 'str', doc: 'Event description at this location.' },
        state: { type: 'str | None', doc: 'Optional state information.' },
      },
    },
    GraphStore: {
      doc: 'Unified access to analysis graphs.',
      methods: {
        available: {
          params: '()',
          returns: 'list[str]',
          doc: 'List available graph types (e.g. "cfg", "callgraph", "defuse", "valueflow").',
        },
        get: {
          params: '(name: str)',
          returns: 'PropertyGraph | None',
          doc: 'Get a graph by name.',
        },
        export: {
          params: '(name: str, function: str = None)',
          returns: 'PropertyGraph | None',
          doc: 'Export a graph, optionally filtered to a specific function.',
        },
      },
      fields: {},
    },
    PtaResult: {
      doc: 'Pointer analysis result for querying points-to sets and aliases.',
      methods: {
        points_to: {
          params: '(pointer_id: str)',
          returns: 'list[str]',
          doc: 'Get locations that a pointer may point to.',
        },
        all_entries: {
          params: '()',
          returns: 'list[dict]',
          doc: 'Get all points-to entries.',
        },
        may_alias: {
          params: '(p: str, q: str)',
          returns: 'bool',
          doc: 'Check if two pointers may alias.',
        },
      },
      fields: {},
    },
  },
};

// ---------------------------------------------------------------------------
// Build completion lists from schema
// ---------------------------------------------------------------------------

const SAF_SECTION = { name: 'SAF API', rank: 0 };

/** Top-level `saf.` completions (functions + class names). */
function safModuleCompletions(): Completion[] {
  const items: Completion[] = [];

  for (const [name, fn] of Object.entries(SAF_API.functions)) {
    items.push({
      label: name,
      type: 'function',
      detail: `${fn.params} -> ${fn.returns}`,
      info: fn.doc,
      boost: 10,
      section: SAF_SECTION,
    });
  }

  for (const [name, cls] of Object.entries(SAF_API.classes)) {
    items.push({
      label: name,
      type: 'class',
      detail: 'class',
      info: cls.doc,
      boost: 5,
      section: SAF_SECTION,
    });
  }

  return items;
}

/** Members of a class (methods + fields). */
function classMemberCompletions(cls: ClassDef): Completion[] {
  const items: Completion[] = [];

  for (const [name, method] of Object.entries(cls.methods)) {
    items.push({
      label: name,
      type: 'method',
      detail: `${method.params} -> ${method.returns}`,
      info: method.doc,
      boost: 10,
      section: SAF_SECTION,
    });
  }

  for (const [name, field] of Object.entries(cls.fields)) {
    items.push({
      label: name,
      type: 'property',
      detail: field.type,
      info: field.doc,
      boost: 8,
      section: SAF_SECTION,
    });
  }

  return items;
}

// Pre-compute completion lists
const MODULE_COMPLETIONS = safModuleCompletions();
const CLASS_COMPLETIONS: Record<string, Completion[]> = {};
for (const [name, cls] of Object.entries(SAF_API.classes)) {
  CLASS_COMPLETIONS[name] = classMemberCompletions(cls);
}

// ---------------------------------------------------------------------------
// Identifier → type resolution for dot-completion
// ---------------------------------------------------------------------------

/**
 * Known variable names / patterns and the SAF class they hold.
 *
 * This handles the common patterns users write in analyzer scripts:
 *   result = saf.analyze()  →  AnalysisResult
 *   result.cfg              →  PropertyGraph
 *   result.callgraph        →  PropertyGraph
 *   node (from find_nodes)  →  PgNode
 *   edge (from .edges)      →  PgEdge
 */
const IDENTIFIER_TYPES: Record<string, string> = {
  result: 'AnalysisResult',
  results: 'AnalysisResult',
  res: 'AnalysisResult',
  r: 'AnalysisResult',
  cfg: 'PropertyGraph',
  callgraph: 'PropertyGraph',
  cg: 'PropertyGraph',
  defuse: 'PropertyGraph',
  valueflow: 'PropertyGraph',
  vfg: 'PropertyGraph',
  graph: 'PropertyGraph',
  pg: 'PropertyGraph',
  node: 'PgNode',
  n: 'PgNode',
  src_node: 'PgNode',
  dst_node: 'PgNode',
  caller: 'PgNode',
  callee: 'PgNode',
  edge: 'PgEdge',
  e: 'PgEdge',
  proj: 'Project',
  project: 'Project',
  p: 'Project',
  q: 'Query',
  query: 'Query',
  gs: 'GraphStore',
  store: 'GraphStore',
  pta: 'PtaResult',
  f: 'Finding',
  finding: 'Finding',
  t: 'Trace',
  trace: 'Trace',
  step: 'TraceStep',
  sel: 'Selector',
  selector: 'Selector',
};

/** Map AnalysisResult field names to their types. */
const RESULT_FIELD_TYPES: Record<string, string> = {};
for (const [field, def] of Object.entries(SAF_API.classes.AnalysisResult.fields)) {
  if (def.type === 'PropertyGraph') {
    RESULT_FIELD_TYPES[field] = 'PropertyGraph';
  }
}

/** Map Project method names to their return types. */
const PROJECT_METHOD_TYPES: Record<string, string> = {
  query: 'Query',
  graphs: 'GraphStore',
  pta_result: 'PtaResult',
  call_graph: 'PropertyGraph',
  value_flow: 'PropertyGraph',
};

/**
 * Try to infer the SAF class for a dotted expression prefix.
 * Returns the class name or null.
 */
function inferType(prefix: string): string | null {
  // "saf" → module-level completions (handled separately)
  if (prefix === 'saf') return '__module__';

  // "result.cfg" → PropertyGraph, "proj.query" → Query
  const parts = prefix.split('.');
  if (parts.length === 2) {
    const [obj, field] = parts;
    const objType = IDENTIFIER_TYPES[obj];
    if (objType === 'AnalysisResult' && RESULT_FIELD_TYPES[field]) {
      return RESULT_FIELD_TYPES[field];
    }
    // Project method return types: proj.query() → Query
    if (objType === 'Project' && PROJECT_METHOD_TYPES[field]) {
      return PROJECT_METHOD_TYPES[field];
    }
  }

  // Single identifier → direct lookup
  if (parts.length === 1) {
    return IDENTIFIER_TYPES[prefix] ?? null;
  }

  return null;
}

// ---------------------------------------------------------------------------
// CodeMirror CompletionSource
// ---------------------------------------------------------------------------

/**
 * Regex to match the text before the cursor for dot-triggered completion.
 * Captures: `identifier.identifier.` (1+ parts ending with a dot).
 */
const DOT_PATTERN = /(\w+(?:\.\w+)*)\.\s*$/;

/**
 * Regex for word-triggered completion (typing a partial identifier).
 * Matches: `saf.ana` or `result.fi` etc.
 */
const WORD_PATTERN = /(\w+(?:\.\w+)*)\.(\w+)$/;

/**
 * Regex for top-level `import saf` / `from saf import ...` completion.
 */
const IMPORT_PATTERN = /(?:^|\n)\s*(?:import|from)\s+(\w*)$/;

function safCompletionSource(context: CompletionContext): CompletionResult | null {
  const line = context.state.doc.lineAt(context.pos);
  const textBefore = line.text.slice(0, context.pos - line.from);

  // Case 1: import statement — suggest "saf"
  const importMatch = textBefore.match(IMPORT_PATTERN);
  if (importMatch) {
    return {
      from: context.pos - importMatch[1].length,
      options: [
        {
          label: 'saf',
          type: 'namespace',
          detail: 'SAF analysis module',
          info: 'Static analysis framework — provides analyze(), report(), PropertyGraph, etc.',
          boost: 99,
          section: SAF_SECTION,
        },
      ],
      validFor: /^\w*$/,
    };
  }

  // Case 2: `from saf import <partial>` — suggest top-level names
  const fromImportMatch = textBefore.match(/from\s+saf\s+import\s+(\w*)$/);
  if (fromImportMatch) {
    return {
      from: context.pos - fromImportMatch[1].length,
      options: MODULE_COMPLETIONS,
      validFor: /^\w*$/,
    };
  }

  // Case 3: Dot-triggered — `expr.` with cursor right after the dot
  const dotMatch = textBefore.match(DOT_PATTERN);
  if (dotMatch) {
    const prefix = dotMatch[1];
    const type = inferType(prefix);

    if (type === '__module__') {
      return { from: context.pos, options: MODULE_COMPLETIONS, validFor: /^\w*$/ };
    }
    if (type && CLASS_COMPLETIONS[type]) {
      return { from: context.pos, options: CLASS_COMPLETIONS[type], validFor: /^\w*$/ };
    }
  }

  // Case 4: Typing after a dot — `expr.partial`
  const wordMatch = textBefore.match(WORD_PATTERN);
  if (wordMatch) {
    const prefix = wordMatch[1];
    const partial = wordMatch[2];
    const type = inferType(prefix);

    if (type === '__module__') {
      return {
        from: context.pos - partial.length,
        options: MODULE_COMPLETIONS,
        validFor: /^\w*$/,
      };
    }
    if (type && CLASS_COMPLETIONS[type]) {
      return {
        from: context.pos - partial.length,
        options: CLASS_COMPLETIONS[type],
        validFor: /^\w*$/,
      };
    }
  }

  // Case 5: Explicit activation (Ctrl+Space) with no dot context —
  // offer all SAF top-level symbols if the partial matches
  if (context.explicit) {
    const word = context.matchBefore(/\w+/);
    if (word) {
      return {
        from: word.from,
        options: MODULE_COMPLETIONS,
        validFor: /^\w*$/,
      };
    }
  }

  return null;
}

// ---------------------------------------------------------------------------
// Public extension
// ---------------------------------------------------------------------------

/**
 * CodeMirror extension that adds SAF Python API autocompletion.
 *
 * Registers as a language-data completion source for the Python language,
 * so it merges naturally with basicSetup's built-in autocompletion.
 *
 * Usage in AnalyzerPanel:
 *   import { safAutocomplete } from '../editor/saf-completions';
 *   // Add to extensions: [..., safAutocomplete]
 */
export const safAutocomplete = pythonLanguage.data.of({
  autocomplete: safCompletionSource,
});
