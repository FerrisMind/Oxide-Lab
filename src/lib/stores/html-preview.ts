/**
 * HTML Preview Store
 *
 * Manages state for the HTML preview panel in chat.
 */

import { writable, derived } from 'svelte/store';

export interface HtmlPreviewState {
  html: string | null;
  isOpen: boolean;
}

const initialState: HtmlPreviewState = {
  html: null,
  isOpen: false,
};

function createHtmlPreviewStore() {
  const { subscribe, set, update } = writable<HtmlPreviewState>(initialState);

  return {
    subscribe,

    openPreview: (html: string) => {
      update((state) => ({ ...state, html, isOpen: true }));
    },

    closePreview: () => {
      update((state) => ({ ...state, isOpen: false }));
    },

    togglePreview: () => {
      update((state) => ({ ...state, isOpen: !state.isOpen }));
    },

    reset: () => set(initialState),
  };
}

export const htmlPreviewStore = createHtmlPreviewStore();

export const isPreviewOpen = derived(htmlPreviewStore, ($store) => $store.isOpen);
export const previewHtml = derived(htmlPreviewStore, ($store) => $store.html);
