<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorView, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
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
  import ProgressBar from './ProgressBar.svelte';

  export let code: string = '';
  export let language: string = '';
  export let isStreaming: boolean = false;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  export let showLineNumbers: boolean = true;
  export let readonly: boolean = true;

  const dispatch = createEventDispatcher();
  
  let container: HTMLElement;
  let editorView: EditorView | null = null;
  let isExpanded: boolean = false;
  let lastCodeLength: number = 0;
  let streamingTimeout: number | null = null;

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
    }
  });

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
  }

  // React to expansion state changes
  $: if (isExpanded && !editorView && container) {
    createEditor();
  } else if (!isExpanded && editorView) {
    destroyEditor();
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
  <ProgressBar 
    {language} 
    {isStreaming} 
    {isExpanded}
    on:click={handleProgressBarClick}
  />
  
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