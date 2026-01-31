#!/usr/bin/env python3
"""
Dump AIR JSON from the Rust/inkwell LLVM frontend for comparison with tree-sitter converter.

Usage (inside Docker via `make shell`):
  python3 scripts/dump-air-json.py tests/fixtures/llvm/memory_ops.ll
  python3 scripts/dump-air-json.py tests/fixtures/llvm/globals.ll
  python3 scripts/dump-air-json.py tests/fixtures/llvm/calls.ll
  python3 scripts/dump-air-json.py tests/fixtures/llvm/e2e/callback_fn_ptr.ll

Outputs: <stem>.inkwell.air.json

Compare with tree-sitter output by running the playground converter in the browser
or using the vitest tests.
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(0)

    ll_files = [f for f in sys.argv[1:] if not f.startswith('--')]

    try:
        import saf
    except ImportError:
        print("Error: saf module not found. Run inside Docker: make shell")
        sys.exit(1)

    for ll_file in ll_files:
        if not Path(ll_file).exists():
            print(f"File not found: {ll_file}")
            continue

        project = saf.Project.open(ll_file)
        module = project.module

        summary = {
            "source": ll_file,
            "converter": "inkwell",
            "functions": [],
            "globals": [],
        }

        # Functions
        for func in module.functions():
            f = {
                "name": func.name,
                "is_declaration": func.is_declaration,
                "param_count": func.param_count,
                "block_count": func.block_count,
                "param_names": func.param_names(),
            }
            summary["functions"].append(f)

        # Globals
        for glob in module.globals():
            g = {
                "name": glob.name,
                "is_constant": glob.is_constant,
            }
            summary["globals"].append(g)

        out_file = Path(ll_file).stem + '.inkwell-summary.json'
        with open(out_file, 'w') as f:
            json.dump(summary, f, indent=2)

        print(f"\n=== {ll_file} ===")
        print(f"Functions: {len(summary['functions'])}")
        for func in summary['functions']:
            decl_str = " (declaration)" if func['is_declaration'] else ""
            print(f"  {func['name']}: {func['param_count']} params, {func['block_count']} blocks{decl_str}")
        print(f"Globals: {len(summary['globals'])}")
        for glob in summary['globals']:
            const_str = " (constant)" if glob['is_constant'] else ""
            print(f"  {glob['name']}{const_str}")
        print(f"Saved: {out_file}")


if __name__ == '__main__':
    main()
