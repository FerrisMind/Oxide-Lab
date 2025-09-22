import type { StreamSegment } from './render_types.js';
import { appendThinkAwareHtml } from './think_html.js';
import {
  _assistantBubbleEls,
  bubbleCtxs,
  getAssistantBubbleEl as _getAssistantBubbleEl,
  registerAssistantBubble as _registerAssistantBubble,
} from './bubble_ctx.js';
import {
  ensureMarkdownContainer,
  appendMarkdownText,
  finalizeMarkdownStreaming,
} from './markdown_block.js';

export const getAssistantBubbleEl = _getAssistantBubbleEl;
export const registerAssistantBubble = _registerAssistantBubble;

export function appendSegments(
  index: number,
  bubble: HTMLDivElement,
  segments: StreamSegment[],
  isStreaming: boolean = true,
) {
  let ctx = (bubbleCtxs.get(index) ?? {
    inThink: false,
    thinkBlock: null,
    thinkCardEl: null,
    thinkBody: null,
    thinkToggleBtn: null,
    thinkLoaderEl: null,
    thinkChevronEl: null,
    thinkLabelEl: null,
    thinkKey: null,
    thinkExpanded: false,
    thinkToggleHandler: null,
    mdEl: null,
    mdContentEl: null,
    mdRawEl: null,
    mdControlsEl: null,
    mdToggleBtn: null,
    mdEyeHost: null,
    mdEyeIcon: null,
    mdText: '',
    lastKind: null,
    codeMirrorWatching: false,
  }) as any;

  for (const seg of segments) {
    if (seg.kind === 'html') {
      appendThinkAwareHtml(ctx, bubble, seg.data);
      ctx.lastKind = 'html';
    } else {
      ctx = ensureMarkdownContainer(ctx, bubble);
      ctx = appendMarkdownText(ctx, seg.data, isStreaming);
      ctx.lastKind = 'text';
    }
  }
  bubbleCtxs.set(index, ctx);
}

export function finalizeStreaming(index: number) {
  let ctx = bubbleCtxs.get(index);
  if (ctx) {
    ctx = finalizeMarkdownStreaming(ctx);
    bubbleCtxs.set(index, ctx);
  }
}

// Rebuild assistant bubble UI from full plain-text content (e.g., after remount)
// rehydrateAssistantBubble removed: Chat remains mounted; no rehydration required
