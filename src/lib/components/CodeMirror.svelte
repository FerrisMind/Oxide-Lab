<script lang="ts">

  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
  import { EditorState, StateEffect, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { foldGutter, bracketMatching, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
  import { lineNumbers, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor } from '@codemirror/view';
  // Theme kept as optional lazy import; loaded on demand
  let oneDark: any = $state(null);

  interface Props {
    code?: string;
    language?: string;
    readonly?: boolean;
    theme?: 'light' | 'dark' | 'auto';
    showLineNumbers?: boolean;
    wrap?: boolean;
    onChange?: (detail: { code: string }) => void;
  }

  let {
    code = '',
    language = '',
    readonly = true,
    theme = 'auto',
    showLineNumbers = false,
    wrap = true,
    onChange
  }: Props = $props();
  
  let container: HTMLElement | undefined = $state();
  let editorView: EditorView | null = $state(null);

  // Compartments for dynamic reconfiguration
  const compTheme = new Compartment();
  const compDark = new Compartment();
  const compLang = new Compartment();
  const compReadonly = new Compartment();
  const compLineNumbers = new Compartment();
  const compWrapping = new Compartment();

  // Lazy language loaders to enable tree-shaking and reduce initial bundle
  async function loadLanguageExtension(langName: string) {
    const lang = (langName || '').toLowerCase();
    try {
      switch (lang) {
        case 'javascript':
        case 'js': {
          const m = await import('@codemirror/lang-javascript');
          return m.javascript();
        }
        case 'typescript':
        case 'ts': {
          const m = await import('@codemirror/lang-javascript');
          return m.javascript({ typescript: true });
        }
        case 'jsx': {
          const m = await import('@codemirror/lang-javascript');
          return m.javascript({ jsx: true });
        }
        case 'tsx': {
          const m = await import('@codemirror/lang-javascript');
          return m.javascript({ typescript: true, jsx: true });
        }
        case 'python':
        case 'py': {
          const m = await import('@codemirror/lang-python');
          return m.python();
        }
        case 'html': {
          const m = await import('@codemirror/lang-html');
          return m.html();
        }
        case 'css': {
          const m = await import('@codemirror/lang-css');
          return m.css();
        }
        case 'json': {
          const m = await import('@codemirror/lang-json');
          return m.json();
        }
        case 'xml': {
          const m = await import('@codemirror/lang-xml');
          return m.xml();
        }
        case 'sql': {
          const m = await import('@codemirror/lang-sql');
          return m.sql();
        }
        default:
          return [];
      }
    } catch (e) {
      console.warn('Failed to lazy-load language:', lang, e);
      return [];
    }
  }

  // Theme detection
  function getTheme(): 'light' | 'dark' {
    if (theme === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }

  async function applyLanguage() {
    if (!editorView) return;
    const ext = await loadLanguageExtension(language);
    editorView.dispatch({ effects: compLang.reconfigure(ext) });
  }

  async function createEditor() {
    if (!container) return;

    const extensions = [
      highlightActiveLineGutter(),
      highlightSpecialChars(),
      history(),
      foldGutter(),
      drawSelection(),
      dropCursor(),
      EditorState.allowMultipleSelections.of(true),
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
      compTheme.of(EditorView.theme({
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
      })),
      compWrapping.of(wrap ? EditorView.lineWrapping : []),
      compLang.of([]),
      compReadonly.of(readonly ? EditorState.readOnly.of(true) : []),
      compLineNumbers.of(
        showLineNumbers
          ? lineNumbers()
          : EditorView.theme({ '.cm-gutters': { display: 'none' } })
      ),
    ];

    // Add theme (dark) if applicable
    const currentTheme = getTheme();
    if (currentTheme === 'dark') {
      if (!oneDark) {
        try { oneDark = (await import('@codemirror/theme-one-dark')).oneDark; } catch {}
      }
      extensions.push(compDark.of(oneDark || []));
    } else {
      extensions.push(compDark.of([]));
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
          onChange?.({ code: newCode });
        }
      });
      editorView.dispatch({
        effects: StateEffect.reconfigure.of([...extensions, updateListener])
      });
    }

    // Apply language asynchronously after view is ready
    applyLanguage();
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
  $effect(() => {
    if (editorView && code !== undefined) {
      updateEditor();
    }
  });

  // Dynamic theme changes via Compartment
  $effect(() => {
    if (editorView) {
      const currentTheme = getTheme();
      (async () => {
        if (currentTheme === 'dark') {
          if (!oneDark) {
            try { oneDark = (await import('@codemirror/theme-one-dark')).oneDark; } catch {}
          }
          if (oneDark) editorView && editorView.dispatch({ effects: compDark.reconfigure(oneDark) });
        } else {
          editorView && editorView.dispatch({ effects: compDark.reconfigure([]) });
        }
      })();
    }
  });

  // Dynamic language change
  $effect(() => {
    if (editorView && language !== undefined) {
      applyLanguage();
    }
  });

  // Dynamic toggles
  $effect(() => {
    if (editorView) {
      editorView.dispatch({ effects: [
        compReadonly.reconfigure(readonly ? EditorState.readOnly.of(true) : []),
        compLineNumbers.reconfigure(
          showLineNumbers ? lineNumbers() : EditorView.theme({ '.cm-gutters': { display: 'none' } })
        ),
        compWrapping.reconfigure(wrap ? EditorView.lineWrapping : [])
      ]});
    }
  });

  // Listen for theme changes
  let mediaQuery: MediaQueryList;
  onMount(() => {
    if (theme === 'auto') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => {
        if (editorView) {
          const isDark = mediaQuery.matches;
          (async () => {
            if (isDark) {
              if (!oneDark) { try { oneDark = (await import('@codemirror/theme-one-dark')).oneDark; } catch {} }
              editorView && editorView.dispatch({ effects: compDark.reconfigure(oneDark || []) });
            } else {
              editorView && editorView.dispatch({ effects: compDark.reconfigure([]) });
            }
          })();
        }
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
