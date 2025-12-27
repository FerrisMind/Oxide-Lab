/**
 * Stream Listener
 * 
 * Handles token stream events from the Tauri backend.
 */

import { createStreamParser } from '$lib/chat/parser';
import { appendSegments, finalizeStreaming, getAssistantBubbleEl } from '$lib/chat/stream_render';
import { get } from 'svelte/store';
import { chatHistory } from '$lib/stores/chat-history';
import type { ChatControllerCtx } from './types';

export function createStreamListener(ctx: ChatControllerCtx) {
    let unlisten: (() => void) | null = null;
    let streamBuf = '';
    let rafId: number | null = null;
    let isStreaming = false;
    const streamParser = createStreamParser();

    function flushStream(streamingFlag: boolean) {
        const { segments, remainder } = streamParser.parse(streamBuf);
        streamBuf = remainder;
        if (segments.length === 0) return;

        // Debug: log segments
        console.log('[flushStream] segments:', segments.map(s => ({ kind: s.kind, len: s.data.length })));

        const msgs = ctx.messages;
        const last = msgs[msgs.length - 1];
        if (last && last.role === 'assistant') {
            const idx = msgs.length - 1;
            const el = getAssistantBubbleEl(idx);

            if (el) appendSegments(idx, el, segments, streamingFlag);

            // Process segments for content and thinking
            let hasUpdates = false;
            for (const segment of segments) {
                if (segment.kind === 'text') {
                    last.content += segment.data;
                    hasUpdates = true;
                } else if (segment.kind === 'think_start') {
                    console.log('[flushStream] think_start - setting isThinking=true');
                    last.isThinking = true;
                    if (!last.thinking) last.thinking = '';
                    hasUpdates = true;
                } else if (segment.kind === 'think_content') {
                    console.log('[flushStream] think_content - adding', segment.data.length, 'chars');
                    if (!last.thinking) last.thinking = '';
                    last.thinking += segment.data;
                    hasUpdates = true;
                } else if (segment.kind === 'think_end') {
                    console.log('[flushStream] think_end - setting isThinking=false, total thinking:', last.thinking?.length);
                    last.isThinking = false;
                    hasUpdates = true;
                }
            }

            if (hasUpdates) {
                ctx.messages = msgs;
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
            // TODO: Integrate with Tauri backend
            // Command: listen('token', callback)
            const { listen } = await import('@tauri-apps/api/event');

            unlisten = await listen<string>('token', async (event) => {
                const token = event.payload ?? '';
                if (token === '') {
                    // Start of new stream
                    const msgs = ctx.messages;
                    const last = msgs[msgs.length - 1];

                    // Always start in think mode for Qwen/thinking models
                    // Parser will detect </think> and switch to content mode
                    const startsInThinkMode = true;

                    if (!last || last.role !== 'assistant' || last.content !== '') {
                        msgs.push({
                            role: 'assistant',
                            content: '',
                            html: '',
                            thinking: '',
                            isThinking: startsInThinkMode
                        });
                        ctx.messages = msgs;
                    } else {
                        // Message already exists, update thinking state
                        last.thinking = '';
                        last.isThinking = startsInThinkMode;
                        ctx.messages = msgs;
                    }

                    if (rafId !== null) {
                        cancelAnimationFrame(rafId);
                        rafId = null;
                    }
                    streamBuf = '';
                    streamParser.reset();
                    streamParser.setInThinkBlock(true);

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
                            const last = msgs[msgs.length - 1];
                            if (last && last.role === 'assistant') {
                                await chatHistory.saveAssistantMessage(state.currentSessionId, last.content);
                            }
                        }
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
            } catch { /* ignore */ }
            unlisten = null;
        }
        if (rafId !== null) {
            cancelAnimationFrame(rafId);
            rafId = null;
        }
    }

    return { ensureListener, destroy };
}
