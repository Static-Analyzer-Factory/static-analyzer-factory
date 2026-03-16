# Taint Analysis

**Taint analysis** tracks the flow of untrusted data through a program to
determine if it can reach security-sensitive operations. It is SAF's primary
technique for detecting injection vulnerabilities, information leaks, and other
data-flow security issues.

## Core Concepts

### Sources, Sinks, and Sanitizers

| Concept | Definition | Examples |
|---------|-----------|----------|
| **Source** | Where untrusted ("tainted") data enters the program | `argv`, `getenv()`, `read()`, `fgets()` |
| **Sink** | A dangerous function that should never receive tainted data | `system()`, `execve()`, `printf()` format arg |
| **Sanitizer** | A function that validates or cleans data, removing the taint | Input validation, bounds checking, escaping |

### The Question

Taint analysis answers: "Can data from a source reach a sink without passing
through a sanitizer?"

If the answer is yes, a **finding** is reported -- a potential vulnerability.

## How SAF Performs Taint Analysis

SAF implements taint analysis as a graph reachability query over the
[ValueFlow graph](value-flow.md):

1. **Identify source nodes** in the ValueFlow graph (e.g., `argv` parameter,
   `getenv()` return value)
2. **Identify sink nodes** (e.g., `system()` argument)
3. **Identify sanitizer nodes** (optional -- nodes that "clean" the taint)
4. **BFS traversal** from sources to sinks, skipping paths through sanitizers
5. **Report findings** with deterministic trace paths

### BFS vs IFDS

SAF provides two taint analysis modes:

| Mode | Method | Precision | Speed |
|------|--------|-----------|-------|
| **BFS** | `q.taint_flow()` | Flow-insensitive | Fast |
| **IFDS** | `proj.ifds_taint()` | Context-sensitive, flow-sensitive | Slower |

BFS is sufficient for most vulnerability detection. IFDS provides higher
precision when false positives from flow-insensitive analysis are a concern.

## Using the Python SDK

### Basic Taint Flow

```python
from saf import Project, sources, sinks

proj = Project.open("program.ll")
q = proj.query()

# Find flows from argv to system()
findings = q.taint_flow(
    sources=sources.function_param("main", 1),  # argv
    sinks=sinks.call("system", arg_index=0),     # system()'s first arg
)

for f in findings:
    print(f"Finding: {f.finding_id}")
    if f.trace:
        for step in f.trace.steps:
            print(f"  -> {step}")
```

### Available Selectors

**Source selectors:**

| Selector | Description |
|----------|-------------|
| `sources.function_param(name, index)` | Function parameter by name and position |
| `sources.function_return(name)` | Return value of a named function |
| `sources.call(name)` | Return value from calls to a named function |

**Sink selectors:**

| Selector | Description |
|----------|-------------|
| `sinks.call(name, arg_index=N)` | Argument N of calls to a named function |
| `sinks.arg_to(name, index)` | Argument at index passed to a named function |

### With Sanitizers

```python
from saf import sources, sinks

findings = q.taint_flow(
    sources=sources.function_param("main", 1),
    sinks=sinks.call("system", arg_index=0),
    sanitizers=sources.function_return("validate_input"),  # Paths through this function are safe
)
```

## Common Vulnerability Patterns

| Vulnerability | CWE | Source | Sink |
|--------------|-----|--------|------|
| Command injection | CWE-78 | `argv`, `getenv()` | `system()`, `execve()` |
| Format string | CWE-134 | User input | `printf()` format arg |
| SQL injection | CWE-89 | HTTP parameters | SQL query functions |
| Path traversal | CWE-22 | User input | `fopen()`, `open()` |
| Buffer overflow | CWE-120 | `malloc()` return | Unchecked memory write |

## Checker Framework

For common patterns, SAF provides built-in checkers that pre-configure the
appropriate sources, sinks, and modes:

```python
# Instead of manually specifying sources and sinks:
findings = proj.check("memory-leak")
findings = proj.check("use-after-free")
findings = proj.check("double-free")

# Or run all 9 built-in checkers at once
all_findings = proj.check_all()
```

The checker framework supports 9 built-in checkers covering memory safety,
information flow, and resource management. See the
[Python SDK reference](../api-reference/python-sdk.md) for the full list.

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=taint_flow&graph=valueflow" loading="lazy"></iframe>
</div>

## Next Steps

- [Tutorials](/tutorials/) -- Hands-on guides for UAF, leaks, double-free, and taint analysis
- [Python SDK Reference](../api-reference/python-sdk.md) -- Full API reference for selectors, checkers, and queries
