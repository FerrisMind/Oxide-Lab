import { writable } from 'svelte/store';

export type TabId = 'local' | 'remote' | '';

interface PageTab {
  id: TabId;
  label: string;
}

export const activePageTab = writable<TabId>('');
export const pageTabsList = writable<PageTab[]>([]);

export function setPageTabs(tabs: PageTab[], active: TabId) {
  pageTabsList.set(tabs);
  activePageTab.set(active);
}

export function clearPageTabs() {
  pageTabsList.set([]);
  activePageTab.set('');
}
