import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { chatHistory } from './chat-history';

// Mock Tauri SQL Plugin
const { mockLoad, mockExecute } = vi.hoisted(() => {
  const execute = vi.fn();
  const load = vi.fn().mockResolvedValue({
    execute: execute,
    select: vi.fn().mockResolvedValue([]),
  });
  return { mockLoad: load, mockExecute: execute };
});

vi.mock('@tauri-apps/plugin-sql', () => ({
  default: {
    load: mockLoad,
  },
}));

// Mock localStorage
const mockRemoveItem = vi.fn();
const mockGetItem = vi.fn();
const mockSetItem = vi.fn();

vi.stubGlobal('localStorage', {
  removeItem: mockRemoveItem,
  getItem: mockGetItem,
  setItem: mockSetItem,
});

describe('chatHistory Store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('saveAssistantMessage', () => {
    it('should insert message into DB and update store', async () => {
      // 1. Init DB
      await chatHistory.init();

      // 2. Mock createSession to return a session
      mockExecute.mockResolvedValueOnce({ lastInsertId: 123 }); // for createSession
      await chatHistory.createSession('model/path', 'repo/id');

      // Get the session ID from the store
      let state = get(chatHistory);
      const sessionId = state.currentSessionId;
      expect(sessionId).toBeTruthy();

      // 3. Call saveAssistantMessage
      const content = 'Test content';
      mockExecute.mockResolvedValueOnce({}); // for saveAssistantMessage insert
      mockExecute.mockResolvedValueOnce({}); // for update session timestamp

      // We need to use `as any` if saveAssistantMessage is not public in type definition
      // but it should be based on our edit.
      await chatHistory.saveAssistantMessage(sessionId!, content);

      // Verify DB calls
      // Expected: INSERT INTO messages ...
      // Expected: UPDATE sessions ...
      expect(mockExecute).toHaveBeenCalledWith(
        expect.stringContaining('INSERT INTO messages'),
        expect.arrayContaining([sessionId, 'assistant', content, expect.any(Number)]),
      );
    });
  });
});
