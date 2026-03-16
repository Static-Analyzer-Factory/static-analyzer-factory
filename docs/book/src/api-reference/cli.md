# CLI

The SAF command-line interface provides access to analysis capabilities from the
terminal. The binary is named `saf`.

## Global Options

| Option | Description |
|--------|-------------|
| `--json-errors` | Output errors as JSON for machine-readable consumption |

## Commands

### saf index

Index input files via a frontend to produce AIR-JSON.

```bash
saf index program.ll
saf index program.ll --output bundle.air.json
saf index --frontend air-json bundle.air.json
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `<inputs>` | positional, required | | Input files to index |
| `--frontend <frontend>` | `llvm`, `air-json` | `llvm` | Frontend to use for ingestion |
| `--output <path>` | path | stdout | Write AIR-JSON output to file instead of stdout |

---

### saf run

Run analysis passes on input programs. This is the primary command -- it ingests
input files, builds internal representations (CFG, call graph, pointer analysis),
runs the selected checkers, and produces findings.

```bash
saf run program.ll
saf run program.ll --mode fast
saf run program.ll --format sarif --output report.sarif
saf run program.ll --checkers memory-leak,null-deref --pta cspta --pta-k 2
```

#### Input Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `<inputs>` | positional, required | | Input files to analyze |
| `--frontend <frontend>` | `llvm`, `air-json` | `llvm` | Frontend to use for ingestion |
| `--specs <path>` | path | | Additional spec files or directories |

#### Analysis Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--mode <mode>` | `fast`, `precise` | `precise` | Analysis mode |
| `--checkers <list>` | string | `all` | Checkers to run (comma-separated, or `all` / `none`) |
| `--combined` | flag | | Run combined PTA + abstract interpretation |

#### Pointer Analysis Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--pta <variant>` | `andersen`, `cspta`, `fspta`, `dda` | `andersen` | PTA variant |
| `--solver <backend>` | `worklist`, `datalog` | `worklist` | PTA solver backend |
| `--pts-repr <repr>` | `auto`, `btreeset`, `fxhash`, `roaring`, `bdd` | `auto` | Points-to set representation |
| `--pta-k <n>` | integer | `2` | k-CFA depth (`cspta` only) |
| `--field-sensitivity <level>` | `struct-fields`, `array-index`, `flat` | `struct-fields` | Field sensitivity level |
| `--max-pta-iterations <n>` | integer | | Maximum PTA iterations |

#### Z3 Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--path-sensitive` | flag | | Enable Z3 path-sensitive checker filtering |
| `--z3-prove` | flag | | Prove assertions via Z3 |
| `--z3-refine-alias` | flag | | Refine alias results via Z3 |
| `--z3-check-reachability` | flag | | Check path reachability via Z3 |
| `--z3-timeout <ms>` | integer | `5000` | Z3 solver timeout in milliseconds |

#### Typestate and Taint Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--typestate <name>` | string | | Run built-in typestate spec |
| `--typestate-custom <path>` | path | | Run custom typestate spec from YAML |
| `--ifds-taint <path>` | path | | Run IFDS taint analysis with config file |

#### Output Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--format <fmt>` | `human`, `json`, `sarif` | `human` | Output format |
| `--output <path>` | path | stdout | Write output to file |
| `--diagnostics` | flag | | Include checker/PTA diagnostics |
| `--verbose` | flag | | Show timing, resource table, stats |

---

### saf query

Query analysis results. Requires `--input` and a subcommand.

```bash
saf query points-to --input program.ll 0x00ab1234...
saf query alias --input program.ll 0x001a... 0x002b...
saf query flows --input program.ll 0xsource... 0xsink...
saf query taint --input program.ll 0xsource... 0xsink...
saf query reachable --input program.ll 0xfunc1... 0xfunc2...
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--input <files>` | path(s), required | | Input files to analyze |
| `--frontend <frontend>` | `llvm`, `air-json` | `llvm` | Frontend to use for ingestion |

#### Subcommands

| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `points-to` | `<pointer>` (hex value ID) | Points-to set for a value |
| `alias` | `<p> <q>` (hex value IDs) | May-alias check between two pointers |
| `flows` | `<source> <sink>` (hex value IDs) | Data-flow reachability |
| `taint` | `<source> <sink>` (hex value IDs) | Taint-flow query |
| `reachable` | `<func_ids>...` (hex function IDs) | Call-graph reachability from functions |

---

### saf export

Export graphs or findings in various formats.

