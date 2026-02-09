import { useEffect, useRef } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { basicSetup } from 'codemirror';
import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';

interface CompiledIRPanelProps {
  compiledIR: string | null;
}

export function CompiledIRPanel({ compiledIR }: CompiledIRPanelProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    const state = EditorState.create({
      doc: compiledIR || '',
      extensions: [
        basicSetup,
        morandiTheme,
        EditorState.readOnly.of(true),
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
  }, [compiledIR]);

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
      <div className="panel-header">
        <h2>Compiled IR</h2>
      </div>
      <div className="panel-content editor-container" ref={editorRef}>
        {!compiledIR && (
          <div className="placeholder">
            <p>Click "Analyze" to compile C source to LLVM IR via Compiler Explorer</p>
          </div>
        )}
      </div>
    </div>
  );
}
