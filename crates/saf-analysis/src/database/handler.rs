//! JSON request handler for `ProgramDatabase`.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::ValueId;

use crate::checkers;
use crate::selector::{Selector, resolve_selectors};
use crate::svfg::SvfgNodeId;
use crate::valueflow::{Flow, NodeId, QueryLimits};

use super::ProgramDatabase;
use super::catalog::CheckCatalog;
use super::protocol::{ErrorDetail, Finding, PathEvent, Request, Response, ResponseMetadata};

use crate::timer::Timer;

impl ProgramDatabase {
    /// Handle a JSON request string and return a JSON response string.
    ///
    /// This is the main entry point for the JSON protocol.
    pub fn handle_request(&self, json: &str) -> Result<String, serde_json::Error> {
        let request: Request = match serde_json::from_str(json) {
            Ok(req) => req,
            Err(e) => {
                let resp =
                    Response::error("INVALID_REQUEST", &format!("Failed to parse request: {e}"));
                return serde_json::to_string(&resp);
            }
        };

        let response = match request {
            Request::Schema => self.handle_schema(),
            Request::Check { name, params } => self.handle_check(&name, params.as_ref()),
            Request::CheckAll => self.handle_check_all(),
            Request::Analyze { config } => {
                // Treat analyze as a named check lookup for now
                self.handle_check(&config.name, None)
            }
            Request::Cypher { .. } => Response::error(
                "NOT_IMPLEMENTED",
                "Cypher queries require Neo4j connection (use Python Neo4jExporter)",
            ),
            Request::Query { query_type, params } => {
                self.handle_query_dispatch(&query_type, &params)
            }
        };

        serde_json::to_string(&response)
    }

    fn handle_schema(&self) -> Response {
        let schema = self.schema();
        let mut resp = Response {
            status: "ok".to_string(),
            findings: None,
            results: None,
            error: None,
            metadata: None,
            engine: None,
            extra: BTreeMap::new(),
        };
        resp.extra.insert(
            "checks".to_string(),
            serde_json::to_value(&schema.checks).unwrap_or_default(),
        );
        resp.extra.insert(
            "graphs".to_string(),
            serde_json::to_value(&schema.graphs).unwrap_or_default(),
        );
        resp.extra.insert(
            "queries".to_string(),
            serde_json::to_value(&schema.queries).unwrap_or_default(),
        );
        resp
    }

