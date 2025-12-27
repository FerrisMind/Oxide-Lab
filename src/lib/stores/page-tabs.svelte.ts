/**
 * Page Tabs Store
 * 
 * Manages tab navigation within pages (e.g., Models page tabs).
 */

import { writable } from 'svelte/store';

export type TabId = string;

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
