import { writable } from 'svelte/store';

// Управление правой боковой панелью (по умолчанию закрыта)
export const rightSidebarOpen = writable(false);
