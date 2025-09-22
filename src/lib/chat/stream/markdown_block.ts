import { mount, unmount } from 'svelte';
import Eye from 'phosphor-svelte/lib/Eye';
import EyeSlash from 'phosphor-svelte/lib/EyeSlash';
import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
// CodeMirror renderer removed — using Shiki-based StreamingCodeBlock for highlighting
import { enableExternalLinks } from '$lib/chat/external-links';
import { StreamingCodeBlock } from '$lib/components/streaming-code';
import type { BubbleCtx } from './bubble_ctx';

function hasMarkdownFeatures(text: string): boolean {
  const t = text ?? '';
  if (!t) return false;
  return (
    /(^|\n)\s{0,3}#{1,6}\s+/m.test(t) || // заголовки
    /(^|\n)\s{0,3}[-*+]\s+/.test(t) || // марк. списки
    /(^|\n)\s{0,3}\d+\.\s+/.test(t) || // нум. списки
    /```|~~~/.test(t) || // кодовые блоки
    /`[^`\n]+`/.test(t) || // инлайн-код
    /!\[[^\]]*\]\([^\)]+\)/.test(t) || // изображения
    /\[[^\]]+\]\([^\)]+\)/.test(t) || // ссылки
    /(^|\n)\|[^\n]*\|/.test(t) || // таблицы
    /(^|\n)>\s+/.test(t) // цитаты
  );
}

export function ensureMarkdownContainer(ctx: BubbleCtx, bubble: HTMLDivElement): BubbleCtx {
  if (ctx.lastKind !== 'text' || !ctx.mdEl) {
    // Cleanup existing streaming components if any
    if (ctx.mdContentEl) {
      cleanupStreamingCodeComponents(ctx.mdContentEl);
    }

    ctx.mdEl = document.createElement('div');
    ctx.mdEl.className = 'md-stream';

    // controls
    const controls = document.createElement('div');
    controls.className = 'md-controls';
    const toggleBtn = document.createElement('button');
    toggleBtn.type = 'button';
    toggleBtn.className = 'md-toggle';
    const eyeHost = document.createElement('span');
    eyeHost.className = 'md-eye-host';
    toggleBtn.appendChild(eyeHost);
    controls.appendChild(toggleBtn);

    // content containers
    const contentEl = document.createElement('div');
    contentEl.className = 'md-content';
    const rawEl = document.createElement('pre');
    rawEl.className = 'md-raw';

    // assemble
    ctx.mdEl.appendChild(controls);
    ctx.mdEl.appendChild(contentEl);
    ctx.mdEl.appendChild(rawEl);
    bubble.appendChild(ctx.mdEl);

    // mount eye icon
    ctx.mdEyeHost = eyeHost;
    ctx.mdEyeIcon = mount(Eye, { target: eyeHost, props: { size: 16, weight: 'regular' } });

    // toggle handler
    toggleBtn.addEventListener('click', () => {
      const showingRaw = ctx.mdEl?.classList.toggle('show-raw') ?? false;
      try {
        if (ctx.mdEyeIcon) unmount(ctx.mdEyeIcon);
      } catch {}
      if (ctx.mdEyeHost) {
        ctx.mdEyeIcon = mount(showingRaw ? EyeSlash : Eye, {
          target: ctx.mdEyeHost,
          props: { size: 16, weight: 'regular' },
        });
      }
    });

    ctx.mdControlsEl = controls;
    ctx.mdToggleBtn = toggleBtn;
    // По умолчанию скрываем контрол до появления markdown-признаков
    try {
      (ctx.mdControlsEl as HTMLElement).style.display = 'none';
    } catch {}
    ctx.mdContentEl = contentEl;
    ctx.mdRawEl = rawEl;
    ctx.mdText = '';
  }
  return ctx;
}

// Detect if text contains streaming code blocks
function hasStreamingCodeBlocks(text: string): boolean {
  // Match both complete and incomplete code blocks
  return /```[\w]*\n[\s\S]*?(?:```|$)/.test(text);
}

// Extract code blocks for streaming
function extractCodeBlocks(text: string): Array<{
  language: string;
  code: string;
  isComplete: boolean;
  startIndex: number;
  endIndex: number;
}> {
  const codeBlocks: Array<{
    language: string;
    code: string;
    isComplete: boolean;
    startIndex: number;
    endIndex: number;
  }> = [];
  // Improved regex to better handle streaming code blocks
  // Use non-greedy matching to avoid cutting off content after code blocks
  const regex = /```(\w*)\n([\s\S]*?)(?:```|$)/g;
  let match;

  while ((match = regex.exec(text)) !== null) {
    const language = match[1] || 'text';
    const code = match[2] || '';
    const isComplete = match[0].endsWith('```');
    const startIndex = match.index;
    const endIndex = match.index + match[0].length;

    codeBlocks.push({ language, code, isComplete, startIndex, endIndex });
  }

  return codeBlocks;
}