    fn handle_check(
        &self,
        name: &str,
        _params: Option<&BTreeMap<String, serde_json::Value>>,
    ) -> Response {
        let timer = Timer::now();

        // Check for numeric checks first
        let is_numeric = matches!(
            name,
            "buffer_overflow" | "integer_overflow" | "division_by_zero"
        );
        if is_numeric {
            let findings = Self::run_numeric_check(name);
            return Response::ok_findings(
                findings,
                ResponseMetadata {
                    elapsed_ms: Some(timer.elapsed_ms()),
                    engines_used: vec!["absint".to_string()],
                },
            );
        }

        // Look up the check in the catalog
        let catalog = CheckCatalog::new();
        let Some(checker_name) = CheckCatalog::to_checker_name(name) else {
            let suggestions: Vec<String> = catalog
                .list()
                .into_iter()
                .filter(|catalog_name| {
                    catalog_name.contains(&name.replace('_', ""))
                        || name.contains(&catalog_name.replace('_', ""))
                })
                .map(String::from)
                .collect();
            return Response {
                status: "error".to_string(),
                findings: None,
                results: None,
                error: Some(ErrorDetail {
                    code: "UNKNOWN_CHECK".to_string(),
                    message: format!("No check named '{name}'"),
                    suggestions,
                }),
                metadata: None,
                engine: None,
                extra: BTreeMap::new(),
            };
        };

        // SVFG-based checks
        let findings = self.run_svfg_check(checker_name);
        Response::ok_findings(
            findings,
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["pta".to_string(), "svfg".to_string()],
            },
        )
    }

    fn handle_check_all(&self) -> Response {
        let timer = Timer::now();
        let catalog = CheckCatalog::new();
        let mut all_findings = Vec::new();

        for name in catalog.list() {
            if let Some(checker_name) = CheckCatalog::to_checker_name(name) {
                all_findings.extend(self.run_svfg_check(checker_name));
            }
        }

        // Suppress cross-checker noise (null-deref overlap, identical specs).
        let deduped = suppress_cross_checker_findings(all_findings);

        Response::ok_findings(
            deduped,
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["pta".to_string(), "svfg".to_string()],
            },
        )
    }

    /// Dispatch a query request based on the `language` parameter.
    ///
    /// If `params` contains a `"language"` key, routes to the appropriate
    /// engine. Defaults to `"builtin"` when no language is specified.
    /// Builtin queries delegate to [`Self::handle_builtin_query`]; `python`
    /// and `datalog` languages return "not yet implemented" errors.
    fn handle_query_dispatch(
        &self,
        query_type: &str,
        params: &BTreeMap<String, serde_json::Value>,
    ) -> Response {
        let language = params
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("builtin");

        match language {
            "builtin" => {
                let mut resp = self.handle_builtin_query(query_type, params);
                resp.engine = Some("builtin".to_string());
                resp
            }
            "python" | "datalog" => {
                let mut resp = Response::error(
                    "NOT_IMPLEMENTED",
                    &format!("{language} query language not yet implemented"),
                );
                resp.engine = Some(language.to_string());
                resp
            }
            other => Response::error(
                "UNKNOWN_LANGUAGE",
                &format!("unknown query language: {other}"),
            ),
        }
    }

    /// Handle a builtin query by type (alias, `points_to`, `taint_flow`, flows, etc.).
    fn handle_builtin_query(
        &self,
        query_type: &str,
        params: &BTreeMap<String, serde_json::Value>,
    ) -> Response {
        let timer = Timer::now();
        match query_type {
            "points_to" => self.handle_points_to_query(params, &timer),
            "alias" => self.handle_alias_query(params, &timer),
            "taint_flow" => self.handle_taint_flow_query(params, &timer),
            "flows" => self.handle_flows_query(params, &timer),
            "reachable" => Response::error(
                "NOT_IMPLEMENTED",
                "reachable query not yet implemented via JSON protocol",
            ),
            _ => Response::error(
                "UNKNOWN_QUERY_TYPE",
                &format!("Unknown query type: {query_type}"),
            ),
        }
    }

    /// Handle a `points_to` query.
    ///
    /// Expects `params["pointer"]` as a hex `ValueId` string. Returns the
    /// set of locations that the pointer may point to.
    fn handle_points_to_query(
        &self,
        params: &BTreeMap<String, serde_json::Value>,
        timer: &Timer,
    ) -> Response {
        let Some(pointer_val) = params.get("pointer") else {
            return Response::error(
                "MISSING_PARAM",
                "points_to query requires a 'pointer' parameter (hex ValueId)",
            );
        };

        let vid: ValueId = match serde_json::from_value(pointer_val.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Response::error("INVALID_PARAM", &format!("invalid 'pointer' hex ID: {e}"));
            }
        };

        let locations = self.points_to(vid);
        let resolver = self.display_resolver();
        let pointer_label = resolver.resolve(vid.raw());
        let results: Vec<BTreeMap<String, serde_json::Value>> = locations
            .iter()
            .map(|loc| {
                let label = resolver.resolve(loc.raw());
                let mut row = BTreeMap::new();
                row.insert(
                    "location".to_string(),
                    serde_json::Value::String(loc.to_hex()),
                );
                row.insert(
                    "display_name".to_string(),
                    serde_json::Value::String(label.short_name.clone()),
                );
                if let Some(ref sl) = label.source_loc {
                    row.insert(
                        "source_loc".to_string(),
                        serde_json::to_value(sl).unwrap_or_default(),
                    );
                }
                row
            })
            .collect();
        // Add pointer display name to metadata
        let _ = pointer_label;

        Response::ok_results(
            results,
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["pta".to_string()],
            },
        )
    }

    /// Handle an `alias` query.
    ///
    /// Expects `params["p"]` and `params["q"]` as hex `ValueId` strings.
    /// Returns whether the two pointers may alias.
    fn handle_alias_query(
        &self,
        params: &BTreeMap<String, serde_json::Value>,
        timer: &Timer,
    ) -> Response {
        let Some(p_val) = params.get("p") else {
            return Response::error(
                "MISSING_PARAM",
                "alias query requires a 'p' parameter (hex ValueId)",
            );
        };
        let Some(q_val) = params.get("q") else {
            return Response::error(
                "MISSING_PARAM",
                "alias query requires a 'q' parameter (hex ValueId)",
            );
        };

        let p: ValueId = match serde_json::from_value(p_val.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Response::error("INVALID_PARAM", &format!("invalid 'p' hex ID: {e}"));
            }
        };
        let q: ValueId = match serde_json::from_value(q_val.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Response::error("INVALID_PARAM", &format!("invalid 'q' hex ID: {e}"));
            }
        };

        let alias_result = self.may_alias(p, q);
        let resolver = self.display_resolver();
        let p_label = resolver.resolve(p.raw()).short_name;
        let q_label = resolver.resolve(q.raw()).short_name;
        let mut row = BTreeMap::new();
        row.insert(
            "may_alias".to_string(),
            serde_json::Value::Bool(alias_result.may_alias_conservative()),
        );
        row.insert(
            "result".to_string(),
            serde_json::Value::String(format!("{alias_result:?}")),
        );
        row.insert("p_name".to_string(), serde_json::Value::String(p_label));
        row.insert("q_name".to_string(), serde_json::Value::String(q_label));

        Response::ok_results(
            vec![row],
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["pta".to_string()],
            },
        )
    }

    /// Handle a `taint_flow` query.
    ///
    /// Expects `params["sources"]`, `params["sinks"]` as `Selector` JSON
    /// arrays, and optionally `params["sanitizers"]`. Returns flows as findings.
    fn handle_taint_flow_query(
        &self,
        params: &BTreeMap<String, serde_json::Value>,
        timer: &Timer,
    ) -> Response {
        let (sources, sinks) = match self.parse_sources_sinks(params) {
            Ok(v) => v,
            Err(resp) => return resp,
        };

        let sanitizers = match params.get("sanitizers") {
            Some(val) => match self.parse_selectors_param(val, "sanitizers") {
                Ok(s) => s,
                Err(resp) => return resp,
            },
            None => BTreeSet::new(),
        };

        let limits = QueryLimits::default();
        let flows = self
            .valueflow()
            .taint_flow(&sources, &sinks, &sanitizers, &limits);

        let resolver = self.display_resolver();
        let findings: Vec<Finding> = flows
            .iter()
            .map(|flow| flow_to_finding(flow, "taint_flow", &resolver))
            .collect();

        Response::ok_findings(
            findings,
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["valueflow".to_string()],
            },
        )
    }

    /// Handle a `flows` query.
    ///
    /// Expects `params["sources"]` and `params["sinks"]` as `Selector` JSON
    /// arrays. Returns all reachable flows as findings.
    fn handle_flows_query(
        &self,
        params: &BTreeMap<String, serde_json::Value>,
        timer: &Timer,
    ) -> Response {
        let (sources, sinks) = match self.parse_sources_sinks(params) {
            Ok(v) => v,
            Err(resp) => return resp,
        };

        let limits = QueryLimits::default();
        let flows = self.valueflow().flows(&sources, &sinks, &limits);

        let resolver = self.display_resolver();
        let findings: Vec<Finding> = flows
            .iter()
            .map(|flow| flow_to_finding(flow, "flows", &resolver))
            .collect();

        Response::ok_findings(
            findings,
            ResponseMetadata {
                elapsed_ms: Some(timer.elapsed_ms()),
                engines_used: vec!["valueflow".to_string()],
            },
        )
    }

    /// Parse `"sources"` and `"sinks"` selector arrays from query params.
    ///
    /// Returns the resolved `ValueId` sets, or an error `Response`.
    // `Response` is the JSON-RPC wire protocol response — large by design
    #[allow(clippy::result_large_err)]
    fn parse_sources_sinks(
        &self,
        params: &BTreeMap<String, serde_json::Value>,
    ) -> Result<(BTreeSet<ValueId>, BTreeSet<ValueId>), Response> {
        let Some(sources_val) = params.get("sources") else {
            return Err(Response::error(
                "MISSING_PARAM",
                "query requires a 'sources' parameter (array of selectors)",
            ));
        };
        let Some(sinks_val) = params.get("sinks") else {
            return Err(Response::error(
                "MISSING_PARAM",
                "query requires a 'sinks' parameter (array of selectors)",
            ));
        };

        let sources = self.parse_selectors_param(sources_val, "sources")?;
        let sinks = self.parse_selectors_param(sinks_val, "sinks")?;
        Ok((sources, sinks))
    }

    /// Parse a single selector array param and resolve to `ValueId`s.
    // `Response` is the JSON-RPC wire protocol response — large by design
    #[allow(clippy::result_large_err)]
    fn parse_selectors_param(
        &self,
        value: &serde_json::Value,
        param_name: &str,
    ) -> Result<BTreeSet<ValueId>, Response> {
        let selectors: Vec<Selector> = serde_json::from_value(value.clone()).map_err(|e| {
            Response::error(
                "INVALID_PARAM",
                &format!("invalid '{param_name}' selector array: {e}"),
            )
        })?;

        resolve_selectors(&selectors, &self.module).map_err(|e| {
            Response::error(
                "RESOLVE_ERROR",
                &format!("failed to resolve '{param_name}' selectors: {e}"),
            )
        })
    }

    /// Run an SVFG-based checker and convert findings to protocol format.
    fn run_svfg_check(&self, checker_name: &str) -> Vec<Finding> {
        let Some(spec) = checkers::builtin_checker(checker_name) else {
            return Vec::new();
        };

        let default_table;
        let table = if let Some(ref t) = self.resource_table {
            t
        } else {
            default_table = checkers::ResourceTable::new();
            &default_table
        };
        let svfg = self.get_or_build_svfg();
        let reachable = crate::callgraph::reachable_from_main(self.call_graph(), &self.module);
        let config = checkers::SolverConfig {
            reachable_functions: reachable,
            ..checkers::SolverConfig::default()
        };
        let result = checkers::run_checker(&spec, &self.module, svfg, table, &config);

        let (source_event, sink_event) = checker_event_descriptions(checker_name);

        // Build a DisplayResolver once for enriching all findings from this check.
        let resolver = self.display_resolver();

        result
            .findings
            .iter()
            .flat_map(|f| {
                let source_loc = resolve_node_location(f.source_node, &self.module);

                // When source_node == sink_node (overlap), resolve_node_location
                // returns the same location for both. For checkers that use
                // dereference-based sinks (LoadDeref/StoreDeref/GepDeref),
                // expand to per-instruction findings so each dereference gets
                // its own source location. This handles the case where GEPs
                // are absent and multiple dereference instructions share the
                // same pointer SVFG node.
                //
                // Only expand when:
                // 1. The checker uses dereference-based sinks
                // 2. The BFS solver didn't already find non-overlap sinks for
                //    this source (which would produce duplicate locations)
                let is_deref_checker =
                    matches!(checker_name, "use-after-free" | "uninit-use" | "null-deref");
                let has_non_overlap_sinks = result.findings.iter().any(|other| {
                    other.source_node == f.source_node && other.sink_node != other.source_node
                });
                if f.source_node == f.sink_node && is_deref_checker && !has_non_overlap_sinks {
                    let expanded = expand_overlap_deref_findings(
                        f,
                        &self.module,
                        source_loc.as_ref(),
                        source_event,
                        sink_event,
                    );
                    if !expanded.is_empty() {
                        return expanded;
                    }
                }

                // Normal case (or overlap with no dereference instructions found)
                let sink_loc = resolve_node_location(f.sink_node, &self.module);

                let mut path = Vec::new();
                if let Some((loc, _)) = &source_loc {
                    let enrichment = resolve_svfg_enrichment(f.source_node, &resolver);
                    path.push(PathEvent {
                        location: loc.clone(),
                        event: source_event.to_string(),
                        state: None,
                        display_name: enrichment.0,
                        source_loc: enrichment.1,
                    });
                }
                if let Some((loc, _)) = &sink_loc {
                    let enrichment = resolve_svfg_enrichment(f.sink_node, &resolver);
                    path.push(PathEvent {
                        location: loc.clone(),
                        event: sink_event.to_string(),
                        state: None,
                        display_name: enrichment.0,
                        source_loc: enrichment.1,
                    });
                }

                // Use source node ValueId hex for precise per-allocation matching
                // in bench mode (alloc_site identity). Fall back to debug symbol.
                let object = if let Some(vid) = f.source_node.as_value() {
                    Some(vid.to_hex())
                } else {
                    source_loc.and_then(|(_, sym)| sym)
                };

                // Derive a display name for the finding from the source node.
                let display_name = f
                    .source_node
                    .as_value()
                    .map(|vid| resolver.resolve(vid.raw()))
                    .map(|label| label.short_name);

                vec![Finding {
                    check: f.checker_name.clone(),
                    severity: format!("{:?}", f.severity).to_lowercase(),
                    cwe: f.cwe,
                    message: f.message.clone(),
                    path,
                    object,
                    display_name,
                }]
            })
            .collect()
    }

    /// Run a numeric (absint) check — placeholder.
    fn run_numeric_check(_name: &str) -> Vec<Finding> {
        // Numeric checks use the absint framework; not wired up yet.
        Vec::new()
    }
}

