# LiveSafeRust VS Code Prototype

This is a small editor integration for the LiveSafeRust paper prototype. It
publishes real VS Code diagnostics for Rust files through
`vscode.languages.createDiagnosticCollection`.

What it is:

- a working VS Code extension prototype;
- a SAF-backed mode that runs the paper path:
  Rust source -> LLVM IR -> SAF JSON -> LiveSafeRust cache/worklist -> VS Code diagnostics;
- a fast source-sidecar fallback for simple local demos.

What it is not:

- not a rust-analyzer plugin;
- not a Salsa query implementation;
- not a full rust-analyzer/Salsa integration yet.

## Run In VS Code

Open this directory in VS Code:

```bash
experiments/livesaferust/vscode-extension
```

Then press `F5` and choose "Run LiveSafeRust Extension" if prompted. The default
backend is `saf-backed`, so diagnostics run on save or from the command palette:

```text
LiveSafeRust: Rescan Current Rust File
LiveSafeRust: Show Output
```

Open `sample/vulnerable.rs`, run the command above, and the `Command::new(cmd)`
line should appear in Problems as a `LiveSafeRust/SAF-POC` warning.
`sample/sanitized.rs` should stay clean.

The SAF-backed backend expects the same local prerequisites as the paper
experiment: `python3`, `node`, `rustc`, and Docker/SAF services from the SAF repo.
For a fresh SAF Docker image, enable `livesaferust.ensureSafSdk` once so the
extension passes the same SDK setup flag used by the R13 experiment script.

On macOS, if VS Code was launched from Finder/Dock, the extension explicitly adds
common Homebrew and Cargo paths (`/opt/homebrew/bin`, `/usr/local/bin`,
`~/.cargo/bin`) before invoking the SAF-backed scripts.

For a no-Docker editor smoke test, set:

```json
{
  "livesaferust.backend": "source-sidecar"
}
```

If the `code` command is installed, this directory can be opened with:

```bash
code experiments/livesaferust/vscode-extension
```

## Test

```bash
npm test
```
