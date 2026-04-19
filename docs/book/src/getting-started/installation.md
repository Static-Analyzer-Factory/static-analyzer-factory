# Installation

SAF can be used in two ways: in the browser (no install needed) or locally via
Docker for full analysis capabilities.

## Browser (No Install)

The fastest way to try SAF is the [Playground](/playground/). It runs entirely in
your browser using WebAssembly -- no server, no Docker, no setup.

The browser version supports:
- Writing C code or LLVM IR
- Visualizing CFGs, call graphs, def-use chains, and value-flow graphs
- Running pointer analysis
- Exporting results as JSON

For the full Python SDK, checker framework, and LLVM bitcode support, use the
local installation below.

## Local Installation (Docker)

The local installation uses Docker to provide a complete development environment
with Rust, Python 3.12, LLVM 18 (default — also available: LLVM 22, opt-in), and
all dependencies pre-installed. See [LLVM Version Support](llvm-versions.md) for
when to pick each variant.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and running
- [Git](https://git-scm.com/)
- A terminal

### Step 1: Clone the Repository

```bash
git clone https://github.com/Static-Analyzer-Factory/static-analyzer-factory.git
cd static-analyzer-factory
```

### Step 2: Start the Development Shell

```bash
make shell
```

This drops you into an interactive shell inside the Docker container. The project
directory is mounted at `/workspace`.

On first run, the container automatically:
1. Creates a Python virtual environment
2. Installs pytest and development dependencies
3. Builds the SAF Python SDK with `maturin develop --release`

Subsequent runs skip the build if `saf` is already importable.

### Step 3: Verify the Installation

Inside the Docker shell:

```bash
python3 -c "import saf; print('SAF version:', saf.version())"
```

You should see the SAF version printed without errors.

To run the full test suite:

```bash
# Exit Docker shell first (Ctrl+D), then:
make test
```

## What's Included

The Docker environment provides:

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable | Core analysis engine |
| Python | 3.12 | SDK and tutorials |
| LLVM/Clang | 18 (default) or 22 (opt-in) | C/C++ to LLVM IR compilation |
| maturin | latest | Python-Rust bridge (PyO3) |

## Available Make Commands

```bash
make help              # Show all available commands (both LLVM variants)
make test              # Run all tests (Rust + Python) in Docker — LLVM 18
make test-llvm22       # Same, but inside the LLVM 22 image
make lint              # Run clippy + rustfmt check in Docker — LLVM 18
make lint-llvm22       # Same, but inside the LLVM 22 image
make fmt               # Auto-format all Rust code in Docker
make shell             # Interactive dev shell (LLVM 18)
make shell-llvm22      # Interactive dev shell (LLVM 22)
make build             # Build minimal runtime Docker image (LLVM 18)
make build-llvm22      # Build minimal runtime Docker image (LLVM 22)
make clean             # Remove Docker volumes and local target/
```

Pick `*-llvm22` variants only when you need to analyze IR emitted by clang-22
(or otherwise post-LLVM-18). The `*-llvm22` targets build a separate Docker
image from `apt.llvm.org` packages; the first run takes a few extra minutes.

## Next Steps

- [First Analysis](first-analysis.md) -- Analyze a C program end to end
- [Playground Tour](playground-tour.md) -- Learn the browser-based playground