/// Convert a value-flow `Flow` to a protocol `Finding`, enriched with
/// human-readable names and source locations via the `DisplayResolver`.
fn flow_to_finding(
    flow: &Flow,
    check_name: &str,
    resolver: &crate::display::DisplayResolver<'_>,
) -> Finding {
    let path: Vec<PathEvent> = flow
        .trace
        .steps
        .iter()
        .map(|step| {
            let (dn, sl) = resolve_node_enrichment(step.from, resolver);
            PathEvent {
                location: format_node_id_resolved(step.from, resolver),
                event: step.edge.name().to_string(),
                state: None,
                display_name: dn,
                source_loc: sl,
            }
        })
        .collect();

    let source_label = resolve_node_label(flow.source, resolver);
    let sink_label = resolve_node_label(flow.sink, resolver);

    Finding {
        check: check_name.to_string(),
        severity: "info".to_string(),
        cwe: None,
        message: format!("Flow from {source_label} to {sink_label}"),
        path,
        object: Some(source_label.clone()),
        display_name: Some(source_label),
    }
}

/// Format a `NodeId` as a human-readable string using the resolver.
fn format_node_id_resolved(node: NodeId, resolver: &crate::display::DisplayResolver<'_>) -> String {
    match node {
        NodeId::Value { id } => {
            let label = resolver.resolve(id.raw());
            if let Some(loc) = &label.source_loc {
                if let Some(ref ctx) = label.containing_function {
                    format!("{}() at {}:{}", ctx, loc.file, loc.line)
                } else {
                    format!("{} at {}:{}", label.short_name, loc.file, loc.line)
                }
            } else {
                label.short_name
            }
        }
        NodeId::Location { id } => {
            let label = resolver.resolve(id.raw());
            label.short_name
        }
        NodeId::UnknownMem => "unknown_mem".to_string(),
    }
}

