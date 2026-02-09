# Integration

This category covers programmatic API usage, export formats, and batch processing for integrating SAF into your workflow.

## Learning Objectives

After completing these tutorials, you will be able to:
- Explore SAF's API schema programmatically
- Export analysis graphs to JSON for visualization
- Generate SARIF reports for CI/CD integration
- Process multiple files and large codebases

## Tutorials

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Schema Discovery](01-schema-discovery/) | Beginner | Exploring the API |
| 02 | [JSON Export](02-json-export/) | Beginner | Exporting graphs |
| 03 | [SARIF Reporting](03-sarif-reporting/) | Intermediate | CI/CD integration |
| 04 | [Batch Scanning](04-batch-scanning/) | Advanced | Multi-file analysis |

## Difficulty Progression

- **Beginner (01-02)**: Single file, simple export
- **Intermediate (03)**: SARIF format for CI integration
- **Advanced (04)**: Multi-file, multi-language scanning

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first

## Key Concepts

### API Schema
SAF's `schema()` method returns a dictionary describing available:
- Analysis methods and their parameters
- Return types and structures
- Selector patterns for sources/sinks

### Export Formats

| Format | Use Case | Tool Support |
|--------|----------|--------------|
| JSON | Visualization, custom processing | Any JSON parser |
| SARIF 2.1.0 | CI/CD integration | GitHub, VS Code, many IDEs |

### SARIF Integration
SARIF (Static Analysis Results Interchange Format) enables:
- GitHub Code Scanning integration
- VS Code SARIF Viewer
- Azure DevOps security dashboards

Example workflow:
```bash
python3 detect.py > results.sarif
# Upload to CI system
gh api repos/{owner}/{repo}/code-scanning/sarifs -f sarif=@results.sarif
```

### Batch Processing Patterns
For large codebases:
1. **Parallel compilation**: Compile source files to IR concurrently
2. **Incremental analysis**: Track file hashes, skip unchanged files
3. **Result aggregation**: Merge findings across files

## Related Categories

All other categories use techniques covered here for their `detect.py` scripts.