```bash
saf export cfg --input program.ll
saf export callgraph --input program.ll --format dot --output cg.dot
saf export findings --input program.ll --format sarif --output report.sarif
saf export valueflow --input program.ll --format json --output vfg.json
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `<target>` | positional, required | | Export target (see below) |
| `--input <files>` | path(s), required | | Input files to analyze |
| `--format <fmt>` | `json`, `sarif`, `dot`, `html` | `json` | Output format |
| `--output <path>` | path | stdout | Write output to file instead of stdout |
| `--function <name>` | string | | Filter to a specific function (for CFG export) |
| `--frontend <frontend>` | `llvm`, `air-json` | `llvm` | Frontend to use for ingestion |

#### Export Targets

| Target | Description |
|--------|-------------|
| `cfg` | Control-flow graph |
| `callgraph` | Call graph (inter-procedural) |
| `defuse` | Def-use chains |
| `valueflow` | Value-flow graph |
| `svfg` | Sparse Value-Flow Graph |
| `findings` | Analysis findings / bug reports |
| `pta` | Points-to analysis results |

---

### saf schema

Print the SAF schema (supported frontends, queries, checkers).

```bash
saf schema
saf schema --checkers
saf schema --frontends
saf schema --format json
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--checkers` | flag | | List available checkers only |
| `--frontends` | flag | | List available frontends only |
| `--format <fmt>` | `human`, `json`, `sarif` | `human` | Output format |

---

### saf specs

Manage function specifications (list, validate, look up).

```bash
saf specs list
saf specs list --verbose
saf specs validate path/to/specs.yaml
saf specs lookup malloc
```

#### Subcommands

| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `list` | `[--verbose]` | List loaded function specifications |
| `validate` | `<path>` (required) | Validate spec file or directory |
| `lookup` | `<name>` (required) | Look up the spec for a function by name |

---

### saf incremental

Run incremental analysis with caching. Only recomputes what changed between runs.

```bash
saf incremental src/*.ll
saf incremental src/*.ll --cache-dir .saf-cache
saf incremental src/*.ll --plan
saf incremental src/*.ll --clean
saf incremental src/*.ll --export-summaries summaries.yaml
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `<inputs>` | positional, required | | Input files to analyze |
| `--frontend <frontend>` | `llvm`, `air-json` | `llvm` | Frontend to use for ingestion |
| `--mode <mode>` | `sound`, `best-effort` | `best-effort` | Precision mode for incremental analysis |
| `--cache-dir <path>` | path | `.saf-cache` | Cache directory for incremental state |
| `--plan` | flag | | Dry-run: show what would be recomputed without running analysis |
| `--clean` | flag | | Clear the cache before analysis |
| `--export-summaries <path>` | path | | Export computed summaries as YAML to the given path |

---

### saf help

Show topic-based help guides.

```bash
saf help               # Overview of all commands and topics
saf help run           # How to run analyses and configure passes
saf help checkers      # Built-in bug checkers and how to select them
saf help pta           # Pointer analysis variants and configuration
saf help typestate     # Typestate checking and custom protocol specs
saf help taint         # Taint analysis modes and configuration
saf help z3            # Z3-based path sensitivity and refinement
saf help export        # Export targets, formats, and examples
saf help specs         # Function specification format and discovery
saf help incremental   # Incremental analysis and caching
saf help examples      # Common usage patterns and recipes
```

#### Available Topics

| Topic | Description |
|-------|-------------|
| `run` | How to run analyses and configure passes |
| `checkers` | Built-in bug checkers and how to select them |
| `pta` | Pointer analysis variants and configuration |
| `typestate` | Typestate checking and custom protocol specs |
| `taint` | Taint analysis modes and configuration |
| `z3` | Z3-based path sensitivity and refinement |
| `export` | Export targets, formats, and examples |
| `specs` | Function specification format and discovery |
| `incremental` | Incremental analysis and caching |
| `examples` | Common usage patterns and recipes |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (no findings, or findings exported) |
| 1 | Error (invalid input, analysis failure) |
| 2 | Findings detected (when used in CI mode) |

## Examples

### Quick Scan

```bash
# Run all checkers with defaults (Andersen PTA, precise mode)
saf run program.ll
```

### CI/CD Integration

```bash
# Run all checkers and output SARIF for GitHub Code Scanning
saf run program.ll --format sarif --output results.sarif

# Check exit code for pass/fail
if saf run program.ll --format json | jq '.findings | length' | grep -q '^0$'; then
    echo "No findings"
else
    echo "Findings detected"
    exit 1
fi
```

### Batch Analysis

```bash
# Analyze multiple files
for f in src/*.ll; do
    saf run "$f" --format sarif --output "results/$(basename "$f" .ll).sarif"
done
```

### Path-Sensitive Analysis

```bash
# Reduce false positives with Z3-based path feasibility filtering
saf run program.ll --path-sensitive --z3-timeout 10000
```

### Export Call Graph as DOT

```bash
saf export callgraph --input program.ll --format dot --output cg.dot
dot -Tpng cg.dot -o cg.png
```

### Incremental Analysis

```bash
# First run caches results; subsequent runs only re-analyze changed files
saf incremental src/*.ll --cache-dir .saf-cache
```

### Demand-Driven Points-To Query

```bash
saf query points-to --input program.ll 0x00ab1234abcd5678
```
