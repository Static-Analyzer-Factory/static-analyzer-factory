/**
 * SAF CodeMirror 6 theme — Fresh & Friendly (warm light).
 *
 * Editor chrome reads from CSS variables where possible so it updates
 * automatically when theme.css changes. Syntax colors are hard-coded
 * (CodeMirror highlight styles don't support var()).
 */
import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import type { Extension } from '@codemirror/state';

/** Editor chrome (backgrounds, gutters, cursor, selection). */
const morandiEditorTheme = EditorView.theme(
  {
    '&': {
      backgroundColor: 'var(--color-surface-editor, #f5f3f0)',
      color: 'var(--color-text, #2c2c2e)',
    },
    '.cm-content': {
      caretColor: 'var(--color-text, #2c2c2e)',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: 'var(--color-text, #2c2c2e)',
    },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
      backgroundColor: 'rgba(61, 155, 143, 0.16)',
    },
    '.cm-panels': {
      backgroundColor: 'var(--color-surface, #f1eeeb)',
      color: 'var(--color-text, #2c2c2e)',
    },
    '.cm-panels.cm-panels-top': {
      borderBottom: '1px solid var(--color-border, #dbd6d0)',
    },
    '.cm-panels.cm-panels-bottom': {
      borderTop: '1px solid var(--color-border, #dbd6d0)',
    },
    '.cm-searchMatch': {
      backgroundColor: 'rgba(196, 154, 60, 0.25)',
      outline: '1px solid rgba(196, 154, 60, 0.4)',
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
      backgroundColor: 'rgba(61, 155, 143, 0.25)',
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(61, 155, 143, 0.06)',
    },
    '.cm-selectionMatch': {
      backgroundColor: 'rgba(61, 155, 143, 0.12)',
    },
    '&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket': {
      backgroundColor: 'rgba(61, 155, 143, 0.2)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--color-surface, #f1eeeb)',
      color: 'var(--color-text-tertiary, #9b9ba0)',
      borderRight: '1px solid var(--color-border-subtle, #eae6e2)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'rgba(61, 155, 143, 0.08)',
    },
    '.cm-foldPlaceholder': {
      backgroundColor: 'var(--color-surface-raised, #e7e3df)',
      border: 'none',
      color: 'var(--color-text-secondary, #6b6b70)',
    },
    '.cm-tooltip': {
      border: '1px solid var(--color-border, #dbd6d0)',
      backgroundColor: 'var(--color-surface, #f1eeeb)',
      color: 'var(--color-text, #2c2c2e)',
    },
    '.cm-tooltip .cm-tooltip-arrow:before': {
      borderTopColor: 'var(--color-border, #dbd6d0)',
      borderBottomColor: 'var(--color-border, #dbd6d0)',
    },
    '.cm-tooltip .cm-tooltip-arrow:after': {
      borderTopColor: 'var(--color-surface, #f1eeeb)',
      borderBottomColor: 'var(--color-surface, #f1eeeb)',
    },
    '.cm-tooltip-autocomplete': {
      '& > ul > li[aria-selected]': {
        backgroundColor: 'var(--color-accent-subtle, rgba(61, 155, 143, 0.10))',
        color: 'var(--color-text, #2c2c2e)',
      },
    },
  },
  { dark: false },
);

/** Syntax highlighting — fresh, readable tones. */
const morandiHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: '#8b5dc8' },
  { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: '#2c2c2e' },
  { tag: [tags.function(tags.variableName), tags.labelName], color: '#3d7e9b' },
  { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: '#b07040' },
  { tag: [tags.definition(tags.name), tags.separator], color: '#2c2c2e' },
  {
    tag: [
      tags.typeName, tags.className, tags.number, tags.changed, tags.annotation,
      tags.modifier, tags.self, tags.namespace,
    ],
    color: '#2e8a7f',
  },
  {
    tag: [
      tags.operator, tags.operatorKeyword, tags.url, tags.escape,
      tags.regexp, tags.special(tags.string),
    ],
    color: '#6b6b70',
  },
  { tag: [tags.meta, tags.comment], color: '#9b9ba0' },
  { tag: tags.strong, fontWeight: 'bold' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.strikethrough, textDecoration: 'line-through' },
  { tag: tags.link, color: '#5088b5', textDecoration: 'underline' },
  { tag: tags.heading, fontWeight: 'bold', color: '#8b5dc8' },
  { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: '#2e8a7f' },
  { tag: [tags.processingInstruction, tags.string, tags.inserted], color: '#548a4e' },
  { tag: tags.invalid, color: '#c75050' },
  { tag: [tags.propertyName], color: '#b07040' },
]);

/** Complete theme extension — drop-in replacement for oneDark. */
export const morandiTheme: Extension = [
  morandiEditorTheme,
  syntaxHighlighting(morandiHighlightStyle),
];
