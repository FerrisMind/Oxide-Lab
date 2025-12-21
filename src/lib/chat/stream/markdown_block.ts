import { mount, unmount } from 'svelte';
import { get } from 'svelte/store';
import { renderMarkdownToSafeHtml, lastAutoClosedThink } from '$lib/chat/markdown';
import { THINK_CLOSE_TOKEN } from '$lib/chat/parser/constants';
// CodeMirror renderer removed — using Shiki-based StreamingCodeBlock for highlighting
import { enableExternalLinks } from '$lib/chat/external-links';
import { StreamingCodeBlock } from '$lib/components/streaming-code';
import ThinkingAccordion from '../components/ThinkingAccordion.svelte';
import { t } from '$lib/i18n';

const thinkState = new Map<string, boolean>();
const THINK_LOADING_LABEL = () => get(t)('chat.thinking.loading');
const THINK_READY_LABEL = () => get(t)('chat.thinking.ready');
import type { BubbleCtx } from './bubble_ctx';

function _hasMarkdownFeatures(text: string): boolean {
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
      cleanupThinkingComponents(ctx.mdContentEl);
    }

    ctx.mdEl = document.createElement('div');
    ctx.mdEl.className = 'md-stream';

    // Удаляем старую логику md-controls - кнопка будет создаваться отдельно

    // content containers
    const contentEl = document.createElement('div');
    contentEl.className = 'md-content';
    const rawEl = document.createElement('pre');
    rawEl.className = 'md-raw';

    // assemble - только контент
    ctx.mdEl.appendChild(contentEl);
    ctx.mdEl.appendChild(rawEl);
    bubble.appendChild(ctx.mdEl);
    ctx.mdContentEl = contentEl;
    ctx.mdRawEl = rawEl;
    ctx.mdText = '';
  }
  return ctx;
}