/// Resolve a `NodeId` to `(display_name, source_loc)` for enrichment.
fn resolve_node_enrichment(
    node: NodeId,
    resolver: &crate::display::DisplayResolver<'_>,
) -> (Option<String>, Option<crate::display::SourceLoc>) {
    match node {
        NodeId::Value { id } => {
            let label = resolver.resolve(id.raw());
            (Some(label.short_name), label.source_loc)
        }
        NodeId::Location { id } => {
            let label = resolver.resolve(id.raw());
            (Some(label.short_name), label.source_loc)
        }
        NodeId::UnknownMem => (Some("unknown_mem".to_string()), None),
    }
}

/// Get a short human-readable label for a `ValueId`.
fn resolve_node_label(vid: ValueId, resolver: &crate::display::DisplayResolver<'_>) -> String {
    let label = resolver.resolve(vid.raw());
    if let Some(loc) = &label.source_loc {
        format!("{} ({}:{})", label.short_name, loc.file, loc.line)
    } else {
        label.short_name
    }
}

/// Resolve an `SvfgNodeId` to enrichment fields via `DisplayResolver`.
///
/// Returns `(display_name, source_loc)` for populating `PathEvent` and
/// `Finding` enrichment fields. Falls back to `None` if the node has
/// no `ValueId` (e.g., `MemPhi` nodes).
fn resolve_svfg_enrichment(
    node: SvfgNodeId,
    resolver: &crate::display::DisplayResolver<'_>,
) -> (Option<String>, Option<crate::display::SourceLoc>) {
    let Some(vid) = node.as_value() else {
        return (None, None);
    };
    let label = resolver.resolve(vid.raw());
    (Some(label.short_name), label.source_loc)
}

