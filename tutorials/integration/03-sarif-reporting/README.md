# Tutorial: SARIF Reporting

**Difficulty:** Intermediate | **Time:** 20 minutes | **Category:** Integration

## What You Will Learn

- How to generate a SARIF 2.1.0 vulnerability report from SAF findings
- The structure of a SARIF report (runs, results, rules, code flows)
- How to include trace information as SARIF code flows
- How to use SARIF reports with GitHub Code Scanning, VS Code, and DefectDojo

## Prerequisites

Complete [Tutorial 02: JSON Export](../02-json-export/README.md) before starting this one.

## What is SARIF?

**SARIF** (Static Analysis Results Interchange Format) is an OASIS standard
(version 2.1.0) for expressing the output of static analysis tools. It is a
JSON-based format that provides a structured, tool-independent way to represent
findings.

SARIF is supported by:

- **GitHub Code Scanning** -- upload SARIF to see findings inline in pull requests
- **VS Code** -- the SARIF Viewer extension renders findings with source locations
- **DefectDojo** -- import SARIF for vulnerability management
- **Azure DevOps** -- native SARIF support in security dashboards

## The Vulnerability

A simple command injection (CWE-78) program:

```c
void run_command(const char *cmd) {
    printf("Running: %s\n", cmd);
    system(cmd);  // SINK
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: program <command>\n");
        return 1;
    }
    run_command(argv[1]);  // SOURCE flows to SINK
    return 0;
}
```

User-controlled `argv[1]` flows through `run_command` to `system()` without
sanitization.

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Found 1 finding(s)

SARIF report written to: .../report.sarif.json
Results: 1

Report preview:
{
  "version": "2.1.0",
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/...",
  "runs": [{
    "tool": {
      "driver": {
        "name": "SAF",
        ...
```

## SARIF Structure

```json
{
  "version": "2.1.0",
  "$schema": "...",
  "runs": [{
    "tool": {
      "driver": {
        "name": "SAF",
        "version": "0.1.0",
        "rules": [{"id": "CWE-78", "name": "OSCommandInjection", ...}]
      }
    },
    "results": [
      {
        "ruleId": "CWE-78",
        "level": "error",
        "message": {"text": "..."},
        "locations": [...],
        "codeFlows": [...]
      }
    ]
  }]
}
```

| Key | Description |
|-----|-------------|
| `runs[].tool.driver` | Information about the analysis tool |
| `runs[].tool.driver.rules` | Rule definitions (CWE IDs, descriptions) |
| `runs[].results` | Array of findings |
| `results[].ruleId` | Links to a rule definition |
| `results[].codeFlows` | Step-by-step trace from source to sink |
| `results[].fingerprints` | Deterministic IDs for deduplication |

## Using SARIF Reports

### GitHub Code Scanning

Upload the SARIF file using the GitHub API:

```bash
gh api repos/{owner}/{repo}/code-scanning/sarifs \
  -X POST \
  -F sarif=@report.sarif.json \
  -F ref=refs/heads/main
```

### VS Code SARIF Viewer

1. Install the "SARIF Viewer" extension
2. Open `report.sarif.json` in VS Code
3. Findings appear with inline annotations at the source locations

### DefectDojo

Import SARIF reports via the DefectDojo API or web UI:

```bash
curl -X POST "https://defectdojo.example.com/api/v2/import-scan/" \
  -H "Authorization: Token <token>" \
  -F "scan_type=SARIF" \
  -F "file=@report.sarif.json"
```

## Next Steps

- [Tutorial 04: Batch Scanning](../04-batch-scanning/README.md) - Scan multiple programs in different languages
