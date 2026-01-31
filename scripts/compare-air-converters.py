#!/usr/bin/env python3
"""
Compare AIR output from the Rust/inkwell frontend vs tree-sitter CST-to-AIR converter.

Usage (run inside Docker via `make shell`):
  python3 scripts/compare-air-converters.py tests/fixtures/llvm/memory_ops.ll
  python3 scripts/compare-air-converters.py tests/fixtures/llvm/globals.ll
  python3 scripts/compare-air-converters.py tests/fixtures/llvm/calls.ll
  python3 scripts/compare-air-converters.py tests/fixtures/llvm/e2e/callback_fn_ptr.ll

This script:
1. Loads the .ll file through the Rust/inkwell LLVM frontend (via saf Python bindings)
2. Serializes the resulting AirModule to JSON
3. Prints a structural summary for manual comparison with the tree-sitter converter

Since the tree-sitter converter runs in the browser, we can't easily run it in Python.
Instead, this produces a reference JSON that can be compared with the tree-sitter output.
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(0)

    ll_file = sys.argv[1]
    if not Path(ll_file).exists():
        print(f"File not found: {ll_file}")
        sys.exit(1)

    try:
        import saf
    except ImportError:
        print("Error: saf module not found. Run this inside Docker: make shell")
        sys.exit(1)

    # Load through inkwell LLVM frontend
    print(f"\n=== Inkwell AIR Analysis: {ll_file} ===\n")
    project = saf.Project.open(ll_file)

    # Get module info
    functions = project.functions()
    print(f"Functions ({len(functions)}):")

    for func_name in functions:
        func = project.function(func_name)
        is_decl = func.is_declaration if hasattr(func, 'is_declaration') else False
        params = func.params if hasattr(func, 'params') else []

        print(f"\n  {func_name} ({'declaration' if is_decl else 'definition'})")
        print(f"    params: {len(params)}")

        if not is_decl and hasattr(func, 'blocks'):
            blocks = func.blocks
            print(f"    blocks: {len(blocks)}")
            for block in blocks:
                label = block.label if hasattr(block, 'label') else '?'
                insts = block.instructions if hasattr(block, 'instructions') else []
                print(f"      {label}: {len(insts)} instructions")
                for inst in insts:
                    op = inst.op if hasattr(inst, 'op') else '?'
                    operands = inst.operands if hasattr(inst, 'operands') else []
                    has_dst = inst.dst is not None if hasattr(inst, 'dst') else False
                    extra = ''
                    if hasattr(inst, 'kind') and inst.kind:
                        extra += f' kind={inst.kind}'
                    if hasattr(inst, 'size_bytes') and inst.size_bytes:
                        extra += f' size_bytes={inst.size_bytes}'
                    if hasattr(inst, 'field_path') and inst.field_path:
                        extra += f' field_path={inst.field_path}'
                    print(f"        {op} operands={len(operands)} dst={'yes' if has_dst else 'no'}{extra}")

    # Export the PropertyGraph for reference
    print("\n\n--- PropertyGraph Exports ---")
    try:
        cfg = project.cfg()
        cg = project.callgraph()
        print(f"CFG nodes: {len(cfg.export()['nodes']) if isinstance(cfg.export(), dict) else '?'}")
        print(f"CG nodes: {len(cg.export()['nodes']) if isinstance(cg.export(), dict) else '?'}")
    except Exception as e:
        print(f"  (export failed: {e})")

    # Dump the raw AIR bundle JSON if requested
    if '--dump' in sys.argv:
        try:
            air_json = project.air_json()
            out_file = ll_file + '.inkwell.air.json'
            with open(out_file, 'w') as f:
                json.dump(json.loads(air_json), f, indent=2)
            print(f"\nDumped AIR JSON to: {out_file}")
        except AttributeError:
            print("\n(air_json() not available in this version)")
    elif '--json' in sys.argv:
        try:
            air_json = project.air_json()
            parsed = json.loads(air_json)
            print(json.dumps(parsed, indent=2))
        except AttributeError:
            print("\n(air_json() not available in this version)")


if __name__ == '__main__':
    main()
