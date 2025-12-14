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
    // #region agent log
    void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        sessionId: 'debug-session',
        runId: 'run-pre-fix',
        hypothesisId: 'H11',
        location: 'listener.ts:flushStream:before-parse',
        message: 'before parse',
        data: { streamingFlag, streamBufLen: streamBuf.length, snippet: streamBuf.slice(0, 120) },
        timestamp: Date.now(),
      }),
    }).catch(() => {});
    // #endregion

    const { segments, remainder } = streamParser.parse(streamBuf);
    streamBuf = remainder;
    if (segments.length === 0) return;

    // #region agent log
    const hasThinkSeg = segments.some(
      (s) => s.data && (s.data.includes('<think') || s.data.includes('&lt;think')),
    );
    void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        sessionId: 'debug-session',
        runId: 'run-pre-fix',
        hypothesisId: 'H11',
        location: 'listener.ts:flushStream:after-parse',
        message: 'after parse',
        data: {
          streamingFlag,
          segmentsCount: segments.length,
          hasThinkSeg,
          remainderLen: remainder.length,
          firstSegKind: segments[0]?.kind ?? null,
          firstSegSnippet: segments[0]?.data?.slice(0, 120) ?? null,
        },
        timestamp: Date.now(),
      }),
    }).catch(() => {});
    // #endregion

    const msgs = ctx.messages;
    const last = msgs[msgs.length - 1];
    if (last && last.role === 'assistant') {
      const idx = msgs.length - 1;
      const el = getAssistantBubbleEl(idx);
      // Capture whether the user was pinned to bottom BEFORE DOM updates
      const container = ctx.messagesEl;
      const wasPinnedToBottom =
        !!container && container.scrollTop + container.clientHeight >= container.scrollHeight - 1;

      if (el) appendSegments(idx, el, segments, streamingFlag);
      const onlyText = segments
        .filter((s) => s.kind === 'text')
        .map((s) => s.data)
        .join('');
      if (onlyText) {
        last.content += onlyText;
        ctx.messages = msgs;
      }
      // Scroll after DOM commit; use one or two rAFs to account for async mounts (e.g., streaming code blocks)
      if (wasPinnedToBottom) {
        requestAnimationFrame(() => {
          const c1 = ctx.messagesEl;
          if (c1) c1.scrollTop = c1.scrollHeight;
          // Schedule a second frame in case nested components expand after first paint
          requestAnimationFrame(() => {
            const c2 = ctx.messagesEl;
            if (c2) c2.scrollTop = c2.scrollHeight;
          });
        });
      }
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
                // #region agent log
                void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
                  method: 'POST',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({
                    sessionId: 'debug-session',
                    runId: 'run1',
                    hypothesisId: 'H1',
                    location: 'listener.ts:[DONE]',
                    message: 'saving assistant message',
                    data: {
                      sessionId: state.currentSessionId,
                      contentLength: last.content?.length ?? 0,
                      hasContent: !!last.content,
                    },
                    timestamp: Date.now(),
                  }),
                }).catch(() => {});
                // #endregion
                await chatHistory.saveAssistantMessage(state.currentSessionId, last.content);
              }
            }

            // Ensure proper scroll position after generation completes
            const container = ctx.messagesEl;
            if (container) {
              // Always scroll to bottom when generation completes
              requestAnimationFrame(() => {
                container.scrollTop = container.scrollHeight;
                // Schedule a second frame to ensure scroll position is properly set
                requestAnimationFrame(() => {
                  container.scrollTop = container.scrollHeight;
                });
              });
            }
          }
          return;
        }

        // #region agent log
        if (token.includes('<think') || token.includes('&lt;think')) {
          void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              sessionId: 'debug-session',
              runId: 'run-pre-fix',
              hypothesisId: 'H10',
              location: 'listener.ts:token',
              message: 'raw token with think',
              data: { snippet: token.slice(0, 200) },
              timestamp: Date.now(),
            }),
          }).catch(() => {});
        }
        // #endregion

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
      } catch {}
      unlisten = null;
    }
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  }

  return { ensureListener, destroy };
}
