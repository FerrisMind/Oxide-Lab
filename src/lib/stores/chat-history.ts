/**
 * Chat History Store
 * 
 * Manages chat sessions with SQLite persistence through Tauri.
 */

import { writable, derived, get } from 'svelte/store';
import type { ChatMessage } from './chat';

export type ChatSession = {
    id: string;
    title: string;
    messages: ChatMessage[];
    createdAt: number;
    updatedAt: number;
    modelPath?: string;
    repoId?: string;
};

export type ChatHistoryState = {
    sessions: ChatSession[];
    currentSessionId: string | null;
    isInitialized: boolean;
};

function createChatHistoryStore() {
    const { subscribe, update, set } = writable<ChatHistoryState>({
        sessions: [],
        currentSessionId: null,
        isInitialized: false,
    });

    return {
        subscribe,

        // TODO: Integrate with Tauri backend
        // Initialize database and migrate from localStorage
        init: async () => {
            try {
                // TODO: Integrate with Tauri backend
                // Command: Database.load('sqlite:chat_history.db')
                // Then: SELECT * FROM sessions ORDER BY updated_at DESC

                // For now, initialize with empty state
                update((s) => ({
                    ...s,
                    isInitialized: true,
                    currentSessionId: null,
                }));

            } catch (err) {
                console.error('Failed to init chat database:', err);
            }
        },

        createSession: async (modelPath?: string, repoId?: string) => {
            const id = crypto.randomUUID();
            const title = 'New chat';
            const now = Date.now();

            // TODO: Integrate with Tauri backend
            // Command: INSERT INTO sessions (id, title, model_path, repo_id, created_at, updated_at)

            update((s) => ({
                ...s,
                sessions: [
                    {
                        id,
                        title,
                        modelPath,
                        repoId,
                        createdAt: now,
                        updatedAt: now,
                        messages: [],
                    },
                    ...s.sessions,
                ],
                currentSessionId: id,
            }));

            return id;
        },

        loadSession: (sessionId: string) => {
            update((s) => ({ ...s, currentSessionId: sessionId }));
        },

        addMessage: async (message: ChatMessage, sessionId?: string) => {
            // TODO: Integrate with Tauri backend
            // Command: INSERT INTO messages (session_id, role, content, created_at)

            const now = Date.now();

            update((s) => {
                const targetSessionId = sessionId ?? s.currentSessionId;
                if (!targetSessionId) return s;

                const sessions = s.sessions.map((sess) => {
                    if (sess.id !== targetSessionId) return sess;

                    const isFirstUserMessage = sess.messages.length === 0 && message.role === 'user';
                    const titleFromMessage = isFirstUserMessage
                        ? message.content.substring(0, 50).split('\n')[0]
                        : sess.title;

                    return {
                        ...sess,
                        title: titleFromMessage ?? sess.title,
                        messages: [...sess.messages, message],
                        updatedAt: now,
                    };
                });

                return { ...s, sessions, currentSessionId: s.currentSessionId ?? targetSessionId };
            });
        },

        updateLastMessageOptimistic: (content: string) => {
            update((s) => {
                if (!s.currentSessionId) return s;
                const sessions = s.sessions.map((sess) => {
                    if (sess.id === s.currentSessionId) {
                        const msgs = [...sess.messages];
                        if (msgs.length > 0) {
                            msgs[msgs.length - 1] = { ...msgs[msgs.length - 1], content };
                        }
                        return { ...sess, messages: msgs };
                    }
                    return sess;
                });
                return { ...s, sessions };
            });
        },

        saveAssistantMessage: async (sessionId: string, content: string) => {
            // TODO: Integrate with Tauri backend
            // Command: INSERT INTO messages (session_id, role, content, created_at)
            console.log(`Saving assistant message to session ${sessionId}`);
        },

        deleteSession: async (sessionId: string) => {
            // TODO: Integrate with Tauri backend
            // Command: DELETE FROM sessions WHERE id = $1

            update((s) => {
                const sessions = s.sessions.filter((sess) => sess.id !== sessionId);
                let current = s.currentSessionId;
                if (current === sessionId) {
                    current = sessions.length > 0 ? sessions[0].id : null;
                }
                return { ...s, sessions, currentSessionId: current };
            });
        },

        renameSession: async (sessionId: string, title: string) => {
            // TODO: Integrate with Tauri backend
            // Command: UPDATE sessions SET title = $1, updated_at = $2 WHERE id = $3

            update((s) => ({
                ...s,
                sessions: s.sessions.map((sess) =>
                    sess.id === sessionId ? { ...sess, title, updatedAt: Date.now() } : sess,
                ),
            }));
        },

        clearAll: async () => {
            // TODO: Integrate with Tauri backend
            // Command: DELETE FROM sessions

            update((s) => ({ ...s, sessions: [], currentSessionId: null }));
        },

        exportSession: (sessionId: string) => {
            const state = get({ subscribe });
            const session = state.sessions.find((s) => s.id === sessionId);
            return session ? JSON.stringify(session, null, 2) : null;
        },

        importSession: async (_json: string) => {
            // TODO: Implement session import
            return false;
        },
    };
}

export const chatHistory = createChatHistoryStore();

export const currentSession = derived(chatHistory, ($h) => {
    if (!$h.currentSessionId) return null;
    return $h.sessions.find((s) => s.id === $h.currentSessionId) || null;
});

export const sortedSessions = derived(chatHistory, ($h) => {
    return [...$h.sessions].sort((a, b) => b.updatedAt - a.updatedAt);
});

// Auto-initialize on client side
if (typeof window !== 'undefined') {
    chatHistory.init();
}
