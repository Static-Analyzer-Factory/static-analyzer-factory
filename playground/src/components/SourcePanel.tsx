import { useEffect, useRef } from 'react';
import { EditorState, StateEffect, StateField } from '@codemirror/state';
import { Decoration, type DecorationSet, EditorView, keymap } from '@codemirror/view';
import { basicSetup } from 'codemirror';
import { cpp } from '@codemirror/lang-cpp';
import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';

// State effect to set highlighted line range [startLine, endLine] (1-based, inclusive)
const setHighlight = StateEffect.define<[number, number] | null>();

const highlightField = StateField.define<DecorationSet>({
  create() {
    return Decoration.none;
  },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(setHighlight)) {
        if (!e.value) return Decoration.none;
        const [start, end] = e.value;
        const decorations: import('@codemirror/state').Range<Decoration>[] = [];
        const lineDeco = Decoration.line({ class: 'cm-highlight-line' });
        for (let line = start; line <= end; line++) {
          if (line >= 1 && line <= tr.state.doc.lines) {
            decorations.push(lineDeco.range(tr.state.doc.line(line).from));
          }
        }
        return Decoration.set(decorations);
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const highlightTheme = EditorView.baseTheme({
  '.cm-highlight-line': {
    backgroundColor: 'rgba(61, 155, 143, 0.15)',
    borderLeft: '3px solid #3d9b8f',
  },
});

interface SourcePanelProps {
  inputMode: 'c' | 'llvm';
  sourceCode: string;
  onModeChange: (mode: 'c' | 'llvm') => void;
  onSourceChange: (code: string) => void;
  highlightedLines?: [number, number] | null;
}

export function SourcePanel({
  inputMode,
  sourceCode,
  onModeChange: _onModeChange,
  onSourceChange,
  highlightedLines,
}: SourcePanelProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        onSourceChange(update.state.doc.toString());
      }
    });

    const state = EditorState.create({
      doc: sourceCode,
      extensions: [
        basicSetup,
        cpp(),
        morandiTheme,
        updateListener,
        highlightField,
        highlightTheme,
        keymap.of([]),
        EditorView.theme({
          '&': { height: '100%' },
          '.cm-scroller': { overflow: 'auto' },
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // Intentionally only run on mount and mode change — sourceCode changes
    // are handled by the editor internally via updateListener.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inputMode]);

  // Sync highlighted lines from graph clicks into editor
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({ effects: setHighlight.of(highlightedLines ?? null) });
    // Scroll to first highlighted line
    if (highlightedLines) {
      const lineInfo = view.state.doc.line(
        Math.min(highlightedLines[0], view.state.doc.lines),
      );
      view.dispatch({
        effects: EditorView.scrollIntoView(lineInfo.from, { y: 'center' }),
      });
    }
  }, [highlightedLines]);

  // Sync external source changes (e.g., example selection) into editor
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentDoc = view.state.doc.toString();
    if (currentDoc !== sourceCode) {
      view.dispatch({
        changes: { from: 0, to: currentDoc.length, insert: sourceCode },
      });
    }
  }, [sourceCode]);

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>Source (C/C++)</h2>
      </div>
      <div className="panel-content editor-container" ref={editorRef} />
    </div>
  );
}
