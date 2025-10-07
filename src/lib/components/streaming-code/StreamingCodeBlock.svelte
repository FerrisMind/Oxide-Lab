<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import CopySimple from 'phosphor-svelte/lib/CopySimple';
  import CircleNotch from 'phosphor-svelte/lib/CircleNotch';
  import CheckCircle from 'phosphor-svelte/lib/CheckCircle';
  import hljs from 'highlight.js/lib/common';

  export let code: string = '';
  export let language: string = '';
  export let isStreaming: boolean = false;
  export let isComplete: boolean = false;
  export let theme: 'light' | 'dark' | 'auto' = 'auto';
  // Spinner control props
  export let spinnerStartSignal: string = '[CODE]';
  export let spinnerEndSignal: string = '[CODE_END]';
  // Props kept for API совместимости; подсветка highlight.js их не использует
  export const showLineNumbers: boolean = false;
  export const readonly: boolean = true;

  const dispatch = createEventDispatcher();

  let container: HTMLElement;
  let preEl: HTMLPreElement | null = null;
  let codeEl: HTMLElement | null = null;
  let lastCodeLength: number = 0;
  let renderedCode: string = '';
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
  let headerIconState: 'spinner' | 'check' | 'hidden' = 'hidden';
  let isSpinnerActive: boolean = false;

  // Detect spinner signals in code
  $: {
    const codeStr = code || '';
    const hasCodeStart = spinnerStartSignal && codeStr.includes(spinnerStartSignal);
    const hasCodeEnd = spinnerEndSignal && codeStr.includes(spinnerEndSignal);

    if (hasCodeStart && !isSpinnerActive) {
      // Start spinning when start signal is detected
      isSpinnerActive = true;
      headerIconState = 'spinner';
    } else if (hasCodeEnd && isSpinnerActive) {
      // Stop spinning and show check when end signal is detected
      isSpinnerActive = false;
      headerIconState = 'check';
      // Auto-hide check after 1.5s
      if (iconTransitionTimer) clearTimeout(iconTransitionTimer);
      iconTransitionTimer = setTimeout(() => {
        headerIconState = 'hidden';
        iconTransitionTimer = null;
      }, 1500);
    } else if (isSpinnerActive) {
      // Keep spinner active while spinning
      headerIconState = 'spinner';
    } else {
      // Hide when not active
      headerIconState = 'hidden';
    }
  }
  let iconTransitionTimer: ReturnType<typeof setTimeout> | null = null;
  // Whether we've already auto-expanded after a streaming session
  let autoExpandedDone: boolean = false;
  // Streaming spinner state
  let _spinnerHost: HTMLSpanElement | null = null;

  // Header actions (outside of the editor)
  function copyCurrentCode() {
    try {
      const text = renderedCode || code || '';
      if (text != null) navigator.clipboard.writeText(text);
      dispatch('copied');
    } catch {}
  }

  const hljsAliases: Record<string, string> = {
    html: 'xml',
    xhtml: 'xml',
    xml: 'xml',
    svg: 'xml',
    md: 'markdown',
    markdown: 'markdown',
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    sh: 'bash',
    shell: 'bash',
    zsh: 'bash',
    console: 'bash',
    shellsession: 'bash',
    ps1: 'powershell',
    powershell: 'powershell',
    py: 'python',
    rb: 'ruby',
    kt: 'kotlin',
    rs: 'rust',
    csharp: 'csharp',
    'c#': 'csharp',
    golang: 'go',
    plaintext: 'plaintext',
    text: 'plaintext',
  };

  function activeLanguage(): string {
    const raw = (language || '').toLowerCase().trim();
    const alias = hljsAliases[raw];
    if (alias) return alias;
    return raw.length ? raw : 'plaintext';
  }

  function ensureCodeElement() {
    if (!container) return;
    if (!preEl || !codeEl || !container.contains(codeEl)) {
      const className = 'hljs language-' + activeLanguage();
      container.innerHTML =
        '<pre class="' + className + '"><code class="' + className + '"></code></pre>';
      preEl = container.querySelector('pre');
      codeEl = container.querySelector('code');
    }
  }

  function renderHighlightedCode(codeStr: string, scrollIntoView = false) {
    if (!container) return;
    ensureCodeElement();
    if (!codeEl) return;

    renderedCode = codeStr ?? '';
    const lang = activeLanguage();
    let languageUsed = lang;
    let highlighted: string;
    try {
      if (lang && hljs.getLanguage(lang)) {
        highlighted = hljs.highlight(renderedCode, { language: lang }).value;
      } else {
        const auto = hljs.highlightAuto(renderedCode);
        highlighted = auto.value;
        languageUsed = auto.language ?? 'plaintext';
      }
    } catch {
      highlighted = escapeHtml(renderedCode);
      languageUsed = 'plaintext';
    }

    const className = 'hljs language-' + (languageUsed || 'plaintext');
    if (codeEl.className !== className) codeEl.className = className;
    if (preEl && preEl.className !== className) preEl.className = className;

    if (codeEl.innerHTML !== highlighted) {
      codeEl.innerHTML = highlighted;
    }

    if (preEl) {
      preEl.style.margin = '0';
      preEl.style.padding = '12px 16px';
      preEl.style.overflow = 'auto';
      preEl.style.lineHeight = '1.6';
      preEl.style.fontSize = '0.85rem';
      preEl.style.background = 'transparent';
      preEl.style.color = 'var(--code-fg, var(--text-primary))';
      preEl.style.fontFamily =
        'var(--font-monospace, ui-monospace, "SFMono-Regular", Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace)';
    }

    if (codeEl) {
      codeEl.style.display = 'block';
      codeEl.style.background = 'transparent';
    }

    if (scrollIntoView && preEl) {
      requestAnimationFrame(() => {
        try {
          preEl!.scrollTop = preEl!.scrollHeight;
        } catch {}
        try {
          container.scrollIntoView({ block: 'end', behavior: 'auto' });
        } catch {}
      });
    }
  }

  function createEditor() {
    if (!container) return;
    renderHighlightedCode(code);
  }

  function updateEditorIncremental(newCode: string) {
    renderHighlightedCode(newCode, true);
  }

  function scheduleStreamingUpdate(newCode: string) {
    pendingStreamingCode = newCode;
    if (pendingAnimationFrame) return;
    pendingAnimationFrame = requestAnimationFrame(() => {
      pendingAnimationFrame = null;
      if (pendingStreamingCode !== null) {
        updateEditorIncremental(pendingStreamingCode);
        pendingStreamingCode = null;
      }
    });
  }

  function updateEditorFull(newCode: string) {
    renderHighlightedCode(newCode, false);
  }

  function destroyEditor() {
    if (container) container.innerHTML = '';
    preEl = null;
    codeEl = null;
    renderedCode = '';
  }

  function escapeHtml(value: string): string {
    return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
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

  onMount(() => {
    createEditor();
  });

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
    if (collapseTimer) {
      clearTimeout(collapseTimer);
      collapseTimer = null;
    }
    handleStreamingTimeout();
    // Coalesced incremental updates during streaming to avoid UI jank
    scheduleStreamingUpdate(code);
    // Keep block in view (stick to bottom of viewport)
    if (container) {
      requestAnimationFrame(() => {
        try {
          container.scrollIntoView({ block: 'end', behavior: 'auto' });
        } catch {}
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
    updateEditorFull(code);
    // If we just finished a streaming session, auto-expand once and then don't auto-toggle anymore
    if (wasStreaming && prevIsStreaming && !isStreaming) {
      // Small delay to avoid races with the final chunk
      if (collapseTimer) {
        clearTimeout(collapseTimer);
      }
      collapseTimer = setTimeout(() => {
        if (!isStreaming && !autoExpandedDone) {
          isExpanded = true;
          autoExpandedDone = true;
        }
        wasStreaming = false;
        collapseTimer = null;

        // Icon transition is now handled by the reactive block above
      }, 120);
    }
  }

  // Auto-expand when this specific code block becomes complete (closing ``` detected)
  $: if (isComplete && !autoExpandedDone) {
    // Only auto-expand once per block; respect manual user collapse/expand afterwards
    isExpanded = true;
    autoExpandedDone = true;
  }

  // Track previous streaming state
  $: prevIsStreaming = isStreaming;

  // No external expand/collapse controls; header click toggles collapsed state

  // React to other prop changes: re-render when code changes and not streaming
  $: if (container && !isStreaming && code !== undefined) {
    updateEditorFull(code);
  }
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
  <div
    class="streaming-code-header"
    role="button"
    aria-label="Code header"
    aria-expanded={isExpanded}
    aria-disabled={isStreaming}
    tabindex={isStreaming ? -1 : 0}
    on:click={handleProgressBarClick}
    on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleProgressBarClick()}
  >
    <div class="left">
      <span
        class="header-icon"
        aria-hidden="true"
        class:visible={headerIconState !== 'hidden'}
        class:spinner={headerIconState === 'spinner'}
      >
        {#if headerIconState === 'spinner'}
          <CircleNotch size={14} weight="bold" class="spinner-icon" />
        {:else if headerIconState === 'check'}
          <CheckCircle size={14} weight="bold" class="check" />
        {/if}
      </span>
      <span class="status">{language || 'text'}</span>
    </div>
    <div class="right">
      <button
        class="copy-btn"
        type="button"
        aria-label="Копировать"
        title="Копировать"
        on:click|stopPropagation={copyCurrentCode}
      >
        <CopySimple size={16} />
      </button>
    </div>
  </div>
  <div bind:this={container} class="code-wrapper" class:streaming-editor={isStreaming}></div>
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

  :global(.code-wrapper) {
    width: 100%;
    transition: opacity 0.3s ease;
    border: 1px solid var(--border-color);
    border-top: none;
    border-radius: 0 0 8px 8px;
    background: var(--code-bg, var(--panel-bg));
  }

  .streaming-editor {
    opacity: 0.9;
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
    cursor: default;
  }
  .streaming-code-header .left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .streaming-code-header .header-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    opacity: 0;
    transition:
      opacity 200ms linear,
      transform 200ms ease;
  }
  .streaming-code-header .header-icon.visible {
    opacity: 1;
    transform: translateY(0);
  }
  .streaming-code-header .header-icon :global(svg) {
    color: var(--accent-color);
    width: 18px;
    height: 18px;
    display: block;
    transform-box: fill-box;
  }
  :global(.header-icon.spinner) {
    animation: cm-spinner-rotate 1s linear infinite;
  }
  :global(.header-icon) :global(.spinner-icon) {
    animation: cm-spinner-rotate 1s linear infinite;
  }
  :global(.header-icon) :global(.spinner-icon) {
    transform-origin: center center;
  }
  :global(.header-icon) :global(.check) {
    transform: scale(0.95);
  }
  .streaming-code-header[aria-disabled='true'] {
    opacity: 0.75;
    cursor: not-allowed;
  }
  .streaming-code-header .status {
    font-size: 12px;
    color: var(--muted);
  }
  .streaming-code-header .copy-btn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    margin: 0;
    border: 1px solid var(--border-color);
    background: var(--panel-bg);
    color: var(--text);
    border-radius: 4px;
  }

  /* Collapse styles */
  .streaming-code-block.collapsed .code-wrapper {
    display: none;
  }
  .streaming-code-block.collapsed .streaming-code-header {
    border-bottom: 1px solid var(--border-color);
    border-radius: 8px;
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .streaming-code-block,
    .code-wrapper {
      transition: none;
    }
  }

  @keyframes cm-spinner-rotate {
    to {
      transform: rotate(360deg);
    }
  }
</style>
