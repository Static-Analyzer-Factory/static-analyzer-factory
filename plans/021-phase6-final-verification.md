# Plan 021: Phase 6 — Final Verification

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8
**Prerequisite:** Plans 016–020 (all tutorials created)

## Goal

Verify all 21 tutorials work end-to-end in Docker, fix any remaining issues, and finalize documentation.

## Tasks

### 1. Run all tutorials in Docker

```bash
make shell
# Then inside Docker:
cd /workspace

# Taint tutorials
for dir in tutorials/taint/*/; do
    echo "=== $dir ==="
    cd "$dir" && python detect.py && cd /workspace
done

# Graph tutorials
for dir in tutorials/graphs/*/; do
    echo "=== $dir ==="
    cd "$dir" && python detect.py && cd /workspace
done

# PTA tutorials
for dir in tutorials/pta/*/; do
    echo "=== $dir ==="
    cd "$dir" && python detect.py && cd /workspace
done

# Integration tutorials
for dir in tutorials/integration/*/; do
    echo "=== $dir ==="
    cd "$dir" && python detect.py && cd /workspace
done
```

### 2. Fix any failures

If any tutorial fails, fix the issue (source code, detect.py, or SAF itself). Document bugs found in `FUTURE.md` if they require deeper framework changes.

### 3. Update category README.md files

Replace placeholder text in `tutorials/graphs/README.md`, `tutorials/pta/README.md`, `tutorials/integration/README.md` with real overviews listing all tutorials in that category (if not already done in earlier phases).

### 4. Update tutorials/SETUP.md

Ensure the tutorial table lists all 21 tutorials across all 4 categories with correct paths.

### 5. Run full test suite

```bash
make test
```

Confirm no regressions in existing Rust and Python tests.

### 6. Update PROGRESS.md

- Set plan 021 status to `done`
- Set all plans 016-021 to `done`
- Update task checklist: T17 → `done`
- Mark E8 epic as complete: `[x] E8: Tutorial Expansion`
- Update "Next Steps" for future work
- Add session log entry

### 7. Update FUTURE.md

Add any new extension points or known limitations discovered during tutorial creation.

## Verification

- All 21 tutorials produce expected output in Docker
- Full test suite passes (no regressions)
- PROGRESS.md and FUTURE.md are up to date
- All tutorial README.md files are complete

## On Completion

E8 is done. Update PROGRESS.md with final status and next potential epics.
