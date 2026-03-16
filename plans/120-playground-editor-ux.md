# Plan 120: Playground Python Editor UX — SAF Autocomplete

**Status:** done
**Created:** 2025-02-18

## Goal

Make the playground's Python analyzer editor more IDE-like by adding SAF API
autocompletion with docstrings/type hints, and verifying that standard
keybindings (Shift+Tab, Cmd+/) work correctly.

## Design

### SAF Completion Source

A single new file `playground/src/editor/saf-completions.ts` containing:

1. **A declarative API schema** describing every public symbol in the `saf`
   Python module (classes, methods, fields, functions). When the Python bridge
   API changes, only this schema needs updating.

2. **A CodeMirror `CompletionSource`** that triggers on `.` after known
   identifiers (`saf`, `result`, graph objects) and on `import saf`.
   Returns completions with:
   - Label (method/field name)
   - Type annotation
   - Signature (for callables)
   - One-line documentation
   - Boost value so SAF completions rank above defaults

### Keybinding Verification

`basicSetup` bundles most standard keybindings. Verify and fix if needed:
- `indentWithTab` for Tab/Shift+Tab (may need explicit addition)
- `toggleComment` via Cmd+/ (bundled in `basicSetup`)

### Changes

| File | Change |
|------|--------|
| `playground/src/editor/saf-completions.ts` | **New** — API schema + completion source |
| `playground/src/components/AnalyzerPanel.tsx` | Import + wire completion extension, fix keybindings |
| `playground/package.json` | Add `@codemirror/autocomplete` + `@codemirror/language` if not transitive |

### Out of Scope

- Python builtins/keywords (already handled by `@codemirror/lang-python`)
- Vim/Emacs keybinding modes
- Minimap, code folding
- Runtime type inference via Pyodide LSP

## Steps

1. Verify `@codemirror/autocomplete` availability (may be transitive via `basicSetup`)
2. Create `saf-completions.ts` with API schema + completion source
3. Wire into AnalyzerPanel editor extensions
4. Add `indentWithTab` keymap explicitly if not already active
5. Test in dev server
6. Build production bundle