/// Resolve an `SvfgNodeId` to a location string and optional symbol name.
///
/// Returns `(location_string, symbol_name)` where `location_string` is
/// formatted as `"func() at file:line"` or `"file:line"`.
/// `MemPhi` nodes have no source location and return `None`.
fn resolve_node_location(node: SvfgNodeId, module: &AirModule) -> Option<(String, Option<String>)> {
    let value_id = node.as_value()?;

    // Check function parameters
    for func in &module.functions {
        for param in &func.params {
            if param.id == value_id {
                let loc = format_location(&func.name, func.span.as_ref(), module);
                return Some((loc, param.name.clone()));
            }
        }

        // Check instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.dst == Some(value_id) {
                    let symbol = inst.symbol.as_ref().map(|s| s.display_name.clone());
                    let loc = format_location(&func.name, inst.span.as_ref(), module);
                    return Some((loc, symbol));
                }
            }
        }
    }

    // Check globals
    for global in &module.globals {
        if global.id == value_id {
            let loc = format_location("<global>", global.span.as_ref(), module);
            return Some((loc, Some(global.name.clone())));
        }
    }

    None
}

/// Format a location string from function name and optional span.
fn format_location(
    func_name: &str,
    span: Option<&saf_core::span::Span>,
    module: &AirModule,
) -> String {
    let Some(span) = span else {
        return format!("{func_name}()");
    };

    let file = module
        .source_files
        .iter()
        .find(|sf| sf.id == span.file_id)
        .map_or_else(|| "?".to_string(), |sf| sf.path.clone());

    format!("{func_name}() at {file}:{}", span.line_start)
}

