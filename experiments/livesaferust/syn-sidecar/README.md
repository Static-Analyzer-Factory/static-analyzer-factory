# syn Sidecar

This is the next parser layer for the LiveSafeRust experiment. It is separate
from the runnable Node POC because the current desktop environment does not
have `cargo`, `rustc`, or `rust-analyzer` installed.

The goal is deliberately modest: parse real Rust syntax with `syn`, emit the
same sidecar-style function/call facts as the JavaScript POC, and keep that JSON
shape replaceable by rust-analyzer/HIR facts later.

Run when a Rust toolchain is available:

```bash
cd experiments/livesaferust/syn-sidecar
cargo run -- ../out/layered/vulnerable.rs > ../out/layered/syn-facts.json
```
Expected output schema:

```json
{
  "schema": "livesaferust.syn-sidecar/0.1",
  "source": "path/to/file.rs",
  "functions": [
    {
      "name": "main",
      "params": [],
      "range": { "startLine": 1, "endLine": 5 },
      "hash": "...",
      "calls": ["read_user", "dispatch"],
      "sourceHints": ["env::args"],
      "sinkHints": ["Command::new"]
    }
  ]
}
```
