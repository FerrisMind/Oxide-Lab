import { writable, derived, get } from 'svelte/store';
import type { ChatMessage } from '$lib/chat/types';
import Database from '@tauri-apps/plugin-sql';

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
  db: Database | null;
  isInitialized: boolean;
};

const DB_PATH = 'sqlite:chat_history.db';
const LOCAL_STORAGE_KEY = 'oxide-lab-chat-history';

function createChatHistoryStore() {
  const { subscribe, update } = writable<ChatHistoryState>({
    sessions: [],
    currentSessionId: null,
    db: null,
    isInitialized: false,
  });

  const insertMessage = async (message: ChatMessage, sessionId?: string) => {
    const state = get({ subscribe });
    const targetSessionId = sessionId ?? state.currentSessionId;
    if (!state.db || !targetSessionId) return;

    const now = Date.now();
    const sessionSnapshot = state.sessions.find((s) => s.id === targetSessionId);
    const isFirstUserMessage = sessionSnapshot?.messages.length === 0 && message.role === 'user';
    const titleFromMessage = isFirstUserMessage
      ? message.content.substring(0, 50).split('\n')[0]
      : sessionSnapshot?.title;

    // #region agent log
    void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        sessionId: 'debug-session',
        runId: 'run1',
        hypothesisId: 'H1',
        location: 'chat-history.ts:insertMessage',
        message: 'insertMessage attempt',
        data: {
          role: message.role,
          targetSessionId,
          currentSessionId: state.currentSessionId,
          hasDb: !!state.db,
          snapshotMessages: sessionSnapshot?.messages?.length ?? null,
        },
        timestamp: Date.now(),
      }),
    }).catch(() => {});
    // #endregion

    await state.db.execute(
      'INSERT INTO messages (session_id, role, content, created_at) VALUES ($1, $2, $3, $4)',
      [targetSessionId, message.role, message.content, now],
    );

    if (isFirstUserMessage && titleFromMessage) {
      await state.db.execute('UPDATE sessions SET title = $1 WHERE id = $2', [
        titleFromMessage,
        targetSessionId,
      ]);
    }

    await state.db.execute('UPDATE sessions SET updated_at = $1 WHERE id = $2', [
      now,
      targetSessionId,
    ]);

      // #region agent log
      void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run1',
          hypothesisId: 'H2',
          location: 'chat-history.ts:insertMessage',
          message: 'insertMessage persisted',
          data: {
            role: message.role,
            sessionId: targetSessionId,
            titleFromMessage: titleFromMessage ?? null,
          },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
      // #endregion

    update((s) => {
      const sessions = s.sessions.map((sess) => {
        if (sess.id !== targetSessionId) return sess;

        return {
          ...sess,
          title: titleFromMessage ?? sess.title,
          messages: [...sess.messages, message],
          updatedAt: now,
        };
      });

      const shouldSetCurrent = s.currentSessionId ?? targetSessionId;

      return { ...s, sessions, currentSessionId: shouldSetCurrent };
    });
  };

  return {
    subscribe,

    // Инициализация БД и миграция
    init: async () => {
      try {
        const db = await Database.load(DB_PATH);

        // Загрузка сессий
        const sessionsData = await db.select<any[]>(
          'SELECT * FROM sessions ORDER BY updated_at DESC',
        );

        const sessions: ChatSession[] = await Promise.all(
          sessionsData.map(async (s) => {
            const messagesData = await db.select<any[]>(
              'SELECT * FROM messages WHERE session_id = $1 ORDER BY created_at ASC',
              [s.id],
            );

            return {
              id: s.id,
              title: s.title,
              modelPath: s.model_path,
              repoId: s.repo_id,
              createdAt: s.created_at,
              updatedAt: s.updated_at,
              messages: messagesData.map((m) => ({
                role: m.role,
                content: m.content,
              })),
            };
          }),
        );

        // Миграция из localStorage если база пуста
        if (sessions.length === 0) {
          const localData = localStorage.getItem(LOCAL_STORAGE_KEY);
          if (localData) {
            try {
              const parsed = JSON.parse(localData);
              if (Array.isArray(parsed.sessions)) {
                for (const oldSession of parsed.sessions) {
                  const id = crypto.randomUUID();
                  const now = Date.now();

                  await db.execute(
                    'INSERT INTO sessions (id, title, model_path, repo_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)',
                    [
                      id,
                      oldSession.title,
                      oldSession.modelPath || null,
                      oldSession.repoId || null,
                      now,
                      now,
                    ],
                  );

                  for (const msg of oldSession.messages) {
                    await db.execute(
                      'INSERT INTO messages (session_id, role, content, created_at) VALUES ($1, $2, $3, $4)',
                      [id, msg.role, msg.content, now],
                    );
                  }

                  // Добавляем в локальный список
                  sessions.push({
                    id,
                    title: oldSession.title,
                    modelPath: oldSession.modelPath,
                    repoId: oldSession.repoId,
                    createdAt: now,
                    updatedAt: now,
                    messages: oldSession.messages,
                  });
                }
              }
              // Очистить localStorage после успешной миграции
              // localStorage.removeItem(LOCAL_STORAGE_KEY); // Можно раскомментировать после тестов
            } catch (e) {
              console.error('Migration failed:', e);
            }
          }
        }

        update((s) => ({
          ...s,
          db,
          sessions,
          isInitialized: true,
          // Не выбираем автоматически прошлые сессии: стартовое состояние пустое
          currentSessionId: null,
        }));

        // #region agent log
        void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sessionId: 'debug-session',
            runId: 'run1',
            hypothesisId: 'H3',
            location: 'chat-history.ts:init',
            message: 'init completed',
            data: {
              sessionsCount: sessions.length,
              selectedSessionId: sessions.length > 0 ? sessions[0].id : null,
            },
            timestamp: Date.now(),
          }),
        }).catch(() => {});
        // #endregion
      } catch (err) {
        console.error('Failed to init chat database:', err);
      }
    },

    createSession: async (modelPath?: string, repoId?: string) => {
      const state = get({ subscribe });
      if (!state.db) return;

      const id = crypto.randomUUID();
      const title = 'Новый чат';
      const now = Date.now();

      try {
        await state.db.execute(
          'INSERT INTO sessions (id, title, model_path, repo_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)',
          [id, title, modelPath || null, repoId || null, now, now],
        );

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
      } catch (e) {
        console.error('Failed to create session:', e);
      }
    },

    loadSession: (sessionId: string) => {
      update((s) => {
        // #region agent log
        void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sessionId: 'debug-session',
            runId: 'run2',
            hypothesisId: 'H4',
            location: 'chat-history.ts:loadSession',
            message: 'loadSession invoked',
            data: {
              requestedSessionId: sessionId,
              sessionsInStore: s.sessions.length,
              hasDb: !!s.db,
            },
            timestamp: Date.now(),
          }),
        }).catch(() => {});
        // #endregion

        return { ...s, currentSessionId: sessionId };
      });
    },

    addMessage: async (message: ChatMessage, sessionId?: string) => {
      try {
        await insertMessage(message, sessionId);
      } catch (e) {
        console.error('Failed to add message:', e);
      }
    },

    // Метод для обновления контента последнего сообщения (для стриминга)
    // В БД пишем только когда стриминг завершен, здесь обновляем UI
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

    // Явное сохранение сообщения ассистента после завершения стриминга
    saveAssistantMessage: async (sessionId: string, content: string) => {
      try {
        await insertMessage({ role: 'assistant', content } as ChatMessage, sessionId);
      } catch (e) {
        console.error('Failed to save assistant message:', e);
      }
    },

    deleteSession: async (sessionId: string) => {
      const state = get({ subscribe });
      if (!state.db) return;

      try {
        await state.db.execute('DELETE FROM sessions WHERE id = $1', [sessionId]);

        update((s) => {
          const sessions = s.sessions.filter((sess) => sess.id !== sessionId);
          let current = s.currentSessionId;
          if (current === sessionId) {
            current = sessions.length > 0 ? sessions[0].id : null;
          }
          return { ...s, sessions, currentSessionId: current };
        });
      } catch (e) {
        console.error('Failed to delete session:', e);
      }
    },

    renameSession: async (sessionId: string, title: string) => {
      const state = get({ subscribe });
      if (!state.db) return;

      try {
        await state.db.execute('UPDATE sessions SET title = $1, updated_at = $2 WHERE id = $3', [
          title,
          Date.now(),
          sessionId,
        ]);

        update((s) => ({
          ...s,
          sessions: s.sessions.map((sess) =>
            sess.id === sessionId ? { ...sess, title, updatedAt: Date.now() } : sess,
          ),
        }));
      } catch (e) {
        console.error('Renaming failed:', e);
      }
    },

    clearAll: async () => {
      const state = get({ subscribe });
      if (!state.db) return;

      try {
        await state.db.execute('DELETE FROM sessions'); // Messages удалятся каскадом
        update((s) => ({ ...s, sessions: [], currentSessionId: null }));
      } catch (e) {
        console.error('Clear all failed:', e);
      }
    },

    // Exports/Imports пока оставляем как есть или дорабатываем
    exportSession: (sessionId: string) => {
      const state = get({ subscribe });
      const session = state.sessions.find((s) => s.id === sessionId);
      return session ? JSON.stringify(session, null, 2) : null;
    },

    // Import теперь должен быть async, пропускаем для краткости MVP
    importSession: async (_json: string) => {
      return false; /* TODO */
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

// Авто-инициализация при импорте (на клиенте)
if (typeof window !== 'undefined') {
  chatHistory.init();
}
