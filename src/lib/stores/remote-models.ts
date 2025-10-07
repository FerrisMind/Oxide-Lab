import { derived, get, writable } from 'svelte/store';
import type {
  DownloadProgressPayload,
  DownloadedFileInfo,
  RemoteModelFilters,
  RemoteModelInfo,
} from '$lib/types/local-models';
import { LocalModelsService } from '$lib/services/local-models';
import type { UnlistenFn } from '@tauri-apps/api/event';

const DEFAULT_LIMIT = 25;

export const remoteResults = writable<RemoteModelInfo[]>([]);
export const remoteIsLoading = writable<boolean>(false);
export const remoteError = writable<string | null>(null);
export const searchQuery = writable<string>('');
export const remoteFilters = writable<RemoteModelFilters>({
  sort_by: 'downloads',
  sort_order: 'desc',
  limit: DEFAULT_LIMIT,
});

const downloadProgress = writable<Record<string, DownloadProgressPayload>>({});

let unlistenProgress: UnlistenFn | null = null;

export async function ensureProgressListener(): Promise<void> {
  if (unlistenProgress || typeof window === 'undefined') {
    return;
  }

  unlistenProgress = await LocalModelsService.onDownloadProgress((payload) => {
    downloadProgress.update((current) => {
      const next = { ...current, [payload.download_id]: payload };
      if (payload.stage === 'finished') {
        setTimeout(() => {
          downloadProgress.update((map) => {
            const cloned = { ...map };
            delete cloned[payload.download_id];
            return cloned;
          });
        }, 1500);
      }
      return next;
    });
  });
}

export function stopProgressListener(): void {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
}

export const activeDownloads = derived(downloadProgress, ($progress) =>
  Object.values($progress),
);

export async function searchRemoteModels(force: boolean = false): Promise<void> {
  remoteIsLoading.set(true);
  remoteError.set(null);

  try {
    const query = get(searchQuery);
    const filters = get(remoteFilters);
    const results = await LocalModelsService.searchRemote(query, filters);
    remoteResults.set(results);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    remoteError.set(message);
    if (!force) {
      remoteResults.set([]);
    }
  } finally {
    remoteIsLoading.set(false);
  }
}

export async function downloadRemoteModel(
  repoId: string,
  filename: string,
  destinationDir: string,
): Promise<DownloadedFileInfo> {
  await ensureProgressListener();
  return LocalModelsService.downloadRemoteModel(repoId, filename, destinationDir);
}

export function updateRemoteFilters(partial: Partial<RemoteModelFilters>): void {
  remoteFilters.update((current) => ({
    ...current,
    ...partial,
  }));
}
