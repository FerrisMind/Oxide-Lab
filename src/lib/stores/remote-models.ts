import { get, writable } from 'svelte/store';
import type { DownloadJob, RemoteGGUFFile, RemoteModelFilters, RemoteModelInfo } from '$lib/types/local-models';
import { LocalModelsService } from '$lib/services/local-models';

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
  destinationDir: string,
  file: RemoteGGUFFile,
): Promise<DownloadJob> {
  const downloadUrl = file.download_url;
  if (!downloadUrl) {
    throw new Error('Download URL is missing for the selected file.');
  }
  return LocalModelsService.downloadRemoteModel(
    repoId,
    file.filename,
    destinationDir,
    downloadUrl,
    file.size,
    file.sha256,
  );
}

export function updateRemoteFilters(partial: Partial<RemoteModelFilters>): void {
  remoteFilters.update((current) => ({
    ...current,
    ...partial,
  }));
}
