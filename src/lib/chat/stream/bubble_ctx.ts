import { unmount } from 'svelte';

export type StreamSegment = { kind: 'html' | 'text'; data: string };

export type BubbleCtx = {
  mdEl: HTMLElement | null;
  mdContentEl: HTMLElement | null;
  mdRawEl: HTMLElement | null;
  mdControlsEl: HTMLElement | null;
  mdToggleBtn: HTMLButtonElement | null;
  mdEyeHost: HTMLElement | null;
  mdEyeIcon: any | null;
  mdText: string;
  lastKind: 'html' | 'text' | null;
  thinkKey: string | null;
  thinkBlock: HTMLDivElement | null;
  thinkComponent: any | null;
  thinkExpanded: boolean;
  inThink: boolean;
  externalButton?: HTMLElement | null;
};

export const _assistantBubbleEls = new Map<number, HTMLDivElement>();
export const bubbleCtxs = new Map<number, BubbleCtx>();

export function getAssistantBubbleEl(index: number): HTMLDivElement | undefined {
  return _assistantBubbleEls.get(index);
}

export function registerAssistantBubble(node: HTMLDivElement, params: { index: number }) {
  _assistantBubbleEls.set(params.index, node);
  bubbleCtxs.set(params.index, {
    mdEl: null,
    mdContentEl: null,
    mdRawEl: null,
    mdControlsEl: null,
    mdToggleBtn: null,
    mdEyeHost: null,
    mdEyeIcon: null,
    mdText: '',
    lastKind: null,
    thinkKey: null,
    thinkBlock: null,
    thinkComponent: null,
    thinkExpanded: false,
    inThink: false,
    externalButton: null,
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
          mdEl: null,
          mdContentEl: null,
          mdRawEl: null,
          mdControlsEl: null,
          mdToggleBtn: null,
          mdEyeHost: null,
          mdEyeIcon: null,
          mdText: '',
          lastKind: null,
          thinkKey: null,
          thinkBlock: null,
          thinkComponent: null,
          thinkExpanded: false,
          inThink: false,
          externalButton: null,
        },
      );
      bubbleCtxs.delete(params.index);
    },
    destroy() {
      const ctx = bubbleCtxs.get(params.index);
      if (ctx?.mdEyeIcon) {
        try {
          unmount(ctx.mdEyeIcon);
        } catch {}
      }
      node.removeEventListener('scroll', onScroll as any);
      _assistantBubbleEls.delete(params.index);
      bubbleCtxs.delete(params.index);
    },
  };
}