/// Resolve a `Span` to a `SourceLoc` for enrichment fields.
fn resolve_span_to_source_loc(
    span: &saf_core::span::Span,
    module: &AirModule,
) -> Option<crate::display::SourceLoc> {
    let file = module
        .source_files
        .iter()
        .find(|sf| sf.id == span.file_id)?;

    Some(crate::display::SourceLoc {
        file: file.path.clone(),
        line: span.line_start,
        col: span.col_start,
        end_line: if span.line_end == span.line_start {
            None
        } else {
            Some(span.line_end)
        },
        end_col: if span.col_end == span.col_start {
            None
        } else {
            Some(span.col_end)
        },
    })
}

/// Get checker-aware event descriptions for source and sink.
fn checker_event_descriptions(checker_name: &str) -> (&'static str, &'static str) {
    match checker_name {
        "use-after-free" => ("memory allocated", "pointer used after free"),
        "double-free" => ("memory allocated", "freed again"),
        "memory-leak" => ("memory allocated", "not freed on this path"),
        "null-deref" => ("may be null", "pointer dereferenced"),
        "uninit-use" => ("uninitialized", "value used"),
        "stack-escape" => ("stack allocation", "pointer escapes function"),
        "file-descriptor-leak" => ("file opened", "not closed on this path"),
        "lock-not-released" => ("lock acquired", "not released on this path"),
        "generic-resource-leak" => ("resource acquired", "not released on this path"),
        _ => ("source", "sink"),
    }
}

