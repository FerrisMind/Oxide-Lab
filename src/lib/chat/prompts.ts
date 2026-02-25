/**
 * Chat Prompt Builder
 * 
 * Builds chat messages for OpenAI-compatible API format.
 * The engine handles template rendering internally.
 */

import { sanitizeForPrompt } from '$lib/chat/sanitize';
import type { ChatMessage } from '$lib/chat/types';

/**
 * Build a clean prompt string from chat history (fallback for non-API usage).
 * Primary path now uses OpenAI-compatible messages array directly.
 */
export async function buildPromptWithChatTemplate(history: ChatMessage[]): Promise<string> {
    // Simple Qwen-compatible format as fallback
    let text = '';

    for (const m of history) {
        const clean = sanitizeForPrompt(m.content);
        if (m.role === 'user') {
            // Remove /think and /no_think control commands  
            const payload = clean.replace(/^\s*\/(?:no_think|think)\b[ \t]*/i, '').trim();
            text += `<|im_start|>user\n${payload}<|im_end|>\n`;
        } else {
            text += `<|im_start|>assistant\n${clean}<|im_end|>\n`;
        }
    }

    text += `<|im_start|>assistant\n`;
    return text;
}