// Replace code blocks with streaming components
function renderMarkdownWithStreamingCode(text: string, _isStreaming: boolean = false): string {
  if (!hasStreamingCodeBlocks(text)) {
    return renderMarkdownToSafeHtml(text);
  }

  const codeBlocks = extractCodeBlocks(text);

  let processedText = text;

  // Process code blocks in reverse order to maintain correct indices
  for (let i = codeBlocks.length - 1; i >= 0; i--) {
    const block = codeBlocks[i];
    const blockId = `streaming-code-${i}`;
    // Minimal placeholder: only mount point for Svelte component (no external header/labels)
    const placeholder = `
<div id="${blockId}" class="streaming-code-placeholder" data-language="${block.language}" data-code="${encodeURIComponent(block.code)}" data-streaming="${_isStreaming && !block.isComplete}" data-is-complete="${block.isComplete}">
  <div class="streaming-code-mount"></div>
</div>`;

    // Replace the code block with placeholder using exact indices
    processedText =
      processedText.substring(0, block.startIndex) +
      placeholder +
      processedText.substring(block.endIndex);
  }

  return renderMarkdownToSafeHtml(processedText);
}

// Mount streaming code components
function mountStreamingCodeComponents(container: HTMLElement, _isStreaming: boolean = false) {
  const placeholders = container.querySelectorAll('.streaming-code-placeholder');

  placeholders.forEach((placeholder) => {
    const element = placeholder as HTMLElement;
    const language = element.dataset.language || '';
    const code = decodeURIComponent(element.dataset.code || '');
    const streaming = element.dataset.streaming === 'true';
    const isComplete = element.dataset.isComplete === 'true';

    try {
      // Reuse already-mounted component if present to avoid tearing down during streaming
      if ((element as any).__streamingCodeComponent) {
        const existing = (element as any).__streamingCodeComponent;
        try {
          // In Svelte 5, we need to update props directly on the component instance
          // We'll update each prop individually if they exist on the component
          if (existing.$$set) {
            existing.$$set({
              code,
              language,
              isStreaming: streaming,
              isComplete,
              readonly: true,
              showLineNumbers: false,
            });
          }
        } catch (e) {
          // If update fails, fall back to re-mount
          try {
            unmount(existing);
          } catch {}
        }
        return;
      }

      // Mount Svelte component into the mount node if present, otherwise mount into element
      const mountTarget = element.querySelector('.streaming-code-mount') ?? element;
      const component = mount(StreamingCodeBlock, {
        target: mountTarget as Element,
        props: {
          code,
          language,
          isStreaming: streaming,
          isComplete,
          readonly: true,
          // For streaming UI we hide line numbers by default
          showLineNumbers: false,
        } as any,
      });

      // Store component reference for cleanup and future updates
      (element as any).__streamingCodeComponent = component;
    } catch (error) {
      console.error('Failed to mount StreamingCodeBlock:', error);
      // Fallback to regular code block without external header/labels
      element.innerHTML = `<pre><code class="language-${language}">${code}</code></pre>`;
    }
  });
}

// Cleanup streaming code components
function cleanupStreamingCodeComponents(container: HTMLElement) {
  const placeholders = container.querySelectorAll('.streaming-code-placeholder');

  placeholders.forEach((placeholder) => {
    const element = placeholder as HTMLElement;
    const component = (element as any).__streamingCodeComponent;

    if (component) {
      try {
        unmount(component);
      } catch (error) {
        console.error('Failed to unmount StreamingCodeBlock:', error);
      }
      delete (element as any).__streamingCodeComponent;
    }
  });
}

