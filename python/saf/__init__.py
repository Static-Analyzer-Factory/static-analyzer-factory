"""Static Analyzer Factory — Python SDK.

This package provides a Python interface to SAF's static analysis capabilities,
designed for both AI agents and human developers.

Example:
    from saf import Project, sources, sinks

    proj = Project.open("program.air.json")
    q = proj.query()

    findings = q.taint_flow(
        sources=sources.function_param("process_input", 0),
        sinks=sinks.call("dangerous_sink"),
    )

    for f in findings:
        print(f.source_location, "->", f.sink_location)
        print(f.trace.pretty())
"""

from saf._saf import (
    Project,
    Query,
    Finding,
    Trace,
    TraceStep,
    Selector,
    SelectorSet,
    SafError,
    FrontendError,
    AnalysisError,
    QueryError,
    ConfigError,
    CheckerFinding,
    PathSensitiveResult,
    ResourceTable,
    TypestateResult,
    TypestateFinding,
    TypestateSpec,
    typestate_specs,
    version,
    # Resource role constants
    Allocator,
    Deallocator,
    Reallocator,
    Acquire,
    Release,
    NullSource,
    Dereference,
    # Reachability mode constants
    MayReach,
    MustNotReach,
    # Severity constants
    Info,
    Warning,
    Error,
    Critical,
)

from saf import sources
from saf import sinks
from saf import sanitizers
from saf import viz

__all__ = [
    # Core classes
    "Project",
    "Query",
    "Finding",
    "Trace",
    "TraceStep",
    # Selectors
    "Selector",
    "SelectorSet",
    # Selector modules
    "sources",
    "sinks",
    "sanitizers",
    # Visualization
    "viz",
    # Checker types
    "CheckerFinding",
    "PathSensitiveResult",
    "ResourceTable",
    # Exceptions
    "SafError",
    "FrontendError",
    "AnalysisError",
    "QueryError",
    "ConfigError",
    # Resource role constants
    "Allocator",
    "Deallocator",
    "Reallocator",
    "Acquire",
    "Release",
    "NullSource",
    "Dereference",
    # Reachability mode constants
    "MayReach",
    "MustNotReach",
    # Severity constants
    "Info",
    "Warning",
    "Error",
    "Critical",
    # Typestate types
    "TypestateResult",
    "TypestateFinding",
    "TypestateSpec",
    "typestate_specs",
    # Functions
    "version",
]