/// Suppress cross-checker noise when running all checkers together.
///
/// When multiple checkers run on the same program, some produce findings that
/// are technically correct but redundant or less informative than findings from
/// more specific checkers. Two suppression rules:
///
/// 1. **`null-deref` suppression**: Suppressed when another checker already
///    flags the same source location (e.g., `malloc` return) or the same sink
///    location (e.g., a dereference). If a more specific checker (UAF,
///    memory-leak, etc.) identified a bug involving the same allocation, the
///    `null-deref` finding adds noise rather than insight.
///
/// 2. **`generic-resource-leak` suppression**: Suppressed when `memory-leak`
///    already flags the same source location. Both checkers use identical
///    `Allocator`/`Deallocator` specs, producing duplicate findings on
///    `malloc`/`free` patterns.
///
/// Note: `uninit-use` is intentionally NOT suppressed. While it can produce
/// false positives (e.g., `strcpy` not recognized as `StoreDeref` sanitizer),
/// suppressing it based on location overlap would also remove true positives
/// when other checkers produce false positives on the same allocation.
fn suppress_cross_checker_findings(findings: Vec<Finding>) -> Vec<Finding> {
    // Phase 1: collect locations from non-null-deref checkers (owned strings
    // to avoid borrow conflict with the consuming filter in phase 2).
    let mut non_null_sources = BTreeSet::new();
    let mut non_null_sinks = BTreeSet::new();
    let mut memory_leak_sources = BTreeSet::new();

    for f in &findings {
        if f.check != "null-deref" {
            if let Some(e) = f.path.first() {
                non_null_sources.insert(e.location.clone());
            }
            if let Some(e) = f.path.last() {
                non_null_sinks.insert(e.location.clone());
            }
        }
        if f.check == "memory-leak" {
            if let Some(e) = f.path.first() {
                memory_leak_sources.insert(e.location.clone());
            }
        }
    }

    // Phase 2: filter
    findings
        .into_iter()
        .filter(|f| {
            // Rule 1: suppress null-deref when source or sink is already covered
            if f.check == "null-deref" {
                let source_covered = f
                    .path
                    .first()
                    .is_some_and(|e| non_null_sources.contains(&e.location));
                let sink_covered = f
                    .path
                    .last()
                    .is_some_and(|e| non_null_sinks.contains(&e.location));
                if source_covered || sink_covered {
                    return false;
                }
            }
            // Rule 2: suppress generic-resource-leak when memory-leak covers it
            if f.check == "generic-resource-leak"
                && f.path
                    .first()
                    .is_some_and(|e| memory_leak_sources.contains(&e.location))
            {
                return false;
            }
            true
        })
        .collect()
}

