<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorView, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
  import { EditorState, StateEffect } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { foldGutter, indentOnInput, indentUnit, bracketMatching, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
  import { lineNumbers, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor } from '@codemirror/view';
  import { javascript } from '@codemirror/lang-javascript';
  import { python } from '@codemirror/lang-python';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { json } from '@codemirror/lang-json';
  import { xml } from '@codemirror/lang-xml';
  import { sql } from '@codemirror/lang-sql';
  import { oneDark } from '@codemirror/theme-one-dark';

  export let code: string = '';
  export let language: string = '';
  export let readonly: boolean = true;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  export let showLineNumbers: boolean = true;
  export const wrap: boolean = true;

  const dispatch = createEventDispatcher();
  
  let container: HTMLElement;
  let editorView: EditorView | null = null;

  // Language mappings
  const languageExtensions: Record<string, () => any> = {
    'javascript': () => javascript(),
    'js': () => javascript(),
    'typescript': () => javascript({ typescript: true }),
    'ts': () => javascript({ typescript: true }),
    'python': () => python(),
    'py': () => python(),
    'html': () => html(),
    'css': () => css(),
    'json': () => json(),
    'xml': () => xml(),
    'sql': () => sql(),
    'jsx': () => javascript({ jsx: true }),
    'tsx': () => javascript({ typescript: true, jsx: true }),
  };

  // Theme detection
  function getTheme(): 'light' | 'dark' {
    if (theme === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }

  function getLanguageExtension() {
    const lang = language.toLowerCase();
    if (languageExtensions[lang]) {
      try {
        return languageExtensions[lang]();
      } catch (error) {
        console.warn('Failed to load language extension for:', lang, error);
        return null;
      }
    }
    return null;
  }

  function createEditor() {
    if (!container) return;

    const extensions = [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightSpecialChars(),
      history(),
      foldGutter(),
      drawSelection(),
      dropCursor(),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      rectangularSelection(),
      crosshairCursor(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
      ]),
      EditorView.theme({
        '&': {
          fontSize: '14px',
          border: '1px solid var(--border-color)',
          borderRadius: '8px',
          overflow: 'hidden',
        },
        '.cm-content': {
          padding: '12px',
          minHeight: '20px',
        },
        '.cm-editor': {
          borderRadius: '8px',
        },
        '.cm-focused': {
          outline: 'none',
        },
        '.cm-scroller': {
          lineHeight: '1.5',
        }
      }),
      EditorView.lineWrapping,
    ];

    // Add language support
    const langExt = getLanguageExtension();
    if (langExt) {
      extensions.push(langExt);
    }

    // Add theme
    const currentTheme = getTheme();
    if (currentTheme === 'dark') {
      extensions.push(oneDark);
    }

    // Configure readonly mode
    if (readonly) {
      extensions.push(EditorState.readOnly.of(true));
    }

    // Line numbers
    if (!showLineNumbers) {
      extensions.push(EditorView.theme({
        '.cm-lineNumbers': { display: 'none' },
        '.cm-gutters': { display: 'none' }
      }));
    }

    const state = EditorState.create({
      doc: code,
      extensions,
    });

    editorView = new EditorView({
      state,
      parent: container,
    });

    // Listen for changes if not readonly
    if (!readonly) {
      const updateListener = EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const newCode = update.state.doc.toString();
          dispatch('change', { code: newCode });
        }
      });
      editorView.dispatch({
        effects: StateEffect.reconfigure.of([...extensions, updateListener])
      });
    }
  }

  function updateEditor() {
    if (!editorView) return;

    const currentCode = editorView.state.doc.toString();
    if (currentCode !== code) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: currentCode.length,
          insert: code
        }
      });
    }
  }

  function destroyEditor() {
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
  }

  onMount(() => {
    createEditor();
  });

  onDestroy(() => {
    destroyEditor();
  });

  // React to prop changes
  $: if (editorView && code !== undefined) {
    updateEditor();
  }

  $: if (container && (language || theme)) {
    destroyEditor();
    createEditor();
  }

  // Listen for theme changes
  let mediaQuery: MediaQueryList;
  onMount(() => {
    if (theme === 'auto') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => {
        destroyEditor();
        createEditor();
      };
      mediaQuery.addEventListener('change', handleChange);
      
      return () => {
        mediaQuery.removeEventListener('change', handleChange);
      };
    }
  });
</script>

<div bind:this={container} class="codemirror-wrapper"></div>

<style>
  .codemirror-wrapper {
    width: 100%;
    margin: 8px 0;
  }

  :global(.cm-editor) {
    background: var(--code-bg) !important;
  }

  :global(.cm-content) {
    color: var(--code-fg) !important;
  }

  :global(.cm-gutters) {
    background: var(--panel-alt-bg) !important;
    border-right: 1px solid var(--border-color) !important;
  }

  :global(.cm-lineNumbers .cm-gutterElement) {
    color: var(--muted) !important;
  }

  /* Light theme adjustments */
  :global(.cm-editor:not(.cm-dark)) {
    background: var(--code-bg) !important;
    color: var(--code-fg) !important;
  }

  /* Dark theme adjustments */
  @media (prefers-color-scheme: dark) {
    :global(.cm-editor) {
      background: var(--code-bg) !important;
    }
  }
</style>