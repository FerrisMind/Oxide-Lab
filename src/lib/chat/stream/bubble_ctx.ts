import { unmount } from "svelte";
import { getCodeMirrorRenderer } from "$lib/chat/codemirror-renderer";

export type StreamSegment = { kind: "html" | "text"; data: string };

export type BubbleCtx = {
  inThink: boolean;
  thinkPre: HTMLElement | null;
  thinkSummary: HTMLElement | null;
  thinkCaretHost: HTMLElement | null;
  thinkBrainHost: HTMLElement | null;
  thinkCaretIcon: any | null;
  thinkBrainIcon: any | null;
  mdEl: HTMLElement | null;
  mdContentEl: HTMLElement | null;
  mdRawEl: HTMLElement | null;
  mdControlsEl: HTMLElement | null;
  mdToggleBtn: HTMLButtonElement | null;
  mdEyeHost: HTMLElement | null;
  mdEyeIcon: any | null;
  mdText: string;
  lastKind: "html" | "text" | null;
  codeMirrorWatching: boolean;
};

export const assistantBubbleEls = new Map<number, HTMLDivElement>();
export const bubbleCtxs = new Map<number, BubbleCtx>();

export function getAssistantBubbleEl(index: number): HTMLDivElement | undefined {
  return assistantBubbleEls.get(index);
}

export function registerAssistantBubble(node: HTMLDivElement, params: { index: number }) {
  assistantBubbleEls.set(params.index, node);
  bubbleCtxs.set(params.index, {
    inThink: false,
    thinkPre: null,
    thinkSummary: null,
    thinkCaretHost: null,
    thinkBrainHost: null,
    thinkCaretIcon: null,
    thinkBrainIcon: null,
    mdEl: null,
    mdContentEl: null,
    mdRawEl: null,
    mdControlsEl: null,
    mdToggleBtn: null,
    mdEyeHost: null,
    mdEyeIcon: null,
    mdText: "",
    lastKind: null,
    codeMirrorWatching: false
  });

  const onScroll = () => {
    // хук под автоскролл (оставлен пустым)
  };
  node.addEventListener("scroll", onScroll, { passive: true });

  return {
    update(newParams: { index: number }) {
      assistantBubbleEls.delete(params.index);
      assistantBubbleEls.set(newParams.index, node);
      const prev = bubbleCtxs.get(params.index);
      bubbleCtxs.set(
        newParams.index,
        prev ?? {
          inThink: false,
          thinkPre: null,
          thinkSummary: null,
          thinkCaretHost: null,
          thinkBrainHost: null,
          thinkCaretIcon: null,
          thinkBrainIcon: null,
          mdEl: null,
          mdContentEl: null,
          mdRawEl: null,
          mdControlsEl: null,
          mdToggleBtn: null,
          mdEyeHost: null,
          mdEyeIcon: null,
          mdText: "",
          lastKind: null,
          codeMirrorWatching: false
        }
      );
      bubbleCtxs.delete(params.index);
    },
    destroy() {
      const ctx = bubbleCtxs.get(params.index);
      if (ctx?.thinkCaretIcon) {
        try { unmount(ctx.thinkCaretIcon); } catch {}
      }
      if (ctx?.thinkBrainIcon) {
        try { unmount(ctx.thinkBrainIcon); } catch {}
      }
      if (ctx?.mdEyeIcon) {
        try { unmount(ctx.mdEyeIcon); } catch {}
      }
      // Cleanup CodeMirror if it was watching this bubble
      if (ctx?.codeMirrorWatching && ctx?.mdContentEl) {
        try {
          const renderer = getCodeMirrorRenderer();
          renderer.stopWatching();
        } catch {}
      }
      node.removeEventListener("scroll", onScroll as any);
      assistantBubbleEls.delete(params.index);
      bubbleCtxs.delete(params.index);
    }
  };
}


