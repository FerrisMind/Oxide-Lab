/**
 * Sidebar State Store
 * 
 * Manages sidebar visibility and chat history panel state.
 */

import { writable } from 'svelte/store';

export const showChatHistory = writable(false);
