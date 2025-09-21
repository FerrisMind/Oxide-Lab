<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import CopySimple from 'phosphor-svelte/lib/CopySimple';
  import CircleNotch from 'phosphor-svelte/lib/CircleNotch';
  import CheckCircle from 'phosphor-svelte/lib/CheckCircle';
  let _oneDark: any = null;
  // Shiki will be dynamically imported to allow ESM-bundle usage and Tauri offline loading

  export let code: string = '';
  export let language: string = '';
  export let isStreaming: boolean = false;
  export let isComplete: boolean = false;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  // Props kept for API compatibility; not used by Shiki renderer
  export const showLineNumbers: boolean = false;
  export const readonly: boolean = true;

  const dispatch = createEventDispatcher();
  
  let container: HTMLElement;
  // Buffer/state for our simplified Shiki renderer
  let lastCodeLength: number = 0;
  let renderedCode: string = '';
  const highlighterCache: Record<string, any> = {};
  // Buffering state for streaming updates to coalesce rapid incoming chunks
  let pendingStreamingCode: string | null = null;
  let pendingAnimationFrame: number | null = null;
  // Local UI state for header/progress interactions
  // By default code blocks should be collapsed per UX requirement
  let isExpanded: boolean = false;
  // Timer handle for streaming inactivity timeout
  let streamingTimeout: ReturnType<typeof setTimeout> | null = null;
  // Track transition from streaming -> idle to auto-collapse only once
  let wasStreaming: boolean = false;
  let prevIsStreaming: boolean = false;
  let collapseTimer: ReturnType<typeof setTimeout> | null = null;
  // Icon transition state: 'spinner' | 'check' | 'hidden'
  let headerIconState: 'spinner' | 'check' | 'hidden' = isStreaming ? 'spinner' : 'hidden';
  let iconTransitionTimer: ReturnType<typeof setTimeout> | null = null;
  // Whether we've already auto-expanded after a streaming session
  let autoExpandedDone: boolean = false;

  // Header actions (outside of the editor)
  function copyCurrentCode() {
    try {
      const text = renderedCode || code || '';
      if (text != null) navigator.clipboard.writeText(text);
      dispatch('copied');
    } catch {}
  }

  // Lazy language loading to reduce initial bundle size
  // For Shiki we dynamically create/cached highlighters per (lang,theme).
  function getHighlighterKey(lang: string, theme: string) {
    return `${lang}::${theme}`;
  }

  async function ensureHighlighter(lang: string, theme: string) {
    const key = getHighlighterKey(lang, theme);
    if ((highlighterCache as any)[key]) return (highlighterCache as any)[key];
    // Dynamically import the ESM bundle at runtime — this helps Tauri packaging and avoids build-time resolution errors
    // @ts-ignore dynamic import of optional bundle for Tauri packaging
    const mod: any = await import('shiki/bundle/web');
    // createHighlighter from bundle/web accepts langs/themes arrays
    const { createHighlighter } = mod;
    const highlighter = await createHighlighter({ langs: [lang], themes: [theme] });
    (highlighterCache as any)[key] = highlighter;
    return highlighter;
  }

  // Theme detection
  function getTheme(): 'light' | 'dark' {
    if (theme === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return theme;
  }

  // Removed CodeMirror compartments and language loader — using Shiki instead

  // Create a simple container-based code render using Shiki
  async function createEditor() {
    if (!container) return;
    // Initial render
    await renderHighlightedCode(code);
  }

  // Incremental update for streaming - for Shiki we re-render the highlighted HTML
  async function updateEditorIncremental(newCode: string) {
    await renderHighlightedCode(newCode);
  }

  // Schedule/coalesce streaming updates to avoid rapid re-highlighting
  function scheduleStreamingUpdate(newCode: string) {
    pendingStreamingCode = newCode;
    if (pendingAnimationFrame) return;
    pendingAnimationFrame = requestAnimationFrame(async () => {
      pendingAnimationFrame = null;
      if (pendingStreamingCode !== null) {
        try {
          await updateEditorIncremental(pendingStreamingCode);
        } catch (_e) {
          await renderHighlightedCode(pendingStreamingCode);
        }
        pendingStreamingCode = null;
      }
    });
  }

  // Full update for non-streaming changes -> re-render highlighted HTML
  async function updateEditorFull(newCode: string) {
    await renderHighlightedCode(newCode);
  }

  function destroyEditor() {
    // Clear container
    if (container) container.innerHTML = '';
    renderedCode = '';
  }

  // Render highlighted HTML using Shiki and insert into container
  async function renderHighlightedCode(codeStr: string) {
    if (!container) return;
    renderedCode = codeStr ?? '';
    const lang = (language || 'text').toLowerCase();
    const theme = getTheme() === 'dark' ? 'github-dark' : 'github-light';
    try {
      const highlighter = await ensureHighlighter(lang, theme);
      const html = highlighter.codeToHtml(renderedCode, { lang, theme });
      // Use innerHTML: we generate HTML ourselves from Shiki -> safe
      container.innerHTML = html;
      // Keep container scrolled to end when streaming
      requestAnimationFrame(() => {
        try { container.scrollIntoView({ block: 'end', behavior: 'auto' }); } catch {}
      });
    } catch (_err) {
      // Fallback: plain text inside pre
      container.innerHTML = `<pre><code>${escapeHtml(renderedCode)}</code></pre>`;
    }
  }

  function escapeHtml(unsafe: string) {
    return unsafe.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
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

  function _handleCopyFromProgressBar() {
    try {
      const text = renderedCode || code || '';
      navigator.clipboard.writeText(text);
      dispatch('copied');
    } catch (err) {
      console.error('Failed to copy code content', err);
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
    if (iconTransitionTimer) {
      clearTimeout(iconTransitionTimer);
      iconTransitionTimer = null;
    }
    if (pendingAnimationFrame) {
      cancelAnimationFrame(pendingAnimationFrame);
      pendingAnimationFrame = null;
    }
  });

  // React to streaming state changes
  $: if (isStreaming) {
    // While streaming, keep the editor collapsed and prevent auto-expansion
    if (isExpanded) isExpanded = false;
    autoExpandedDone = false;
    // Cancel any pending auto-collapse/expand timers
    if (collapseTimer) { clearTimeout(collapseTimer); collapseTimer = null; }
    handleStreamingTimeout();
    // Coalesced incremental updates during streaming to avoid UI jank
    scheduleStreamingUpdate(code);
    // Keep block in view (stick to bottom of viewport)
    if (container) {
      requestAnimationFrame(() => {
        try { container.scrollIntoView({ block: 'end', behavior: 'auto' }); } catch {}
      });
    }
    wasStreaming = true;
    // Ensure header shows spinner immediately
    headerIconState = 'spinner';
  } else {
    // Clear timeout when streaming stops
    if (streamingTimeout) {
      clearTimeout(streamingTimeout);
      streamingTimeout = null;
    }
    // Full update when streaming completes
    updateEditorFull(code);
    // If we just finished a streaming session, auto-expand once and then don't auto-toggle anymore
    if (wasStreaming && prevIsStreaming && !isStreaming) {
      // Small delay to avoid races with the final chunk
      if (collapseTimer) { clearTimeout(collapseTimer); }
      collapseTimer = setTimeout(() => {
        if (!isStreaming && !autoExpandedDone) {
          isExpanded = true;
          autoExpandedDone = true;
        }
        wasStreaming = false;
        collapseTimer = null;

        // Start icon transition: spinner -> check -> hidden
        headerIconState = 'check';
        if (iconTransitionTimer) clearTimeout(iconTransitionTimer);
        // after 2s show check, after additional 1.5s hide icon
        iconTransitionTimer = setTimeout(() => {
          headerIconState = 'hidden';
          iconTransitionTimer = null;
        }, 1500);
      }, 120);
    }
  }

  // Auto-expand when this specific code block becomes complete (closing ``` detected)
  $: if (isComplete && !autoExpandedDone) {
    // Only auto-expand once per block; respect manual user collapse/expand afterwards
    isExpanded = true;
    autoExpandedDone = true;
    // When complete (not streaming) show check briefly then hide
    headerIconState = 'check';
    if (iconTransitionTimer) clearTimeout(iconTransitionTimer);
    iconTransitionTimer = setTimeout(() => {
      headerIconState = 'hidden';
      iconTransitionTimer = null;
    }, 1500);
  }

  // Track previous streaming state
  $: prevIsStreaming = isStreaming;

  // No external expand/collapse controls; header click toggles collapsed state

  // React to other prop changes: re-render when code changes and not streaming
  $: if (container && !isStreaming && code !== undefined) { updateEditorFull(code); }
  // Recreate on theme change (rare) to keep header/widget layout intact
  $: if (container && theme) {
    // Re-render to pick new theme
    updateEditorFull(code);
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

<div class="streaming-code-block" class:streaming={isStreaming} class:collapsed={!isExpanded}>
  <div class="streaming-code-header" role="button" aria-label="Code header" aria-expanded={isExpanded} aria-disabled={isStreaming} tabindex={isStreaming ? -1 : 0} on:click={handleProgressBarClick} on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleProgressBarClick()}>
    <div class="left">
      <span class="header-icon" aria-hidden="true" class:visible={headerIconState !== 'hidden'}>
        {#if headerIconState === 'spinner'}
          <CircleNotch size={14} weight="bold" class="spinner" />
        {:else if headerIconState === 'check'}
          <CheckCircle size={14} weight="bold" class="check" />
        {/if}
      </span>
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
  .streaming-code-header .header-icon { display: inline-flex; align-items: center; justify-content: center; width: 18px; height: 18px; opacity: 0; transition: opacity 200ms linear, transform 200ms ease; }
  .streaming-code-header .header-icon.visible { opacity: 1; transform: translateY(0); }
  .streaming-code-header .header-icon :global(svg) { color: var(--accent-color); width: 18px; height: 18px; display: block; transform-box: fill-box; }
  :global(.header-icon) :global(.spinner) { animation: cm-spinner-rotate 1s linear infinite; transform-origin: center center; }
  :global(.header-icon) :global(.check) { transform: scale(0.95); }
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

  @keyframes cm-spinner-rotate {
    to { transform: rotate(360deg); }
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

