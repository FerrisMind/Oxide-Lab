import { writable } from 'svelte/store';

export const headerSearchQuery = writable('');
export const searchTrigger = writable(0);

// Function to trigger search from header
export function triggerHeaderSearch(query: string) {
  headerSearchQuery.set(query);
  searchTrigger.update((n) => n + 1);
}
