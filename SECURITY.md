# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in SAF, please report it responsibly.

**Email:** [open an issue](https://github.com/Static-Analyzer-Factory/static-analyzer-factory/issues) with the label `security`, or contact the maintainers directly.

Please include:
- A description of the vulnerability
- Steps to reproduce
- Potential impact

We will acknowledge receipt within 48 hours and aim to provide a fix or mitigation plan within 7 days.

## Scope

SAF is a static analysis tool — it processes untrusted program inputs (LLVM IR, C source). Security concerns include:
- Crashes or resource exhaustion when processing malformed input files
- Incorrect analysis results that could cause users to miss real vulnerabilities
- Dependencies with known CVEs

## Supported Versions

Security fixes are applied to the latest version on the `main` branch. There are no LTS or backport branches at this time.
