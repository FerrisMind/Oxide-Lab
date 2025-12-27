/**
 * Chat message types
 */

export type Role = 'user' | 'assistant';

export type ChatMessage = {
    role: Role;
    content: string;
    html?: string;
    thinking?: string;
    isThinking?: boolean;
};

