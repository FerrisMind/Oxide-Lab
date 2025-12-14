import type { StreamSegment } from './render_types.js';
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
import { createExternalViewButton as _createExternalViewButton } from '../external-controls.js';

export const getAssistantBubbleEl = _getAssistantBubbleEl;
export const registerAssistantBubble = _registerAssistantBubble;

export function appendSegments(
  index: number,
  bubble: HTMLDivElement,
  segments: StreamSegment[],
  isStreaming: boolean = true,
) {
  let ctx = (bubbleCtxs.get(index) ?? {
    mdEl: null,
    mdContentEl: null,
    mdRawEl: null,
    mdText: '',
    lastKind: null,
    externalButton: null,
  }) as any;

  for (const seg of segments) {
    // #region agent log
    if (seg.kind === 'text' && (seg.data.includes('<think') || seg.data.includes('&lt;think'))) {
      fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run-pre-fix',
          hypothesisId: 'H9',
          location: 'render_impl.ts:appendSegments',
          message: 'incoming text segment with think',
          data: { index, isStreaming, snippet: seg.data.slice(0, 160) },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
    }
    if (seg.kind === 'html' && (seg.data.includes('<think') || seg.data.includes('&lt;think'))) {
      fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run-pre-fix',
          hypothesisId: 'H9',
          location: 'render_impl.ts:appendSegments:html',
          message: 'incoming html segment with think',
          data: { index, isStreaming, snippet: seg.data.slice(0, 160) },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
    }
    // #endregion

    if (seg.kind === 'html') {
      bubble.insertAdjacentHTML('beforeend', seg.data);
      ctx.lastKind = 'html';
    } else {
      ctx = ensureMarkdownContainer(ctx, bubble);
      ctx = appendMarkdownText(ctx, seg.data, isStreaming);
      ctx.lastKind = 'text';

      // Внешняя кнопка теперь создается в InferenceMetricsDisplay компоненте
      // if (ctx.mdEl && !ctx.externalButton) {
      //   const messageEl = bubble.parentElement;
      //   if (messageEl) {
      //     ctx.externalButton = createExternalViewButton(messageEl, ctx.mdEl);
      //   }
      // }
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
