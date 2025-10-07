import { derived, writable } from 'svelte/store';
import type {
  DownloadHistoryEntry,
  DownloadJob,
  DownloadManagerSnapshot,
} from '$lib/types/local-models';
import { LocalModelsService } from '$lib/services/local-models';
import type { UnlistenFn } from '@tauri-apps/api/event';

const snapshot = writable<DownloadManagerSnapshot>({ active: [], history: [] });
const isReady = writable(false);

let unlisten: UnlistenFn | null = null;

function sortByUpdatedAt<T extends { updated_at?: string; finished_at?: string }>(items: T[]): T[] {
  return [...items].sort((a, b) => {
    const aTime = a.updated_at ?? a.finished_at ?? '';
    const bTime = b.updated_at ?? b.finished_at ?? '';
    return bTime.localeCompare(aTime);
  });
}

export const activeDownloads = derived(snapshot, ($snapshot) =>
  sortByUpdatedAt($snapshot.active).filter(
    (job) => job.status !== 'completed' && job.status !== 'cancelled',
  ),
);

export const completedDownloads = derived(snapshot, ($snapshot) =>
  sortByUpdatedAt([
    ...$snapshot.active.filter((job) => job.status === 'completed'),
    ...$snapshot.history.filter((entry) => entry.status === 'completed'),
  ]),
);

export const downloadHistory = derived(snapshot, ($snapshot) => sortByUpdatedAt($snapshot.history));
export const downloadsLoaded = derived(isReady, ($ready) => $ready);

export async function ensureDownloadManager(): Promise<void> {
  if (typeof window === 'undefined' || unlisten) {
    return;
  }

  try {
    const initial = await LocalModelsService.getDownloadSnapshot();
    snapshot.set(initial);
    isReady.set(true);
  } catch (error) {
    console.error('Failed to load download snapshot:', error);
  }

  unlisten = await LocalModelsService.onDownloadSnapshotUpdate((payload) => {
    snapshot.set(payload);
    isReady.set(true);
  });
}

export async function refreshDownloadSnapshot(): Promise<void> {
  try {
    const current = await LocalModelsService.getDownloadSnapshot();
    snapshot.set(current);
    isReady.set(true);
  } catch (error) {
    console.error('Failed to refresh download snapshot:', error);
  }
}

export function stopDownloadManager(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  isReady.set(false);
}

export async function pauseDownload(job: DownloadJob): Promise<void> {
  await LocalModelsService.pauseDownload(job.id);
}

export async function resumeDownload(job: DownloadJob): Promise<void> {
  await LocalModelsService.resumeDownload(job.id);
}

export async function cancelDownload(job: DownloadJob): Promise<void> {
  await LocalModelsService.cancelDownload(job.id);
}

export async function removeDownload(
  entry: DownloadHistoryEntry | DownloadJob,
  deleteFile: boolean,
): Promise<void> {
  await LocalModelsService.removeDownloadEntry(entry.id, deleteFile);
}

export async function clearHistory(): Promise<void> {
  await LocalModelsService.clearDownloadHistory();
}
