import { unmount } from 'svelte';
import { cleanupRenderer } from '$lib/chat/codemirror-renderer';

export type StreamSegment = { kind: 'html' | 'text'; data: string };

export type BubbleCtx = {
  inThink: boolean;
  thinkBlock: HTMLElement | null;
  thinkCardEl: HTMLElement | null;
  thinkBody: HTMLElement | null;
  thinkToggleBtn: HTMLButtonElement | null;
  thinkLoaderEl: HTMLElement | null;
  thinkChevronEl: HTMLElement | null;
  thinkLabelEl: HTMLElement | null;
  thinkKey: string | null;
  thinkExpanded: boolean;
  thinkToggleHandler: ((event: Event) => void) | null;
  mdEl: HTMLElement | null;
  mdContentEl: HTMLElement | null;
  mdRawEl: HTMLElement | null;
  mdControlsEl: HTMLElement | null;
  mdToggleBtn: HTMLButtonElement | null;
  mdEyeHost: HTMLElement | null;
  mdEyeIcon: any | null;
  mdText: string;
  lastKind: 'html' | 'text' | null;
  codeMirrorWatching: boolean;
};

export const _assistantBubbleEls = new Map<number, HTMLDivElement>();
export const bubbleCtxs = new Map<number, BubbleCtx>();

export function getAssistantBubbleEl(index: number): HTMLDivElement | undefined {
  return _assistantBubbleEls.get(index);
}

export function registerAssistantBubble(node: HTMLDivElement, params: { index: number }) {
  _assistantBubbleEls.set(params.index, node);
  bubbleCtxs.set(params.index, {
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
  });

  const onScroll = () => {
    // хук под автоскролл (оставлен пустым)
  };
  node.addEventListener('scroll', onScroll, { passive: true });

  return {
    update(newParams: { index: number }) {
      _assistantBubbleEls.delete(params.index);
      _assistantBubbleEls.set(newParams.index, node);
      const prev = bubbleCtxs.get(params.index);
      bubbleCtxs.set(
        newParams.index,
        prev ?? {
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
        },
      );
      bubbleCtxs.delete(params.index);
    },
    destroy() {
      const ctx = bubbleCtxs.get(params.index);
      if (ctx?.thinkToggleBtn && ctx?.thinkToggleHandler) {
        try {
          ctx.thinkToggleBtn.removeEventListener('click', ctx.thinkToggleHandler);
        } catch {}
        ctx.thinkToggleHandler = null;
      }
      if (ctx?.mdEyeIcon) {
        try {
          unmount(ctx.mdEyeIcon);
        } catch {}
      }
      // Cleanup CodeMirror if it was watching this bubble
      if (ctx?.codeMirrorWatching && ctx?.mdContentEl) {
        try {
          cleanupRenderer(ctx.mdContentEl);
        } catch {}
      }
      node.removeEventListener('scroll', onScroll as any);
      _assistantBubbleEls.delete(params.index);
      bubbleCtxs.delete(params.index);
    },
  };
}
