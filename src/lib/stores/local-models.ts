/**
 * Local Models Store
 *
 * Manages local model discovery, caching, filtering, and selection.
 * Uses LocalModelsService for all backend communication (SRP/DIP).
 */

import { writable, derived } from 'svelte/store';
import type {
    ModelInfo,
    LocalModelsCache,
    SortOptions,
    FilterOptions,
    ValidationLevel,
} from '$lib/types/local-models';
import { LocalModelsService } from '$lib/services/local-models';

const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes in milliseconds
const STORAGE_KEY = 'local_models_folder_path';

/**
 * Store for selected folder path
 */
function createFolderPathStore() {
    const savedPath =
        typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) || '' : '';

    const { subscribe, set } = writable<string>(savedPath);

    return {
        subscribe,
        set: (path: string) => {
            set(path);
            if (typeof localStorage !== 'undefined') {
                localStorage.setItem(STORAGE_KEY, path);
            }
        },
        clear: () => {
            set('');
            if (typeof localStorage !== 'undefined') {
                localStorage.removeItem(STORAGE_KEY);
            }
        },
    };
}

export const folderPath = createFolderPathStore();

/**
 * Helper to set folder path
 */
export function setFolderPath(path: string): void {
    folderPath.set(path);
}

/**
 * Store for models list
 */
export const models = writable<ModelInfo[]>([]);

/**
 * Store for selected model
 */
export const selectedModel = writable<ModelInfo | null>(null);

/**
 * Store for loading state
 */
export const isLoading = writable<boolean>(false);

/**
 * Store for error messages
 */
export const error = writable<string | null>(null);

/**
 * Store for cache
 */
const cache = writable<LocalModelsCache | null>(null);

/**
 * Store for sort options
 */
export const sortOptions = writable<SortOptions>({
    field: 'name',
    order: 'asc',
});

/**
 * Store for filter options
 */
export const filterOptions = writable<FilterOptions>({
    validation: 'all',
});

// NOTE: Removed local filterModels and sortModels functions.
// Now using LocalModelsService.filterModels and LocalModelsService.sortModels (DRY principle).

/**
 * Derived store for filtered and sorted models
 */
export const filteredModels = derived(
    [models, sortOptions, filterOptions],
    ([$models, $sortOptions, $filterOptions]) => {
        // Using LocalModelsService for consistent filtering/sorting (DRY)
        let result = LocalModelsService.filterModels($models, $filterOptions);
        result = LocalModelsService.sortModels(result, $sortOptions.field, $sortOptions.order);
        return result;
    },
);

// NOTE: Removed local getUniqueValues function.
// Now using LocalModelsService.getUniqueValues (DRY principle).

/**
 * Derived store for unique architectures (for filter dropdown)
 */

/**
 * Derived store for unique architectures (for filter dropdown)
 */
export const uniqueArchitectures = derived(models, ($models) => {
    return LocalModelsService.getUniqueValues($models, 'architecture');
});

/**
 * Derived store for unique quantizations (for filter dropdown)
 */
export const uniqueQuantizations = derived(models, ($models) => {
    return LocalModelsService.getUniqueValues($models, 'quantization');
});

/**
 * Count models by validation level for quick summary.
 */
export const validationCounters = derived(models, ($models) => {
    return $models.reduce<Record<ValidationLevel | 'total', number>>(
        (acc, model) => {
            acc.total += 1;
            const level = model.validation_status.level;
            acc[level] = (acc[level] ?? 0) + 1;
            return acc;
        },
        { ok: 0, warning: 0, error: 0, total: 0 },
    );
});

/**
 * Check if cache is valid
 */
function isCacheValid(cachedData: LocalModelsCache | null, path: string): boolean {
    if (!cachedData) return false;
    if (cachedData.folder_path !== path) return false;

    const now = Date.now();
    const cacheAge = now - cachedData.cached_at;
    const duration = cachedData.cache_duration || CACHE_DURATION;

    return cacheAge < duration;
}

/**
 * Scan folder for models
 */
export async function scanFolder(path: string, forceRefresh: boolean = false): Promise<void> {
    // Check cache first if not forcing refresh
    if (!forceRefresh) {
        let cachedData: LocalModelsCache | null = null;
        cache.subscribe((value) => {
            cachedData = value;
        })();

        if (isCacheValid(cachedData, path)) {
            models.set(cachedData!.models);
            error.set(null);
            return;
        }
    }

    isLoading.set(true);
    error.set(null);

    try {
        // Call Tauri backend through LocalModelsService
        const foundModels = await LocalModelsService.scanFolder(path);

        // Update stores
        models.set(foundModels);
        folderPath.set(path);

        // Update cache
        cache.set({
            folder_path: path,
            models: foundModels,
            cached_at: Date.now(),
            cache_duration: CACHE_DURATION,
        });

        error.set(null);
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        error.set(errorMessage);
        models.set([]);
    } finally {
        isLoading.set(false);
    }
}

/**
 * Delete a model
 */
export async function deleteModel(modelPath: string): Promise<void> {
    try {
        // Call Tauri backend through LocalModelsService
        await LocalModelsService.deleteModel(modelPath);

        // Remove from models list
        models.update(($models) => $models.filter((m) => m.path !== modelPath));

        // Clear selection if deleted model was selected
        selectedModel.update(($selected) => ($selected?.path === modelPath ? null : $selected));

        // Update cache
        cache.update(($cache) => {
            if ($cache) {
                return {
                    ...$cache,
                    models: $cache.models.filter((m) => m.path !== modelPath),
                };
            }
            return $cache;
        });

        error.set(null);
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        error.set(errorMessage);
        throw err;
    }
}

/**
 * Clear cache and reset state
 */
export function clearCache(): void {
    cache.set(null);
    models.set([]);
    selectedModel.set(null);
    error.set(null);
}

/**
 * Select a model
 */
export function selectModel(model: ModelInfo | null): void {
    selectedModel.set(model);
}
