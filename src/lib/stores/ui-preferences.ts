import { browser } from '$app/environment';
import { writable } from 'svelte/store';

const MODEL_SELECTOR_SEARCH_KEY = 'ui.modelSelectorSearch';
const DEFAULT_SEARCH_ENABLED = true;

const readInitialSearchValue = () => {
  if (!browser) return DEFAULT_SEARCH_ENABLED;

  try {
    const saved = window.localStorage.getItem(MODEL_SELECTOR_SEARCH_KEY);
    if (saved === null) return DEFAULT_SEARCH_ENABLED;
    return saved === 'true';
  } catch {
    return DEFAULT_SEARCH_ENABLED;
  }
};

const modelSelectorSearchStore = writable<boolean>(readInitialSearchValue());

if (browser) {
  modelSelectorSearchStore.subscribe((value) => {
    try {
      window.localStorage.setItem(MODEL_SELECTOR_SEARCH_KEY, String(value));
    } catch {
      // Ignore storage failures (e.g., Safari private mode)
    }
  });
}

export const modelSelectorSearchEnabled = {
  subscribe: modelSelectorSearchStore.subscribe,
  set(value: boolean) {
    modelSelectorSearchStore.set(value);
  },
  toggle() {
    modelSelectorSearchStore.update((value) => !value);
  },
};
