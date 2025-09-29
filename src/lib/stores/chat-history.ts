import { writable, derived, get } from 'svelte/store';
import type { ChatMessage } from '$lib/chat/types';

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
};

const STORAGE_KEY = 'oxide-lab-chat-history';
const MAX_SESSIONS = 50; // Лимит количества сохраненных чатов

// Загрузка из localStorage
function loadFromStorage(): ChatHistoryState {
  if (typeof window === 'undefined') {
    return { sessions: [], currentSessionId: null };
  }

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        sessions: Array.isArray(parsed.sessions) ? parsed.sessions : [],
        currentSessionId: parsed.currentSessionId || null,
      };
    }
  } catch (error) {
    console.error('Failed to load chat history from localStorage:', error);
  }

  return { sessions: [], currentSessionId: null };
}

// Сохранение в localStorage
function saveToStorage(state: ChatHistoryState): void {
  if (typeof window === 'undefined') return;

  try {
    // Ограничиваем количество сохраненных сессий
    const limitedSessions = state.sessions.slice(0, MAX_SESSIONS);
    const toSave = {
      ...state,
      sessions: limitedSessions,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
  } catch (error) {
    console.error('Failed to save chat history to localStorage:', error);
  }
}

// Генерация заголовка из первого сообщения
function generateTitle(messages: ChatMessage[]): string {
  const firstUserMessage = messages.find((m) => m.role === 'user');
  if (firstUserMessage) {
    const content = firstUserMessage.content.trim();
    // Берем первые 50 символов или до первой точки/новой строки
    const truncated = content.split(/[.\n]/)[0].substring(0, 50).trim();
    return truncated || 'Новый чат';
  }
  return 'Новый чат';
}

function createChatHistoryStore() {
  const { subscribe, set, update } = writable<ChatHistoryState>(loadFromStorage());

  // Подписываемся на изменения и сохраняем
  subscribe((state) => {
    saveToStorage(state);
  });

  return {
    subscribe,

    // Создать новую сессию
    createSession: (modelPath?: string, repoId?: string): string => {
      const newSession: ChatSession = {
        id: `session-${Date.now()}-${Math.random().toString(36).substring(7)}`,
        title: 'Новый чат',
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
        modelPath,
        repoId,
      };

      update((state) => ({
        sessions: [newSession, ...state.sessions],
        currentSessionId: newSession.id,
      }));

      return newSession.id;
    },

    // Загрузить сессию
    loadSession: (sessionId: string) => {
      update((state) => {
        const session = state.sessions.find((s) => s.id === sessionId);
        if (session) {
          console.log(
            'loadSession: загружаем сессию',
            sessionId,
            'с',
            session.messages.length,
            'сообщениями',
          );
          const userMsgs = session.messages.filter((m) => m.role === 'user').length;
          const assistantMsgs = session.messages.filter((m) => m.role === 'assistant').length;
          console.log(`  - Пользователь: ${userMsgs}, Ассистент: ${assistantMsgs}`);
        } else {
          console.warn('loadSession: сессия не найдена', sessionId);
        }

        return {
          ...state,
          currentSessionId: sessionId,
        };
      });
    },

    // Обновить сообщения в текущей сессии
    updateMessages: (messages: ChatMessage[]) => {
      update((state) => {
        if (!state.currentSessionId) {
          console.warn('updateMessages: нет активной сессии');
          return state;
        }

        console.log(
          'updateMessages: обновляем сессию',
          state.currentSessionId,
          'с',
          messages.length,
          'сообщениями',
        );

        const sessions = state.sessions.map((session) => {
          if (session.id === state.currentSessionId) {
            // Логируем типы сообщений
            const userMsgs = messages.filter((m) => m.role === 'user').length;
            const assistantMsgs = messages.filter((m) => m.role === 'assistant').length;
            console.log(`  - Пользователь: ${userMsgs}, Ассистент: ${assistantMsgs}`);

            return {
              ...session,
              messages: [...messages],
              title: messages.length > 0 ? generateTitle(messages) : session.title,
              updatedAt: Date.now(),
            };
          }
          return session;
        });

        return { ...state, sessions };
      });
    },

    // Добавить сообщение в текущую сессию
    addMessage: (message: ChatMessage) => {
      update((state) => {
        if (!state.currentSessionId) return state;

        const sessions = state.sessions.map((session) => {
          if (session.id === state.currentSessionId) {
            const newMessages = [...session.messages, message];
            return {
              ...session,
              messages: newMessages,
              title: newMessages.length === 1 ? generateTitle(newMessages) : session.title,
              updatedAt: Date.now(),
            };
          }
          return session;
        });

        return { ...state, sessions };
      });
    },

    // Обновить последнее сообщение (для streaming)
    updateLastMessage: (content: string) => {
      update((state) => {
        if (!state.currentSessionId) return state;

        const sessions = state.sessions.map((session) => {
          if (session.id === state.currentSessionId) {
            const messages = [...session.messages];
            if (messages.length > 0) {
              messages[messages.length - 1] = {
                ...messages[messages.length - 1],
                content,
              };
            }
            return {
              ...session,
              messages,
              updatedAt: Date.now(),
            };
          }
          return session;
        });

        return { ...state, sessions };
      });
    },

    // Удалить сессию
    deleteSession: (sessionId: string) => {
      update((state) => {
        const sessions = state.sessions.filter((s) => s.id !== sessionId);
        let currentSessionId = state.currentSessionId;

        // Если удаляем текущую сессию, переключаемся на первую доступную
        if (currentSessionId === sessionId) {
          currentSessionId = sessions.length > 0 ? sessions[0].id : null;
        }

        return { sessions, currentSessionId };
      });
    },

    // Переименовать сессию
    renameSession: (sessionId: string, title: string) => {
      update((state) => ({
        ...state,
        sessions: state.sessions.map((session) =>
          session.id === sessionId ? { ...session, title, updatedAt: Date.now() } : session,
        ),
      }));
    },

    // Очистить все сессии
    clearAll: () => {
      set({ sessions: [], currentSessionId: null });
    },

    // Экспорт сессии
    exportSession: (sessionId: string): string | null => {
      const state = get({ subscribe });
      const session = state.sessions.find((s) => s.id === sessionId);
      if (session) {
        return JSON.stringify(session, null, 2);
      }
      return null;
    },

    // Импорт сессии
    importSession: (jsonData: string): boolean => {
      try {
        const session = JSON.parse(jsonData) as ChatSession;
        // Валидация
        if (!session.id || !Array.isArray(session.messages)) {
          return false;
        }

        // Генерируем новый ID для избежания конфликтов
        const newSession: ChatSession = {
          ...session,
          id: `session-${Date.now()}-${Math.random().toString(36).substring(7)}`,
          createdAt: Date.now(),
          updatedAt: Date.now(),
        };

        update((state) => ({
          sessions: [newSession, ...state.sessions],
          currentSessionId: newSession.id,
        }));

        return true;
      } catch (error) {
        console.error('Failed to import session:', error);
        return false;
      }
    },
  };
}

export const chatHistory = createChatHistoryStore();

// Derived store для текущей сессии
export const currentSession = derived(chatHistory, ($history) => {
  if (!$history.currentSessionId) return null;
  return $history.sessions.find((s) => s.id === $history.currentSessionId) || null;
});

// Derived store для сортированных сессий (новые сверху)
export const sortedSessions = derived(chatHistory, ($history) => {
  return [...$history.sessions].sort((a, b) => b.updatedAt - a.updatedAt);
});
