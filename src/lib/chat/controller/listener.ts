import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { createStreamParser } from '$lib/chat/parser';
import { appendSegments, finalizeStreaming, getAssistantBubbleEl } from '$lib/chat/stream_render';
import { get } from 'svelte/store';
import { chatHistory } from '$lib/stores/chat-history';
import type { ChatControllerCtx } from './types';

export function createStreamListener(ctx: ChatControllerCtx) {
  let unlisten: UnlistenFn | null = null;
  let streamBuf = '';
  let rafId: number | null = null;
  let isStreaming = false;
  const streamParser = createStreamParser();

  function flushStream(streamingFlag: boolean) {
    const { segments, remainder } = streamParser.parse(streamBuf);
    streamBuf = remainder;
    if (segments.length === 0) return;

    const msgs = ctx.messages;
    const last = msgs[msgs.length - 1];
    if (last && last.role === 'assistant') {
      const idx = msgs.length - 1;
      const el = getAssistantBubbleEl(idx);

      if (el) appendSegments(idx, el, segments, streamingFlag);
      const onlyText = segments
        .filter((s) => s.kind === 'text')
        .map((s) => s.data)
        .join('');
      if (onlyText) {
        last.content += onlyText;
        ctx.messages = msgs;
      }
      // Note: Auto-scroll is now handled by ChatContainerContext via MutationObserver
    }
  }

  function scheduleFlush() {
    if (rafId !== null) return;
    rafId = requestAnimationFrame(() => {
      rafId = null;
      flushStream(isStreaming);
    });
  }

  async function ensureListener() {
    if (!unlisten) {
      unlisten = await listen<string>('token', async (event) => {
        const token = event.payload ?? '';
        if (token === '') {
          // Start of new stream
          const msgs = ctx.messages;
          const last = msgs[msgs.length - 1];
          if (!last || last.role !== 'assistant' || last.content !== '') {
            msgs.push({ role: 'assistant', content: '', html: '' } as any);
            ctx.messages = msgs;
          }
          if (rafId !== null) {
            cancelAnimationFrame(rafId);
            rafId = null;
          }
          streamBuf = '';
          streamParser.reset();
          isStreaming = true;
          return;
        }

        if (token === '[DONE]') {
          // End of stream
          if (rafId !== null) {
            cancelAnimationFrame(rafId);
            rafId = null;
          }
          isStreaming = false;
          flushStream(false);
          streamParser.reset();
          streamBuf = '';
          const msgs = ctx.messages;
          if (msgs.length > 0) {
            const idx = msgs.length - 1;
            finalizeStreaming(idx);

            // Persist the complete assistant message to SQLite
            const state = get(chatHistory);
            if (state.currentSessionId) {
              const msgs = ctx.messages;
              const last = msgs[msgs.length - 1];
              if (last && last.role === 'assistant') {
                await chatHistory.saveAssistantMessage(state.currentSessionId, last.content);
              }
            }
            // Note: Auto-scroll is handled by ChatContainerContext via MutationObserver
          }
          return;
        }

        streamBuf += token;
        scheduleFlush();

        // Optimistic update for UI stores (debounced slightly by scheduleFlush logic,
        // but here we might want to update the store less frequently or just rely on flushStream)
        // flushStream updates ctx.messages[last].content.
        // We should sync that to chatHistory store optimistically so UI bound to store updates too
        // But ctx.messages is currently the source of truth for the chat view.
        // So maybe we don't need updateLastMessageOptimistic if the view uses ctx.messages?
        // Let's check if the view uses ctx.messages or chatHistory store.
        // Typically current view uses ctx.messages array locally.
        // So persistLastMessage at the end is the critical part for persistence.
        // I will add persistLastMessage and skip optimistic updates to store to avoid double reactivity overhead
        // unless other components need to see the stream in real-time from the store.
        // Given complexity, saving at the end is the requirement "messages will be saved... after generation completes".
        // So just the [DONE] block change is sufficient.
      });
    }
  }

  function destroy() {
    if (unlisten) {
      try {
        unlisten();
      } catch { }
      unlisten = null;
    }
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  }

  return { ensureListener, destroy };
}
