<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorView, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
  import { EditorState, StateEffect, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { foldGutter, bracketMatching, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
  import { lineNumbers, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor } from '@codemirror/view';
  import CopySimple from 'phosphor-svelte/lib/CopySimple';
  let oneDark: any = null;
  // CodeMirror widget utilities for in-editor progress UI

  export let code: string = '';
  export let language: string = '';
  export let isStreaming: boolean = false;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  export let showLineNumbers: boolean = false;
  export let readonly: boolean = true;

  const dispatch = createEventDispatcher();
  
  let container: HTMLElement;
  let editorView: EditorView | null = null;
  let lastCodeLength: number = 0;
  // Local UI state for header/progress interactions
  let isExpanded: boolean = true;
  // Timer handle for streaming inactivity timeout
  let streamingTimeout: ReturnType<typeof setTimeout> | null = null;
  // Track transition from streaming -> idle to auto-collapse only once
  let wasStreaming: boolean = false;
  let prevIsStreaming: boolean = false;
  let collapseTimer: ReturnType<typeof setTimeout> | null = null;

  // Header actions (outside of the editor)
  function copyCurrentCode() {
    try {
      const text = editorView ? editorView.state.doc.toString() : code;
      if (text != null) navigator.clipboard.writeText(text);
    } catch {}
  }

  // Lazy language loading to reduce initial bundle size
  async function loadLanguageExtension(langName: string) {
    const lang = (langName || '').toLowerCase();
    try {
      switch (lang) {
        case 'javascript':
        case 'js': return (await import('@codemirror/lang-javascript')).javascript();
        case 'typescript':
        case 'ts': return (await import('@codemirror/lang-javascript')).javascript({ typescript: true });
        case 'jsx': return (await import('@codemirror/lang-javascript')).javascript({ jsx: true });
        case 'tsx': return (await import('@codemirror/lang-javascript')).javascript({ typescript: true, jsx: true });
        case 'python':
        case 'py': return (await import('@codemirror/lang-python')).python();
        case 'html': return (await import('@codemirror/lang-html')).html();
        case 'css': return (await import('@codemirror/lang-css')).css();
        case 'json': return (await import('@codemirror/lang-json')).json();
        case 'xml': return (await import('@codemirror/lang-xml')).xml();
        case 'sql': return (await import('@codemirror/lang-sql')).sql();
        default: return [];
      }
    } catch { return []; }
  }

  // Theme detection
  function getTheme(): 'light' | 'dark' {
    if (theme === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }

  // Compartments for dynamic reconfiguration of language and theme
  const compLang = new Compartment();
  const compDark = new Compartment();
  async function applyLanguage() {
    if (!editorView) return;
    const viewAtCall = editorView;
    const ext = await loadLanguageExtension(language);
    // Editor may be destroyed/recreated while awaiting
    if (!editorView || editorView !== viewAtCall) return;
    editorView.dispatch({ effects: compLang.reconfigure(ext) });
  }

  async function createEditor() {
    if (!container) return;

    const extensions = [
      // Line numbers disabled by default; can be re-enabled by theme if needed
      // highlightActiveLineGutter keeps active line styling without numbers
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

    // Language placeholder; real language applied asynchronously below
    extensions.push(compLang.of([]));

    // Add theme
    const currentTheme = getTheme();
    if (currentTheme === 'dark') {
      if (!oneDark) { try { oneDark = (await import('@codemirror/theme-one-dark')).oneDark; } catch {} }
      if (oneDark) extensions.push(compDark.of(oneDark));
    }

    // Configure readonly mode
    if (readonly) {
      extensions.push(EditorState.readOnly.of(true));
    }

    // Line numbers disabled unless explicitly enabled (we hide gutters)
    if (!showLineNumbers) {
      extensions.push(EditorView.theme({ '.cm-gutters': { display: 'none' } }));
    } else {
      extensions.push(lineNumbers());
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
      if (editorView) {
        editorView.dispatch({
          effects: StateEffect.reconfigure.of([...extensions, updateListener])
        });
      }
    }

    // Apply language after view mounts
    applyLanguage();
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
    // Disable expand/collapse during streaming
    if (isStreaming) return;
    isExpanded = !isExpanded;
    dispatch('toggle', { expanded: isExpanded });
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

  onMount(() => { createEditor(); });

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
    if (collapseTimer) {
      clearTimeout(collapseTimer);
      collapseTimer = null;
    }
  });

  // React to streaming state changes
  $: if (isStreaming) {
    // Force expanded state while streaming
    if (!isExpanded) isExpanded = true;
    // Cancel any pending auto-collapse while streaming resumes
    if (collapseTimer) { clearTimeout(collapseTimer); collapseTimer = null; }
    handleStreamingTimeout();
    if (editorView) {
      // Use incremental updates during streaming
      updateEditorIncremental(code);
    }
    // Ensure block stays in view (stick to bottom of viewport)
    if (container) {
      requestAnimationFrame(() => {
        try { container.scrollIntoView({ block: 'end', behavior: 'auto' }); } catch {}
      });
    }
    wasStreaming = true;
  } else {
    // Clear timeout when streaming stops
    if (streamingTimeout) {
      clearTimeout(streamingTimeout);
      streamingTimeout = null;
    }
    // Full update when streaming completes
    if (editorView) {
      updateEditorFull(code);
    }
    // Auto-collapse only on actual transition streaming -> idle, with slight delay to avoid race
    if (wasStreaming && prevIsStreaming && !isStreaming) {
      if (collapseTimer) { clearTimeout(collapseTimer); }
      collapseTimer = setTimeout(() => {
        if (!isStreaming) {
          isExpanded = false;
          wasStreaming = false;
          collapseTimer = null;
        }
      }, 120);
    }
  }

  // Track previous streaming state
  $: prevIsStreaming = isStreaming;

  // No external expand/collapse controls; header click toggles collapsed state

  // React to other prop changes
  $: if (editorView && !isStreaming && code !== undefined) {
    updateEditorFull(code);
  }

  // Reconfigure language on change
  $: if (editorView && language !== undefined) { applyLanguage(); }
  // Recreate on theme change (rare) to keep header/widget layout intact
  $: if (container && editorView && theme) {
    // Only handle theme auto changes via media listener; explicit theme toggle triggers recreate
  }

  // No longer recreate editor based on streaming flag: header is external

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

<div class="streaming-code-block" class:streaming={isStreaming} class:collapsed={!isExpanded}>
  <div class="streaming-code-header" role="button" aria-label="Code header" aria-expanded={isExpanded} aria-disabled={isStreaming} tabindex={isStreaming ? -1 : 0} on:click={handleProgressBarClick} on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleProgressBarClick()}>
    <div class="left">
      <span class="status">{language || 'text'}</span>
    </div>
    <div class="right">
      <button class="copy-btn" type="button" aria-label="Копировать" title="Копировать" on:click|stopPropagation={copyCurrentCode}>
        <CopySimple size={16} />
      </button>
    </div>
  </div>
  <div bind:this={container} class="codemirror-wrapper" class:streaming-editor={isStreaming}></div>
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

  /* Hide gutters (line numbers, fold markers) for streaming editors */
  :global(.streaming-code-block .cm-gutters) {
    display: none !important;
  }

  .streaming-code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    background: var(--panel-bg);
    border: 1px solid var(--border-color);
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    cursor: pointer;
  }
  .streaming-code-header .left { display: flex; align-items: center; gap: 8px; }
  .streaming-code-header[aria-disabled="true"] {
    opacity: 0.75;
    cursor: not-allowed;
  }
  .streaming-code-header .status { font-size: 12px; color: var(--muted); }
  .streaming-code-header .copy-btn {
    display: grid; place-items: center;
    width: 24px; height: 24px;
    padding: 0; margin: 0;
    border: 1px solid var(--border-color);
    background: var(--panel-bg); color: var(--text); border-radius: 4px;
  }
  .codemirror-wrapper { border: 1px solid var(--border-color); border-top: none; border-radius: 0 0 8px 8px; }

  /* Collapse styles */
  .streaming-code-block.collapsed .codemirror-wrapper { display: none; }
  .streaming-code-block.collapsed .streaming-code-header {
    border-bottom: 1px solid var(--border-color);
    border-radius: 8px;
  }

  :global(.cm-progress-widget) { cursor: pointer; }
  :global(.cm-progress-copy) {
    font-size: 12px;
    padding: 2px 8px;
    border: 1px solid var(--border-color);
    background: var(--panel-bg);
    color: var(--text-primary);
    border-radius: 4px;
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .streaming-code-block,
    .codemirror-wrapper {
      transition: none;
    }
    /* Keep spinner running even with reduced motion to signal activity */
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