export function appendMarkdownText(
  ctx: BubbleCtx,
  text: string,
  isStreaming: boolean = false,
): BubbleCtx {
  const normalized = text.replace(/\r/g, '');
  ctx.mdText += normalized;

  if (ctx.mdContentEl) {
    // Render markdown with streaming code support
    const newContent = renderMarkdownWithStreamingCode(ctx.mdText, isStreaming);

    // Check if the structure has changed (new code blocks added)
    const hasStructuralChanges = hasStreamingCodeBlocksChanged(ctx.mdContentEl, newContent);

    if (hasStructuralChanges) {
      // Only cleanup and re-render if the structure has changed
      cleanupStreamingCodeComponents(ctx.mdContentEl);

      // Update the content
      ctx.mdContentEl.innerHTML = newContent;

      // Enable external link handling
      enableExternalLinks(ctx.mdContentEl);

      // Mount streaming code components
      mountStreamingCodeComponents(ctx.mdContentEl, isStreaming);
    } else {
      // Just update existing components without remounting
      // During streaming, we need to update the code content in placeholders
      updateStreamingCodeComponents(ctx.mdContentEl, isStreaming, ctx.mdText);
    }

    // No CodeMirror rendering anymore — Shiki handles highlighting in streaming components.
  }

  if (ctx.mdRawEl) {
    ctx.mdRawEl.textContent = ctx.mdText;
  }

  // Показываем/скрываем кнопку-глаз только если есть элементы Markdown
  try {
    if (ctx.mdControlsEl) {
      (ctx.mdControlsEl as HTMLElement).style.display = hasMarkdownFeatures(ctx.mdText)
        ? 'flex'
        : 'none';
    }
  } catch {}

  ctx.lastKind = 'text';
  return ctx;
}

// Check if the structure of code blocks has changed
function hasStreamingCodeBlocksChanged(container: HTMLElement, newContent: string): boolean {
  // Parse the new content to check for structural changes
  const tempDiv = document.createElement('div');
  tempDiv.innerHTML = newContent;
  const newPlaceholders = tempDiv.querySelectorAll('.streaming-code-placeholder');
  const existingPlaceholders = container.querySelectorAll('.streaming-code-placeholder');

  // If the number of placeholders is different, there are structural changes
  if (newPlaceholders.length !== existingPlaceholders.length) {
    return true;
  }

  // Compare placeholder attributes for significant changes
  for (let i = 0; i < newPlaceholders.length; i++) {
    const newPlaceholder = newPlaceholders[i] as HTMLElement;
    const existingPlaceholder = existingPlaceholders[i] as HTMLElement;

    // Check if language or completion status changed
    if (
      newPlaceholder.dataset.language !== existingPlaceholder.dataset.language ||
      newPlaceholder.dataset.isComplete !== existingPlaceholder.dataset.isComplete
    ) {
      return true;
    }
  }

  return false;
}

// Update existing streaming code components without remounting
function updateStreamingCodeComponents(
  container: HTMLElement,
  isStreaming: boolean,
  mdText: string = '',
) {
  const placeholders = container.querySelectorAll('.streaming-code-placeholder');

  // Extract current code blocks from the full markdown text
  const codeBlocks = extractCodeBlocks(mdText);

  placeholders.forEach((placeholder, index) => {
    const element = placeholder as HTMLElement;
    const component = (element as any).__streamingCodeComponent;

    if (component && index < codeBlocks.length) {
      try {
        const block = codeBlocks[index];
        const code = block.code;
        const language = block.language;
        const isComplete = block.isComplete;
        const streaming = isStreaming && !isComplete;

        // Update the data attributes on the element
        element.dataset.code = encodeURIComponent(code);
        element.dataset.language = language;
        element.dataset.streaming = streaming.toString();
        element.dataset.isComplete = isComplete.toString();

        // Update component props using Svelte 5 API
        // In Svelte 5, we use the $$set method if available
        if (component.$$set) {
          component.$$set({
            code,
            language,
            isStreaming: streaming,
            isComplete,
          });
        }
      } catch (error) {
        console.error('Failed to update StreamingCodeBlock:', error);
      }
    }
  });
}

// Finalize streaming - convert streaming components to final state
export function finalizeMarkdownStreaming(ctx: BubbleCtx): BubbleCtx {
  if (ctx.mdContentEl) {
    // Update all streaming code components to completed state
    const placeholders = ctx.mdContentEl.querySelectorAll('.streaming-code-placeholder');

    placeholders.forEach((placeholder) => {
      const element = placeholder as HTMLElement;
      const component = (element as any).__streamingCodeComponent;

      if (component) {
        // Update component props to stop streaming
        try {
          const finalCode = decodeURIComponent(element.dataset.code || '');
          // Use Svelte 5 API for updating props
          if (component.$$set) {
            component.$$set({ code: finalCode, isStreaming: false, isComplete: true });
          }
        } catch (error) {
          console.error('Failed to update StreamingCodeBlock:', error);
        }
      }
    });
  }

  return ctx;
}
