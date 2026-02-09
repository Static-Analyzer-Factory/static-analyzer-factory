import { useEffect, useRef } from 'react';
import { EditorView } from '@codemirror/view';
import { EditorState } from '@codemirror/state';
import { cpp } from '@codemirror/lang-cpp';
import { python } from '@codemirror/lang-python';
import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';
import {
  Decoration,
  ViewPlugin,
} from '@codemirror/view';
import type { DecorationSet, ViewUpdate } from '@codemirror/view';

interface CodeBlockProps {
  code: string;
  language: 'c' | 'python' | 'bash';
  highlightLines?: number[];
}

const highlightMark = Decoration.line({ class: 'cm-highlighted-line' });

function makeHighlightPlugin(lines: number[]) {
  const lineSet = new Set(lines);
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = this.build(view);
      }
      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = this.build(update.view);
        }
      }
      build(view: EditorView): DecorationSet {
        const ranges: { from: number; decoration: ReturnType<typeof Decoration.line> }[] = [];
        for (let i = 1; i <= view.state.doc.lines; i++) {
          if (lineSet.has(i)) {
            const line = view.state.doc.line(i);
            ranges.push({ from: line.from, decoration: highlightMark });
          }
        }
        // Sort by from position and build DecorationSet
        ranges.sort((a, b) => a.from - b.from);
        return Decoration.set(ranges.map(r => r.decoration.range(r.from)));
      }
    },
    { decorations: (v) => v.decorations },
  );
}

function getLanguageExtension(lang: 'c' | 'python' | 'bash') {
  switch (lang) {
    case 'c':
      return cpp();
    case 'python':
      return python();
    case 'bash':
      // No dedicated bash extension; use plain text
      return [];
  }
}

export default function CodeBlock({ code, language, highlightLines }: CodeBlockProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    if (viewRef.current) {
      viewRef.current.destroy();
      viewRef.current = null;
    }

    const extensions = [
      EditorView.editable.of(false),
      EditorState.readOnly.of(true),
      morandiTheme,
      getLanguageExtension(language),
    ];

    if (highlightLines && highlightLines.length > 0) {
      extensions.push(makeHighlightPlugin(highlightLines));
    }

    const state = EditorState.create({
      doc: code,
      extensions,
    });

    const view = new EditorView({
      state,
      parent: containerRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, [code, language, highlightLines]);

  return <div ref={containerRef} className="code-block" />;
}
