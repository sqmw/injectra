import { EditorState, Compartment } from '@codemirror/state';
import { EditorView, lineNumbers, highlightActiveLine, keymap, drawSelection } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language';
import { javascript } from '@codemirror/lang-javascript';
import { css } from '@codemirror/lang-css';
import { html } from '@codemirror/lang-html';

export interface SourceEditor { load(path: string, content: string, editable: boolean): void; focusLine(line: number): void; destroy(): void }
export function createEditor(parent: HTMLElement, change: (content: string) => void, save: () => void, run: () => void): SourceEditor {
  const language = new Compartment(), editable = new Compartment();
  let loading = false;
  const view = new EditorView({ parent, state: EditorState.create({ doc: '', extensions: [
    lineNumbers(), history(), drawSelection(), bracketMatching(), highlightActiveLine(), syntaxHighlighting(defaultHighlightStyle),
    keymap.of([{ key: 'Mod-s', run: () => { save(); return true; } }, { key: 'Mod-Enter', run: () => { run(); return true; } }, ...defaultKeymap, ...historyKeymap]),
    language.of(javascript()), editable.of([EditorState.readOnly.of(true), EditorView.editable.of(false)]),
    EditorView.theme({ '&': { height: '100%', fontSize: '13px' }, '.cm-scroller': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace', lineHeight: '1.65' }, '.cm-content': { padding: '18px 0' }, '.cm-gutters': { background: '#f8f9fc', color: '#9299ae', borderRight: '1px solid #edf0f6' }, '.cm-activeLine': { background: '#f4f1ff' }, '.cm-activeLineGutter': { background: '#eee9ff' } }),
    EditorView.updateListener.of(update => { if (update.docChanged && !loading) change(update.state.doc.toString()); }),
  ] }) });
  return {
    load(path, content, writable) {
      loading = true;
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: content }, selection: { anchor: 0 }, effects: [
        language.reconfigure(/\.css$/i.test(path) ? css() : /\.html?$/i.test(path) ? html() : javascript({ typescript: /\.tsx?$/i.test(path) })),
        editable.reconfigure([EditorState.readOnly.of(!writable), EditorView.editable.of(writable)]),
      ] });
      loading = false;
    },
    focusLine(line) { const position = view.state.doc.line(Math.max(1, Math.min(view.state.doc.lines, line))).from; view.dispatch({ selection: { anchor: position }, effects: EditorView.scrollIntoView(position, { y: 'center' }) }); view.focus(); },
    destroy() { view.destroy(); },
  };
}
