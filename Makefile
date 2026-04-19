.PHONY: test test-rust test-python test-crate test-clean lint lint-clippy lint-fmt fmt shell build rebuild clean cloc help \
	compile-lua-fixtures test-incremental bench-incremental \
	compile-ptaben test-ptaben test-ptaben-json clean-ptaben \
	compile-ptaben-llvm22 test-ptaben-llvm22 test-ptaben-llvm22-json \
	compile-llvm22-syntax-fixtures \
	compile-svcomp compile-svcomp-category test-svcomp test-svcomp-category test-svcomp-json clean-svcomp \
	svcomp-categories \
	compile-juliet test-juliet test-juliet-json juliet-categories clean-juliet \
	compile-oracle verify-oracle verify-props verify-props-quick verify-quick verify clean-oracle \
	notebook wasm wasm-dev playground playground-dev playground-deploy \
	prepare-cruxbc test-cruxbc test-cruxbc-svf test-cruxbc-svf-mem2reg test-cruxbc-llvm-cg compare-cruxbc clean-cruxbc \
	tutorials tutorials-dev site site-dev-all site-dev site-clean \
	shell-llvm22 test-llvm22 lint-llvm22 build-llvm22

# Feature sets for LLVM 22 cargo invocations — disables defaults (which pin
# llvm-18) and re-activates the same non-LLVM features the workspace would
# normally get from each crate's default set.
#
# SAF_LLVM22_FEATURES covers the full workspace (for `cargo build/test --workspace`).
# Per-package invocations (`cargo run -p saf-bench`) need the subset that maps
# to that package's dependency graph.
SAF_LLVM22_FEATURES := --no-default-features --features "saf-frontends/llvm-22,saf-analysis/z3-solver,saf-core/logging-subscriber,saf-cli/llvm-22,saf-bench/llvm-22,saf-trace/llvm-22,saf-datalog/llvm-22,saf-test-utils/llvm-22"
SAF_LLVM22_FEATURES_BENCH := --no-default-features --features "saf-frontends/llvm-22,saf-analysis/z3-solver,saf-core/logging-subscriber,saf-cli/llvm-22,saf-bench/llvm-22"
SAF_LLVM22_FEATURES_CLI := --no-default-features --features "saf-frontends/llvm-22,saf-analysis/z3-solver,saf-core/logging-subscriber,saf-cli/llvm-22"

