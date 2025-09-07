import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { createStreamParser } from '$lib/chat/parser';
import { appendSegments, finalizeStreaming, getAssistantBubbleEl } from '$lib/chat/stream_render';
import type { ChatControllerCtx } from './types';

export function createStreamListener(ctx: ChatControllerCtx) {
  let unlisten: UnlistenFn | null = null;
  let streamBuf = '';
  let rafId: number | null = null;
  let isStreaming = false;
  const streamParser = createStreamParser();

  function scheduleFlush() {
    if (rafId !== null) return;
    rafId = requestAnimationFrame(() => {
      rafId = null;
      const { segments, remainder } = streamParser.parse(streamBuf);
      const msgs = ctx.messages;
      const last = msgs[msgs.length - 1];
      if (last && last.role === 'assistant') {
        const idx = msgs.length - 1;
        const el = getAssistantBubbleEl(idx);
        if (el) appendSegments(idx, el, segments, isStreaming);
        const onlyText = segments
          .filter((s) => s.kind === 'text')
          .map((s) => s.data)
          .join('');
        if (onlyText) {
          last.content += onlyText;
          ctx.messages = msgs;
        }
        queueMicrotask(() => {
          const container = ctx.messagesEl;
          if (!container) return;
          const threshold = 32;
          const atBottom =
            container.scrollTop + container.clientHeight >= container.scrollHeight - threshold;
          if (atBottom) container.scrollTop = container.scrollHeight;
        });
      }
      streamBuf = remainder;
    });
  }

  async function ensureListener() {
    if (!unlisten) {
      unlisten = await listen<string>('token', (event) => {
        const token = event.payload ?? '';
        if (token === '') {
          // Start of new stream
          const msgs = ctx.messages;
          const last = msgs[msgs.length - 1];
          if (!last || last.role !== 'assistant' || last.content !== '') {
            msgs.push({ role: 'assistant', content: '', html: '' } as any);
            ctx.messages = msgs;
          }
          streamBuf = '';
          streamParser.reset();
          isStreaming = true;
          return;
        }
        
        if (token === '[DONE]') {
          // End of stream
          isStreaming = false;
          const msgs = ctx.messages;
          if (msgs.length > 0) {
            const idx = msgs.length - 1;
            finalizeStreaming(idx);
          }
          return;
        }
        
        streamBuf += token;
        scheduleFlush();
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
