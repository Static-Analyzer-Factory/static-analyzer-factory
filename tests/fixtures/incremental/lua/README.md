# Lua Incremental Analysis Fixtures

Pre-compiled LLVM IR from Lua 5.4.7 for incremental analysis E2E tests.

## Files

- `*.ll` — All Lua 5.4.7 source files compiled to LLVM IR
- `lmathlib_v2.ll` — Leaf edit variant (added allocation in `math_abs`)
- `lobject_v2.ll` — Core edit variant (added indirect call in `luaO_str2num`)
- `patches/` — Source patches used to generate v2 variants

## Regenerating

Run inside Docker:

    make compile-lua-fixtures

Or manually:

    docker compose run --rm dev sh -c 'bash scripts/compile-lua-fixtures.sh'

## Pinned Version

Lua 5.4.7 — patches are pinned to this exact version.