help: ## Show this help message
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-32s\033[0m %s\n", $$1, $$2}'

test: ## Run all tests (Rust + Python, cached)
	docker compose run --rm test

test-rust: ## Run Rust tests only (no Python rebuild)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c "cargo nextest run --workspace --exclude saf-python"

test-python: ## Run Python tests only (assumes extension is built)
	docker compose run --rm dev sh -c "pytest python/tests/ -v"

test-crate: ## Run tests for a single crate (usage: make test-crate CRATE=saf-core)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c "cargo nextest run -p $(CRATE)"

test-clean: ## Run all tests (hermetic — clean build, no cache)
	docker compose run --rm --build test-clean

lint: lint-clippy lint-fmt ## Run clippy and rustfmt check inside Docker

lint-clippy: ## Run clippy only inside Docker
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
		"cargo clippy --workspace -- -D warnings"

lint-fmt: ## Run rustfmt check only inside Docker
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
		"cargo fmt --all -- --check"

fmt: ## Auto-format all Rust code inside Docker
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo fmt --all

shell: ## Open an interactive dev shell inside Docker (LLVM 18)
	docker compose run --rm dev

shell-llvm22: ## Open an interactive dev shell inside Docker (LLVM 22)
	docker compose run --rm dev-llvm22

test-llvm22: ## Run all tests (Rust + Python) inside the LLVM 22 image
	docker compose run --rm test-llvm22

lint-llvm22: ## Run clippy + rustfmt check inside the LLVM 22 image
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev-llvm22 sh -c \
		"cargo clippy --workspace $(SAF_LLVM22_FEATURES) -- -D warnings && cargo fmt --all -- --check"

build: ## Build the runtime Docker image (LLVM 18)
	docker compose build build

build-llvm22: ## Build the runtime Docker image (LLVM 22)
	docker compose build build-llvm22

rebuild: ## Rebuild Docker images (use after Dockerfile changes)
	docker compose build

clean: ## Remove Docker volumes and local target/
	docker compose down -v
	rm -rf target/

notebook: ## Start Jupyter Lab with port forwarding (no rebuild)
	docker compose run --rm -p 8888:8888 dev sh -c \
		'pip install jupyterlab ipycytoscape && jupyter lab --ip=0.0.0.0 --allow-root --no-browser'

# --- Statistics ---

cloc: ## Count lines of code in core source (crates + python)
	@cloc --exclude-dir=target,.git crates/ python/

# --- Incremental Analysis Fixtures & Benchmarks ---

compile-lua-fixtures: ## Compile Lua 5.4.7 to LLVM IR for incremental analysis fixtures
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c 'bash scripts/compile-lua-fixtures.sh'

test-incremental: ## Run incremental analysis E2E tests (Lua fixtures, ~60s)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c 'cargo nextest run -p saf-analysis --test incremental_e2e -- --ignored'

bench-incremental: ## Run incremental analysis benchmark (CPython, requires compile-incremental-bench first)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c 'cargo run --release -p saf-bench -- incremental --compiled-dir tests/benchmarks/incremental/cpython/.compiled --patches-dir tests/benchmarks/incremental/patches -o /workspace/tests/benchmarks/incremental/results.json'

# --- PTABen Benchmark Suite ---

compile-ptaben: ## Compile PTABen test suite with LLVM 18 (run once after clone/update)
	@echo "Compiling PTABen test suite..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/compile-ptaben.sh

compile-ptaben-llvm22: ## Compile PTABen test suite with LLVM 22 to .compiled-llvm22/
	@echo "Compiling PTABen test suite with clang-22..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev-llvm22 ./scripts/compile-ptaben.sh

compile-llvm22-syntax-fixtures: ## Compile LLVM 19-22 syntax-target fixtures with clang-22
	@echo "Compiling LLVM 22 syntax fixtures with clang-22..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev-llvm22 ./tests/programs/compile-llvm22-syntax.sh

test-ptaben: ## Run PTABen benchmark suite against SAF (LLVM 18)
	@echo "Running PTABen benchmarks..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c '\
	  cargo build --release -p saf-cli && \
	  cargo run --release -p saf-bench -- ptaben \
	    --compiled-dir tests/benchmarks/ptaben/.compiled'

test-ptaben-llvm22: ## Run PTABen benchmark suite against SAF (LLVM 22)
	@echo "Running PTABen benchmarks on LLVM 22..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev-llvm22 sh -c '\
	  cargo build --release -p saf-cli $(SAF_LLVM22_FEATURES_CLI) && \
	  cargo run --release -p saf-bench $(SAF_LLVM22_FEATURES_BENCH) -- ptaben \
	    --compiled-dir tests/benchmarks/ptaben/.compiled-llvm22'

test-ptaben-json: ## Run PTABen benchmarks with JSON output (LLVM 18)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c '\
	  cargo build --release -p saf-cli && \
	  cargo run --release -p saf-bench -- ptaben \
	    --compiled-dir tests/benchmarks/ptaben/.compiled \
	    -o /workspace/tests/benchmarks/ptaben/results.json'

test-ptaben-llvm22-json: ## Run PTABen benchmarks with JSON output (LLVM 22)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev-llvm22 sh -c '\
	  cargo build --release -p saf-cli $(SAF_LLVM22_FEATURES_CLI) && \
	  cargo run --release -p saf-bench $(SAF_LLVM22_FEATURES_BENCH) -- ptaben \
	    --compiled-dir tests/benchmarks/ptaben/.compiled-llvm22 \
	    -o /workspace/tests/benchmarks/ptaben/results-llvm22.json'

clean-ptaben: ## Remove compiled PTABen bitcode (both LLVM 18 and LLVM 22)
	rm -rf tests/benchmarks/ptaben/.compiled tests/benchmarks/ptaben/.compiled-llvm22

# --- SV-COMP Benchmark Suite ---
# Use AGGRESSIVE=1 for aggressive mode (more bug detection, slightly higher FP risk)

# Helper to add --aggressive flag when AGGRESSIVE=1
SVCOMP_FLAGS := $(if $(filter 1,$(AGGRESSIVE)),--aggressive,)

compile-svcomp: ## Compile SV-COMP benchmarks with LLVM 18 (run once after clone/update)
	@echo "Compiling SV-COMP benchmarks..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/compile-svcomp.sh --verbose

compile-svcomp-category: ## Compile single SV-COMP category (CAT=ReachSafety)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/compile-svcomp.sh --category $(CAT) --verbose

test-svcomp: ## Run SV-COMP benchmarks (AGGRESSIVE=1 for aggressive mode)
	@echo "Running SV-COMP benchmarks$(if $(filter 1,$(AGGRESSIVE)), (aggressive mode),)..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled $(SVCOMP_FLAGS)

test-svcomp-category: ## Run single SV-COMP category (CAT=memsafety AGGRESSIVE=1)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled \
		--category $(CAT) $(SVCOMP_FLAGS)

svcomp-categories: ## List available SV-COMP benchmark categories
	@echo "Memory Safety:"
	@echo "  memsafety, memsafety-ext, memsafety-ext2, memsafety-ext3"
	@echo "  array-memsafety, ldv-memsafety, pthread-memsafety"
	@echo ""
	@echo "Arrays:"
	@echo "  array-examples, array-programs, array-patterns, array-crafted"
	@echo "  array-industry-pattern, array-multidimensional"
	@echo ""
	@echo "Loops:"
	@echo "  loops, loops-crafted-1, loop-simple, loop-crafted, loop-new"
	@echo "  loop-invariants, loop-invgen, loop-lit, loop-acceleration"
	@echo ""
	@echo "Concurrency/Pthread:"
	@echo "  pthread, pthread-atomic, pthread-complex, pthread-ext, pthread-lit"
	@echo "  pthread-divine, pthread-driver-races, pthread-race-challenges"
	@echo "  ldv-races, ldv-linux-3.14-races"
	@echo ""
	@echo "Overflow:"
	@echo "  signedintegeroverflow-regression, unsignedintegeroverflow-sas23"
	@echo ""
	@echo "Recursion:"
	@echo "  recursive, recursive-simple, recursive-with-pointer"
	@echo ""
	@echo "Termination:"
	@echo "  termination-crafted, termination-numeric, termination-nla"
	@echo ""
	@echo "Software Systems:"
	@echo "  sqlite, openssl, busybox-1.22.0, coreutils-v8.31, aws-c-common"
	@echo ""
	@echo "Usage: make test-svcomp-category CAT=memsafety"
	@echo "       make test-svcomp-category CAT=memsafety AGGRESSIVE=1"

test-svcomp-json: ## Run SV-COMP benchmarks with JSON output (AGGRESSIVE=1 supported)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled \
		--json $(SVCOMP_FLAGS)

clean-svcomp: ## Remove compiled SV-COMP bitcode
	rm -rf tests/benchmarks/sv-benchmarks/.compiled

# --- Juliet Benchmark Suite (NIST CWE Test Suite) ---
# Precision/Recall/F1 scoring per CWE category

compile-juliet: ## Compile Juliet tests (15 supported CWEs) with LLVM 18 + mem2reg
	@echo "Compiling Juliet test suite..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/compile-juliet.sh --verbose

test-juliet: ## Run Juliet benchmarks with precision/recall/F1 (CWE=CWE476 to filter)
	@echo "Running Juliet benchmarks..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- juliet \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet \
		$(if $(CWE),--cwe $(CWE),)

test-juliet-json: ## Run Juliet benchmarks with JSON output to file
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- juliet \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet \
		$(if $(CWE),--cwe $(CWE),) \
		-o /workspace/tests/benchmarks/sv-benchmarks/juliet-results.json

juliet-categories: ## List supported Juliet CWE categories
	@echo "Supported CWEs (15 categories, ~24,815 tests):"
	@echo ""
	@echo "  Memory Safety (valid-memsafety):"
	@echo "    CWE121  Stack Buffer Overflow"
	@echo "    CWE122  Heap Buffer Overflow"
	@echo "    CWE124  Buffer Underwrite"
	@echo "    CWE126  Buffer Over-read"
	@echo "    CWE127  Buffer Under-read"
	@echo "    CWE401  Memory Leak"
	@echo "    CWE415  Double Free"
	@echo "    CWE416  Use After Free"
	@echo "    CWE476  NULL Pointer Dereference"
	@echo "    CWE590  Free of Non-Heap Variable"
	@echo "    CWE690  NULL Deref from Return"
	@echo "    CWE761  Free Pointer Not at Start"
	@echo "    CWE789  Uncontrolled Memory Allocation"
	@echo ""
	@echo "  Integer Overflow (no-overflow):"
	@echo "    CWE190  Integer Overflow"
	@echo "    CWE191  Integer Underflow"
	@echo ""
	@echo "Usage: make test-juliet CWE=CWE476"

clean-juliet: ## Remove compiled Juliet bitcode
	rm -rf tests/benchmarks/sv-benchmarks/.compiled-juliet

# --- CruxBC Scalability Benchmark (SAF vs SVF) ---
# Source .bc files from ptaben submodule (tests/benchmarks/ptaben/test_cases_bc/crux-bc/)
# Setup: git submodule update --init && make prepare-cruxbc && make test-cruxbc-svf (one-time)
# Daily: make test-cruxbc FILTER=small && make compare-cruxbc

prepare-cruxbc: ## Convert cruxbc .bc → .ll with mem2reg (LLVM 18, one-time)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/prepare-cruxbc.sh

test-cruxbc: ## Run SAF benchmark (use FILTER=small for fast feedback)
	@[ -f tests/benchmarks/cruxbc/saf-results.json ] && \
	  mv tests/benchmarks/cruxbc/saf-results.json \
	     tests/benchmarks/cruxbc/saf-results.prev.json || true
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c '\
	  cargo build --release -p saf-cli && \
	  cargo run --release -p saf-bench -- cruxbc \
	    --compiled-dir tests/benchmarks/cruxbc/.compiled \
	    $(if $(FILTER),--filter $(FILTER)) \
	    -o /workspace/tests/benchmarks/cruxbc/saf-results.json'

test-cruxbc-svf: ## Run SVF on raw bitcode via Docker image (use FILTER=small)
	./scripts/run-svf-cruxbc.sh \
	  $(if $(FILTER),--filter $(FILTER)) \
	  -o tests/benchmarks/cruxbc/svf-results.json

test-cruxbc-svf-mem2reg: ## Run SVF on mem2reg-optimized .ll files (apples-to-apples with SAF)
	./scripts/run-svf-cruxbc.sh \
	  --compiled-dir tests/benchmarks/cruxbc/.compiled \
	  $(if $(FILTER),--filter $(FILTER)) \
	  -o tests/benchmarks/cruxbc/svf-mem2reg-results.json

test-cruxbc-llvm-cg: ## Run LLVM callgraph pass for cross-check (use FILTER=small)
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/run-llvm-cg-cruxbc.sh \
	  --compiled-dir tests/benchmarks/cruxbc/.compiled \
	  $(if $(FILTER),--filter $(FILTER)) \
	  -o /workspace/tests/benchmarks/cruxbc/llvm-cg-results.json

bench-cruxbc-multi: ## Run SAF + SVF N times each, average results (N=5 FILTER=small)
	bash scripts/bench-cruxbc-multi.sh $(or $(N),5) $(FILTER)

compare-cruxbc: ## Show SAF vs SVF vs LLVM comparison + SAF delta
	python3 scripts/compare-cruxbc.py \
	  --saf-results tests/benchmarks/cruxbc/saf-results.json \
	  $(if $(wildcard tests/benchmarks/cruxbc/svf-mem2reg-results.json),--svf-results tests/benchmarks/cruxbc/svf-mem2reg-results.json,$(if $(wildcard tests/benchmarks/cruxbc/svf-results.json),--svf-results tests/benchmarks/cruxbc/svf-results.json)) \
	  $(if $(wildcard tests/benchmarks/cruxbc/llvm-cg-results.json),--llvm-cg-results tests/benchmarks/cruxbc/llvm-cg-results.json) \
	  $(if $(wildcard tests/benchmarks/cruxbc/saf-results.prev.json),--saf-prev tests/benchmarks/cruxbc/saf-results.prev.json)

clean-cruxbc: ## Remove compiled .ll files and all results
	rm -rf tests/benchmarks/cruxbc/.compiled tests/benchmarks/cruxbc/*-results*.json

# --- Verification Suite ---

compile-oracle: ## Compile oracle C programs to LLVM IR
	@echo "Compiling oracle verification programs..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev ./scripts/compile-oracle.sh \
		$(if $(LAYER),--layer $(LAYER),)

verify-oracle: ## Run oracle verification suite (LAYER=pta to filter)
	@echo "Running oracle verification suite..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev cargo run --release -p saf-bench -- oracle \
		--oracle-dir tests/verification/oracle \
		$(if $(LAYER),--layer $(LAYER),)

verify-props: ## Run property tests with 10000 iterations
	@echo "Running property tests (10000 iterations)..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
		"PROPTEST_CASES=10000 cargo nextest run --workspace --exclude saf-python -E 'test(proptest) | test(constraint_extraction)'"

verify-props-quick: ## Run property tests with 256 iterations (fast feedback)
	@echo "Running property tests (256 iterations)..."
	docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
		"cargo nextest run --workspace --exclude saf-python -E 'test(proptest) | test(constraint_extraction)'"

verify-quick: verify-props-quick verify-oracle ## Quick verification (proptest + oracle)

verify: verify-props verify-oracle ## Run ALL verification (proptest + oracle)

clean-oracle: ## Remove compiled oracle LLVM IR
	rm -rf tests/verification/oracle/.compiled

# --- WASM Build ---

wasm: ## Build saf-wasm (release)
	wasm-pack build crates/saf-wasm --target web --release --out-dir ../../packages/shared/src/wasm

wasm-dev: ## Build saf-wasm (debug, faster)
	wasm-pack build crates/saf-wasm --target web --dev --out-dir ../../packages/shared/src/wasm

# --- Playground ---

playground: wasm ## Build playground for production
	npm ci && cd playground && GITHUB_PAGES=1 npm run build

playground-dev: wasm-dev ## Start playground dev server (http://localhost:5173)
	npm install && cd playground && npm run dev

playground-deploy: playground ## Deploy to GitHub Pages
	cd playground && npx gh-pages -d dist

# --- Tutorials ---

tutorials: ## Build tutorials app for production
	npm ci && cp -n playground/public/tree-sitter-llvm.wasm playground/public/web-tree-sitter.wasm tutorials/public/ 2>/dev/null || true
	cd tutorials && GITHUB_PAGES=1 npm run build

tutorials-dev: ## Start tutorials dev server (http://localhost:5174)
	cp -n playground/public/tree-sitter-llvm.wasm playground/public/web-tree-sitter.wasm tutorials/public/ 2>/dev/null || true
	cd tutorials && npm install && npm run dev -- --port 5174

# --- Full Site ---

site: ## Build full site (landing + playground + docs + tutorials) and serve locally
	@echo "Installing all workspace dependencies..."
	npm install
	@echo "Building landing page..."
	cd site && npm run build
	@echo "Building playground..."
	cd playground && npx vite build
	@echo "Building docs..."
	cd docs/book && mdbook build
	@echo "Copying tree-sitter WASM to tutorials..."
	cp -n playground/public/tree-sitter-llvm.wasm playground/public/web-tree-sitter.wasm tutorials/public/ 2>/dev/null || true
	@echo "Building tutorials..."
	cd tutorials && npm run build
	@echo "Assembling site..."
	rm -rf _site
	mkdir -p _site/playground _site/docs _site/tutorials
	cp -r site/dist/* _site/
	cp -r playground/dist/* _site/playground/
	cp -r docs/book/build/* _site/docs/
	cp -r tutorials/dist/* _site/tutorials/
	@echo "Serving at http://localhost:8080"
	npx serve _site -l 8080

site-dev-all: ## Start full site with hot reload (landing + tutorials + docs on :8080)
	bash scripts/dev-site.sh

site-dev: ## Start landing page dev server (http://localhost:5173)
	cd site && npm install && npm run dev

site-clean: ## Remove assembled site directory
	rm -rf _site site/dist docs/book/build tutorials/dist

.DEFAULT_GOAL := help
