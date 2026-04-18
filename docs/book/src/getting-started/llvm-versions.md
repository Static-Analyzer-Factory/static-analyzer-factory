# LLVM Version Support

SAF ships two Docker image variants, one per supported LLVM version:

| Tag              | LLVM | Ubuntu base | Status  | Use when                                              |
| ---------------- | ---- | ----------- | ------- | ----------------------------------------------------- |
| `saf:llvm18`     | 18.1 | 24.04 LTS   | stable  | default — analyze IR emitted by clang-18 (or older)   |
| `saf:latest`     | 18.1 | 24.04 LTS   | stable  | alias for `saf:llvm18`                                |
| `saf:llvm22`     | 22.1 | 24.04 LTS   | opt-in  | analyze IR emitted by clang-22 / modern C++26 sources |

Each image contains **exactly one** LLVM toolchain. Images do not share caches
or layers; pick the tag that matches the clang you'll compile your sources
with.

## Which tag should I pick?

**Pick the LLVM whose major version matches the clang you'll compile with.**

LLVM bitcode is *backward-compatible* but not forward-compatible — an LLVM-18
library cannot parse IR produced by clang-22. If your source requires clang-22
(e.g., C++26 reflection, new attributes, recent ptrauth intrinsics), use the
`saf:llvm22` image so the analyzer and the compiler agree on the IR format.

For everyday analysis of mainstream C / C++ code, `saf:llvm18` is the
recommended default. It's the version our benchmark corpora (PTABen, NIST
Juliet, SV-COMP, CruxBC) are compiled with, and it's what every stable SAF
release is validated against.

## Switching between tags

```bash
# Pull and run the LLVM 18 image (default)
docker pull ghcr.io/thepatrickstar/saf:llvm18
docker run --rm -v "$PWD":/work ghcr.io/thepatrickstar/saf:llvm18 analyze /work/program.ll

# Pull and run the LLVM 22 image
docker pull ghcr.io/thepatrickstar/saf:llvm22
docker run --rm -v "$PWD":/work ghcr.io/thepatrickstar/saf:llvm22 analyze /work/program.ll
```

From a local checkout:

```bash
make shell           # dev shell backed by LLVM 18
make shell-llvm22    # dev shell backed by LLVM 22
make test            # tests against LLVM 18
make test-llvm22     # tests against LLVM 22
make build           # build runtime image (LLVM 18)
make build-llvm22    # build runtime image (LLVM 22)
```

## Support policy

- SAF actively supports **two LLVM versions** at a time: the current stable
  default and one opt-in newer version.
- Every release is tested against both tags in CI.
- Older LLVM versions (16, 17, 19, 20, 21) are **not** actively supported.
  Adding support is mostly mechanical (see `crates/saf-frontends/src/llvm/`),
  but we don't carry the CI or maintenance burden by default.
- The oldest supported LLVM is advanced roughly once a year, in sync with
  Ubuntu LTS release cadence and user demand.

## Forward-incompatibility caveats

When the `saf:llvm18` analyzer is handed IR emitted by clang-19 or newer, it
will fail at parse time with a message like:

```
LLVM parse error: Invalid record at offset N
```

or warnings about unrecognized attributes / intrinsics. These are not SAF
bugs — they indicate the IR uses features introduced after LLVM 18. Either
switch to the `saf:llvm22` image or have your build emit bitcode targeting an
older LLVM (`clang -emit-llvm -Xclang -target-version=18` and similar
workarounds).

## Adding support for a new LLVM version

Adding an `llvm-N` feature flag is a roughly three-step process:

1. Add a cargo feature to `crates/saf-frontends/Cargo.toml` pointing to the
   matching `inkwell/llvmN-M` feature.
2. Create `crates/saf-frontends/src/llvm/llvmN.rs` with a one-line
   `impl_llvm_adapter!(LlvmNAdapter, "N.M");` invocation, and wire it into
   `mod.rs` / `adapter.rs` beside the existing versions.
3. Add a `dev-llvmN` / `test-llvmN` service to `docker-compose.yml` and a
   Make convenience target.

Audit `crates/saf-frontends/src/llvm/{mapping, intrinsics, debug_info}.rs`
for any IR constructs that have changed meaning across the gap — new
attributes, intrinsics, or the LLVM 19 debug-record format transition are
typical sources of work.