function cleanupThinkingComponents(container: HTMLElement) {
  const placeholders = container.querySelectorAll('.thinking-placeholder');
  placeholders.forEach((placeholder) => {
    const element = placeholder as HTMLElement & { __thinkComponent?: any };
    const component = element.__thinkComponent;
    if (component) {
      try {
        unmount(component);
      } catch {}
      delete element.__thinkComponent;
    }
  });
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
        } catch {
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

// Mount thinking accordions for sanitized <think> blocks
function mountThinkingComponents(container: HTMLElement) {
  const placeholders = container.querySelectorAll('.thinking-placeholder');

  placeholders.forEach((placeholder) => {
    const element = placeholder as HTMLElement & { __thinkComponent?: any };
    const mountTarget = (element.querySelector('.thinking-mount') as HTMLElement | null) ?? element;
    const contentEl = element.querySelector('[data-think-slot]') as HTMLElement | null;
    const innerHtml = contentEl?.innerHTML ?? '';
    const key = element.dataset.thinkId || 'thinking';
    const streaming = element.dataset.streaming === 'true';

    // remove stash content to avoid double render
    if (contentEl) {
      contentEl.remove();
    }

    try {
      // reuse if already mounted
      if (element.__thinkComponent) {
        const existing = element.__thinkComponent;
        existing.resetContent?.();
        existing.appendHtml?.(innerHtml);
        const stored = thinkState.has(key) ? thinkState.get(key)! : false;
        const open = streaming ? true : stored;
        existing.setOpen?.(open);
        existing.setStreaming?.(streaming);

        return;
      }

      const component = mount(ThinkingAccordion, {
        target: mountTarget,
        props: {
          value: key,
          labelLoading: THINK_LOADING_LABEL(),
          labelReady: THINK_READY_LABEL(),
          streaming,
          autoCollapse: true,
          onToggle: (open: boolean) => thinkState.set(key, open),
        },
      });

      component.resetContent?.();
      component.appendHtml?.(innerHtml);

      const stored = thinkState.has(key) ? thinkState.get(key)! : false;
      const open = streaming ? true : stored;
      component.setOpen?.(open);
      component.setStreaming?.(streaming);

      element.__thinkComponent = component;
    } catch (error) {
      console.error('Failed to mount ThinkingAccordion:', error);
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
    let newContent = renderMarkdownWithStreamingCode(ctx.mdText, isStreaming);

    let needsFullRender = hasStreamingCodeBlocksChanged(ctx.mdContentEl, newContent);
    let openCount = (ctx.mdText.match(/<think>/g) || []).length;
    let closeCount = (ctx.mdText.match(/<\/think>/g) || []).length;

    // Если markdown-рендер автозакрыл think, а сырой поток всё ещё без закрытия,
    // сохраняем закрывающий тег в буфере, чтобы последующий текст не попадал внутрь.
    if (lastAutoClosedThink && openCount > closeCount) {
      ctx.mdText += THINK_CLOSE_TOKEN;
      closeCount += 1;
      newContent = renderMarkdownWithStreamingCode(ctx.mdText, isStreaming);
      needsFullRender = true;
    }

    if (needsFullRender) {
      cleanupStreamingCodeComponents(ctx.mdContentEl);
      cleanupThinkingComponents(ctx.mdContentEl);

      ctx.mdContentEl.innerHTML = newContent;

      enableExternalLinks(ctx.mdContentEl);
      mountStreamingCodeComponents(ctx.mdContentEl, isStreaming);
      mountThinkingComponents(ctx.mdContentEl);
    } else {
      updateStreamingCodeComponents(ctx.mdContentEl, isStreaming, ctx.mdText);
    }

    // No CodeMirror rendering anymore — Shiki handles highlighting in streaming components.
  }

  if (ctx.mdRawEl) {
    ctx.mdRawEl.textContent = ctx.mdText;
  }

  // Управление кнопкой теперь происходит отдельно

  ctx.lastKind = 'text';
  return ctx;
}

// Check if the structure of code blocks has changed
function normalizePlaceholderHtml(html: string): string {
  return html
    .replace(/data-code="[^"]*"/g, 'data-code=""')
    .replace(/data-code='[^']*'/g, "data-code=''")
    .replace(/data-streaming="[^"]*"/g, 'data-streaming=""')
    .replace(/data-streaming='[^']*'/g, "data-streaming=''")
    .replace(/data-is-complete="[^"]*"/g, 'data-is-complete=""')
    .replace(/data-is-complete='[^']*'/g, "data-is-complete=''")
    .replace(/data-language="[^"]*"/g, 'data-language=""')
    .replace(/data-language='[^']*'/g, "data-language=''");
}

function hasStreamingCodeBlocksChanged(container: HTMLElement, newContent: string): boolean {
  const existingHtml = container.innerHTML;
  if (!existingHtml) {
    return true;
  }

  if (existingHtml === newContent) {
    return false;
  }

  // Parse the new content to check for structural changes
  const tempDiv = document.createElement('div');
  tempDiv.innerHTML = newContent;
  const newPlaceholders = tempDiv.querySelectorAll('.streaming-code-placeholder');
  const existingPlaceholders = container.querySelectorAll('.streaming-code-placeholder');

  if (newPlaceholders.length !== existingPlaceholders.length) {
    return true;
  }

  for (let i = 0; i < newPlaceholders.length; i++) {
    const newPlaceholder = newPlaceholders[i] as HTMLElement;
    const existingPlaceholder = existingPlaceholders[i] as HTMLElement;

    if (
      newPlaceholder.dataset.language !== existingPlaceholder.dataset.language ||
      newPlaceholder.dataset.isComplete !== existingPlaceholder.dataset.isComplete
    ) {
      return true;
    }
  }

  // Detect content changes outside streaming code placeholders
  const normalizedExisting = normalizePlaceholderHtml(existingHtml);
  const normalizedNew = normalizePlaceholderHtml(newContent);

  return normalizedExisting !== normalizedNew;
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
