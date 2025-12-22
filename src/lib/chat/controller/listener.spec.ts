import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createStreamListener } from './listener';
import { chatHistory } from '$lib/stores/chat-history';
import { get as _get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

// Mocks
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('$lib/stores/chat-history', () => ({
  chatHistory: {
    subscribe: vi.fn((fn) => {
      fn({ currentSessionId: 'test-session-id' }); // Fake state
      return () => { };
    }),
    saveAssistantMessage: vi.fn(),
  },
}));

vi.mock('svelte/store', () => ({
  get: vi.fn((_store) => {
    // Return fake state
    return { currentSessionId: 'test-session-id' };
  }),
}));

// Mock stream_render imports to avoid DOM dependency issues in basic logic test
vi.mock('$lib/chat/stream_render', () => ({
  appendSegments: vi.fn(),
  finalizeStreaming: vi.fn(),
  getAssistantBubbleEl: vi.fn(),
}));

describe('Listener Controller', () => {
  let mockUnlisten: any;
  let listenerCallback: (payload: any) => Promise<void>;

  beforeEach(() => {
    vi.clearAllMocks();
    mockUnlisten = vi.fn();
    (listen as any).mockImplementation(async (event: string, cb: any) => {
      listenerCallback = cb;
      return mockUnlisten;
    });
  });

  it('should call saveAssistantMessage with accumulated content on [DONE]', async () => {
    const ctx: any = {
      messages: [],
      // Note: messagesEl removed - scroll is now handled by ChatContainerContext
    };
    const listener = createStreamListener(ctx);
    await listener.ensureListener();

    expect(listen).toHaveBeenCalled();

    // Simulate stream start
    await listenerCallback({ payload: '' });

    // Simulate tokens
    await listenerCallback({ payload: 'Hello ' });
    await listenerCallback({ payload: 'World' });

    // Simulate end
    // Trigger save logic
    await listenerCallback({ payload: '[DONE]' });

    // Verify saveAssistantMessage call
    // The listener writes to ctx.messages[last].content via flushStream
    // So we expect saveAssistantMessage to be called with 'Hello World'
    expect(chatHistory.saveAssistantMessage).toHaveBeenCalledWith('test-session-id', 'Hello World');
  });
});
