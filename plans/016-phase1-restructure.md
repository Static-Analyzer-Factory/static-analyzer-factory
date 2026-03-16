# Plan 016: Phase 1 — Restructure Tutorial Directories

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8

## Goal

Move existing 5 taint tutorials into `tutorials/taint/` subdirectory, create empty category directories for graphs/pta/integration, and update SETUP.md.

## Tasks

### 1. Create category directories

```
mkdir -p tutorials/taint
mkdir -p tutorials/graphs
mkdir -p tutorials/pta
mkdir -p tutorials/integration
```

### 2. Move existing tutorials

| From | To |
|------|-----|
| `tutorials/01-taint-detection/` | `tutorials/taint/01-command-injection/` |
| `tutorials/02-format-string/` | `tutorials/taint/02-format-string/` |
| `tutorials/03-use-after-free/` | `tutorials/taint/03-use-after-free/` |
| `tutorials/04-buffer-overflow/` | `tutorials/taint/04-buffer-overflow/` |
| `tutorials/05-unsafe-rust/` | `tutorials/taint/05-unsafe-rust/` |

Note: `01-taint-detection` is renamed to `01-command-injection` (more specific).

### 3. Delete empty old directories

Remove `tutorials/01-*` through `tutorials/05-*` after moving.

### 4. Create category README.md files

- `tutorials/taint/README.md` — overview of taint tutorials, what taint_flow() does, list of 5 tutorials
- `tutorials/graphs/README.md` — placeholder: "Graph exploration tutorials coming soon"
- `tutorials/pta/README.md` — placeholder: "Pointer analysis tutorials coming soon"
- `tutorials/integration/README.md` — placeholder: "Integration tutorials coming soon"

### 5. Update tutorials/SETUP.md

Replace the flat tutorial table with a category-based table:

```
| Category | # | Tutorial | Language | API |
|----------|---|----------|----------|-----|
| taint | 01 | Command Injection | C | taint_flow() |
| taint | 02 | Format String | C | taint_flow() |
| ... | ... | ... | ... | ... |
```

### 6. Fix internal cross-references

Update any README.md links that reference old paths (e.g., `../03-use-after-free/` → within taint category).

## Verification

- All 5 moved tutorials still work: `cd tutorials/taint/01-command-injection && python detect.py` (in Docker)
- No broken symlinks or dangling references
- `tutorials/SETUP.md` accurately reflects new structure

## On Completion

Update `PROGRESS.md`:
- Set plan 016 status to `done`
- Update task checklist: T1 → `done`
- Update "Next Steps" to point to plan 017
