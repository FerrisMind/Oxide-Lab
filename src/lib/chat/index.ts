/**
 * Chat Module - Central exports
 */

// Main Chat component
export { default as Chat } from './Chat.svelte';

// Types
export * from './types';

// Controller
export { createChatController } from './controller';
export type { ChatControllerCtx } from './controller/types';

// Utils
export { sanitizeForPrompt } from './sanitize';
export { buildPromptWithChatTemplate } from './prompts';

// Components
export * from './components';
