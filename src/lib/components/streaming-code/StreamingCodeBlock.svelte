<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorView, ViewPlugin, Decoration, WidgetType, ViewUpdate, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
  import { EditorState, StateEffect } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { foldGutter, bracketMatching, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
  import { lineNumbers, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor } from '@codemirror/view';
  import { javascript } from '@codemirror/lang-javascript';
  import { python } from '@codemirror/lang-python';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { json } from '@codemirror/lang-json';
  import { xml } from '@codemirror/lang-xml';
  import { sql } from '@codemirror/lang-sql';
  import { oneDark } from '@codemirror/theme-one-dark';
  // CodeMirror widget utilities for in-editor progress UI

  export let code: string = '';
  export let language: string = '';
  export let isStreaming: boolean = false;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  export let showLineNumbers: boolean = true;
  export let readonly: boolean = true;

  const dispatch = createEventDispatcher();
  
  let container: HTMLElement;
  let compactContainer: HTMLElement | null = null;
  let editorView: EditorView | null = null;
  let compactView: EditorView | null = null;
  let isExpanded: boolean = false;
  let lastCodeLength: number = 0;
  let streamingTimeout: number | null = null;

  // Top-level widget class (moved out of factory to avoid Svelte nested-class warning)
  class ProgressWidget extends WidgetType {
    streaming: boolean;
    constructor(streaming: boolean) { super(); this.streaming = streaming; }
    toDOM() {
      const root = document.createElement('div');
      root.className = 'cm-progress-widget';

      const content = document.createElement('div');
      content.className = 'cm-progress-content';

      const langSpan = document.createElement('span');
      langSpan.className = 'cm-progress-lang';
      langSpan.textContent = language || 'text';

      const statusSpan = document.createElement('span');
      statusSpan.className = 'cm-progress-status';
      statusSpan.textContent = this.streaming ? 'Выполняется' : 'Готово';

      content.appendChild(langSpan);
      content.appendChild(statusSpan);

      if (this.streaming) {
        const bar = document.createElement('div');
        bar.className = 'cm-progress-bar';
        const line = document.createElement('div');
        line.className = 'cm-progress-line';
        bar.appendChild(line);
        content.appendChild(bar);
      }

      root.appendChild(content);
      return root;
    }
    ignoreEvent() { return false; }
  }

  // Create a CodeMirror widget extension that renders a progress bar at the
  // top of the editor. We recreate the editor when `isStreaming` changes to
  // apply the appropriate decoration.
  function createProgressExtension(streaming: boolean) {
    const plugin = ViewPlugin.fromClass(
      class {
        view: EditorView;
        decorations: any;
        constructor(view: EditorView) {
          this.view = view;
          this.decorations = Decoration.set([
            Decoration.widget({ widget: new ProgressWidget(streaming), side: -1 }).range(0),
          ]);
        }
        update(_update: ViewUpdate) {
          // no-op; widget will be recreated when editor is recreated
        }
        destroy() {
          // cleanup is automatic for widgets; nothing extra required
        }
      },
      { decorations: (v: any) => v.decorations }
    );

    return plugin;
  }

  // CSS for the inline widget is added to the document head so the widget
  // looks consistent with existing UI. We only inject once.
  function ensureProgressWidgetStyles() {
    if (document.getElementById('cm-progress-styles')) return;
    const style = document.createElement('style');
    style.id = 'cm-progress-styles';
    style.textContent = `
    .cm-progress-widget { padding: 8px 12px; background: var(--panel-bg); border-bottom: 1px solid var(--border-color); }
    .cm-progress-content { display:flex; align-items:center; gap:12px; }
    .cm-progress-lang { font-weight:600; color:var(--text-primary); }
    .cm-progress-status { font-size:12px; color:var(--text-secondary); }
    .cm-progress-bar { flex:1; height:4px; background:var(--border-color); border-radius:2px; overflow:hidden; }
    .cm-progress-line { height:100%; background:var(--accent-color); width:100%; animation:progress-slide 2s linear infinite; }
    @keyframes progress-slide { 0%{transform:translateX(-100%);} 100%{transform:translateX(100%);} }
    `;
    document.head.appendChild(style);
  }

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

    // Ensure widget styles are present
    ensureProgressWidgetStyles();

    // Insert progress widget plugin so CodeMirror renders an editor-integrated
    // progress bar. The widget reflects the current `isStreaming` state.
    extensions.unshift(createProgressExtension(isStreaming));

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

  // Create a minimal, compact CodeMirror instance inside the compact header
  // to render the progress widget when the block is collapsed.
  function createCompactEditor() {
    if (!compactContainer) return;
    ensureProgressWidgetStyles();

    const extensions = [
      createProgressExtension(isStreaming),
      EditorView.theme({ '&': { padding: '0', border: 'none', background: 'transparent' } }),
      EditorState.readOnly.of(true),
    ];

    const state = EditorState.create({ doc: '', extensions });
    compactView = new EditorView({ state, parent: compactContainer });
    console.log('[StreamingCodeBlock] createCompactEditor: isStreaming=', isStreaming);
  }

  function destroyCompactEditor() {
    if (compactView) {
      compactView.destroy();
      compactView = null;
      console.log('[StreamingCodeBlock] destroyCompactEditor');
    }
  }

  // Incremental update for streaming - only append new content
  function updateEditorIncremental(newCode: string) {
    if (!editorView) return;

    const currentCode = editorView.state.doc.toString();
    
    // Only append if new code is longer and starts with current code
    if (newCode.length > currentCode.length && newCode.startsWith(currentCode)) {
      const insertText = newCode.slice(currentCode.length);
      
      editorView.dispatch({
        changes: {
          from: currentCode.length,
          to: currentCode.length,
          insert: insertText
        },
        // Scroll to show new content
        effects: [
          EditorView.scrollIntoView(newCode.length, { y: 'end' })
        ]
      });
    } else if (newCode !== currentCode) {
      // Fallback to full replacement if content doesn't match expected pattern
      editorView.dispatch({
        changes: {
          from: 0,
          to: currentCode.length,
          insert: newCode
        }
      });
    }
  }

  // Full update for non-streaming changes
  function updateEditorFull(newCode: string) {
    if (!editorView) return;

    const currentCode = editorView.state.doc.toString();
    if (currentCode !== newCode) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: currentCode.length,
          insert: newCode
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

  function handleProgressBarClick() {
    if (!isStreaming) {
      isExpanded = !isExpanded;
      dispatch('toggle', { expanded: isExpanded });
    }
  }

  function handleStreamingTimeout() {
    if (streamingTimeout) {
      clearTimeout(streamingTimeout);
    }
    
    // Auto-complete streaming after 30 seconds of inactivity
    streamingTimeout = setTimeout(() => {
      if (isStreaming) {
        dispatch('streamingTimeout');
      }
    }, 30000);
  }

  onMount(() => {
    if (isExpanded || !isStreaming) {
      createEditor();
    } else if (!isExpanded && isStreaming) {
      // show compact progress widget when collapsed and streaming
      createCompactEditor();
    }
  });

  function handleCopyFromProgressBar() {
    if (editorView) {
      try {
        const text = editorView.state.doc.toString();
        navigator.clipboard.writeText(text);
        dispatch('copied');
      } catch (err) {
        console.error('Failed to copy CodeMirror content', err);
      }
    }
  }

  onDestroy(() => {
    destroyEditor();
    if (streamingTimeout) {
      clearTimeout(streamingTimeout);
    }
  });

  // React to streaming state changes
  $: if (isStreaming) {
    handleStreamingTimeout();
    if (isExpanded && editorView) {
      // Use incremental updates during streaming
      updateEditorIncremental(code);
    }
    // If collapsed, ensure compact view exists and is configured for streaming
    if (!isExpanded) {
      if (compactView) {
        // Reconfigure decorations to streaming mode without recreating the view
        try {
          compactView.dispatch({ effects: StateEffect.reconfigure.of([createProgressExtension(isStreaming)]) });
          console.log('[StreamingCodeBlock] reconfigured compactView -> streaming');
        } catch (err) {
          console.warn('[StreamingCodeBlock] failed to reconfigure compactView, recreating', err);
          destroyCompactEditor();
          createCompactEditor();
        }
      } else {
        createCompactEditor();
      }
    }
  } else {
    // Clear timeout when streaming stops
    if (streamingTimeout) {
      clearTimeout(streamingTimeout);
      streamingTimeout = null;
    }
    
    // Full update when streaming completes
    if (isExpanded && editorView) {
      updateEditorFull(code);
    }
    // Keep a non-streaming compact progress widget when collapsed; try to reconfigure
    if (!isExpanded) {
      if (compactView) {
        try {
          compactView.dispatch({ effects: StateEffect.reconfigure.of([createProgressExtension(isStreaming)]) });
          console.log('[StreamingCodeBlock] reconfigured compactView -> not streaming');
        } catch (err) {
          console.warn('[StreamingCodeBlock] failed to reconfigure compactView, recreating', err);
          destroyCompactEditor();
          createCompactEditor();
        }
      } else {
        createCompactEditor();
      }
    }
  }

  // React to expansion state changes
  $: if (isExpanded && !editorView && container) {
    // switching to expanded: destroy compact editor and create full editor
    destroyCompactEditor();
    createEditor();
  } else if (!isExpanded && editorView) {
    // switching to collapsed: destroy full editor and create compact view if streaming
    destroyEditor();
    if (isStreaming) {
      createCompactEditor();
    }
  }

  // React to other prop changes
  $: if (editorView && !isStreaming && code !== undefined) {
    updateEditorFull(code);
  }

  $: if (container && editorView && (language || theme)) {
    destroyEditor();
    createEditor();
  }

  // Track code length changes for streaming detection
  $: if (code.length !== lastCodeLength) {
    lastCodeLength = code.length;
  }

  // Listen for theme changes
  let mediaQuery: MediaQueryList;
  onMount(() => {
    if (theme === 'auto') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const handleChange = () => {
        if (editorView) {
          destroyEditor();
          createEditor();
        }
      };
      mediaQuery.addEventListener('change', handleChange);
      
      return () => {
        mediaQuery.removeEventListener('change', handleChange);
      };
    }
  });
</script>

<div class="streaming-code-block" class:streaming={isStreaming} class:expanded={isExpanded}>
  <!-- Compact header (ProgressBar replaced by CodeMirror in-editor widget) -->
  <div class="streaming-compact-header">
    <span class="streaming-lang-label">{language || 'text'}</span>
    <div bind:this={compactContainer} class="compact-codemirror-host"></div>
  </div>
  
  {#if isExpanded}
    <div 
      bind:this={container} 
      class="codemirror-wrapper"
      class:streaming-editor={isStreaming}
    ></div>
  {/if}
</div>

<style>
  .streaming-code-block {
    width: 100%;
    margin: 8px 0;
    border-radius: 8px;
    overflow: hidden;
    transition: all 0.3s ease;
  }

  .streaming-code-block.streaming {
    border: 1px solid var(--accent-color);
    box-shadow: 0 0 0 1px var(--accent-color-alpha);
  }

  .codemirror-wrapper {
    width: 100%;
    transition: opacity 0.3s ease;
  }

  .streaming-editor {
    opacity: 0.9;
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .streaming-code-block,
    .codemirror-wrapper {
      transition: none;
    }
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