/// Expand an overlap finding (`source_node == sink_node`) into per-instruction
/// protocol findings.
///
/// When GEPs are absent (e.g., `buffer[0]` with zero offset is folded), both
/// Load and Store instructions use the pointer value directly. The SVFG
/// collapses them into one node, and the runner's overlap detection creates a
/// single finding. This function walks AIR instructions to find all
/// dereferences of the overlapping pointer, creating one protocol `Finding`
/// per instruction with the instruction's actual source location.
fn expand_overlap_deref_findings(
    f: &checkers::finding::CheckerFinding,
    module: &AirModule,
    source_loc: Option<&(String, Option<String>)>,
    source_event: &str,
    sink_event: &str,
) -> Vec<Finding> {
    let Some(value_id) = f.source_node.as_value() else {
        return Vec::new();
    };

    // Use source node ValueId hex for precise per-allocation matching.
    let object = f
        .source_node
        .as_value()
        .map(ValueId::to_hex)
        .or_else(|| source_loc.as_ref().and_then(|(_, sym)| sym.clone()));
    let mut findings = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                let is_deref = match &inst.op {
                    Operation::Store => inst.operands.get(1) == Some(&value_id),
                    Operation::Load | Operation::Gep { .. } => {
                        inst.operands.first() == Some(&value_id)
                    }
                    _ => false,
                };

                if is_deref {
                    let sink_loc = format_location(&func.name, inst.span.as_ref(), module);
                    let sink_source_loc = inst
                        .span
                        .as_ref()
                        .and_then(|s| resolve_span_to_source_loc(s, module));
                    let sink_display = inst
                        .symbol
                        .as_ref()
                        .map_or_else(|| format!("{:?}", inst.op), |s| s.display_name.clone());

                    let mut path = Vec::new();
                    if let Some((loc, _)) = source_loc {
                        path.push(PathEvent {
                            location: loc.clone(),
                            event: source_event.to_string(),
                            state: None,
                            display_name: None,
                            source_loc: None,
                        });
                    }
                    path.push(PathEvent {
                        location: sink_loc,
                        event: sink_event.to_string(),
                        state: None,
                        display_name: Some(sink_display),
                        source_loc: sink_source_loc,
                    });

                    findings.push(Finding {
                        check: f.checker_name.clone(),
                        severity: format!("{:?}", f.severity).to_lowercase(),
                        cwe: f.cwe,
                        message: f.message.clone(),
                        path,
                        object: object.clone(),
                        display_name: None,
                    });
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(check: &str, source: &str, sink: &str) -> Finding {
        Finding {
            check: check.to_string(),
            severity: "warning".to_string(),
            cwe: None,
            message: format!("{check} finding"),
            path: vec![
                PathEvent {
                    location: source.to_string(),
                    event: "source".to_string(),
                    state: None,
                    display_name: None,
                    source_loc: None,
                },
                PathEvent {
                    location: sink.to_string(),
                    event: "sink".to_string(),
                    state: None,
                    display_name: None,
                    source_loc: None,
                },
            ],
            object: None,
            display_name: None,
        }
    }

    #[test]
    fn keep_uninit_use_even_when_uaf_covers_same_source() {
        // uninit-use is NOT suppressed because suppressing it globally
        // would also remove true positives on the uninit-use example.
        let findings = vec![
            make_finding("uninit-use", "process() at ?:5", "process() at ?:12"),
            make_finding("use-after-free", "process() at ?:5", "process() at ?:12"),
            make_finding("use-after-free", "process() at ?:5", "process() at ?:15"),
        ];
        let result = suppress_cross_checker_findings(findings);
        let checks: Vec<&str> = result.iter().map(|f| f.check.as_str()).collect();
        assert!(
            checks.contains(&"uninit-use"),
            "uninit-use should be kept, got: {checks:?}"
        );
        assert_eq!(checks.iter().filter(|c| **c == "use-after-free").count(), 2);
    }

    #[test]
    fn suppress_null_deref_when_uaf_covers_same_source() {
        let findings = vec![
            make_finding("null-deref", "process() at ?:5", "process() at ?:12"),
            make_finding("use-after-free", "process() at ?:5", "process() at ?:12"),
        ];
        let result = suppress_cross_checker_findings(findings);
        let checks: Vec<&str> = result.iter().map(|f| f.check.as_str()).collect();
        assert!(
            !checks.contains(&"null-deref"),
            "null-deref should be suppressed"
        );
        assert!(checks.contains(&"use-after-free"));
    }

    #[test]
    fn suppress_generic_resource_leak_when_memory_leak_covers() {
        let findings = vec![
            make_finding(
                "generic-resource-leak",
                "process() at ?:5",
                "process() at ?:10",
            ),
            make_finding("memory-leak", "process() at ?:5", "process() at ?:10"),
        ];
        let result = suppress_cross_checker_findings(findings);
        let checks: Vec<&str> = result.iter().map(|f| f.check.as_str()).collect();
        assert!(
            !checks.contains(&"generic-resource-leak"),
            "generic-resource-leak should be suppressed"
        );
        assert!(checks.contains(&"memory-leak"));
    }

    #[test]
    fn keep_uninit_use_when_no_other_checker_fires() {
        let findings = vec![make_finding(
            "uninit-use",
            "process() at ?:4",
            "process() at ?:7",
        )];
        let result = suppress_cross_checker_findings(findings);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].check, "uninit-use");
    }
}
