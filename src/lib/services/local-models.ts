/**
 * Service for managing local and remote GGUF models through the Tauri backend.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  DownloadProgressPayload,
  DownloadedFileInfo,
  FilterOptions,
  ModelInfo,
  RemoteModelFilters,
  RemoteModelInfo,
  SortField,
  SortOrder,
} from '$lib/types/local-models';

export class LocalModelsService {
  /**
   * Scan a directory for local GGUF models.
   */
  static async scanFolder(folderPath: string): Promise<ModelInfo[]> {
    try {
      return await invoke<ModelInfo[]>('scan_models_folder', { folderPath });
    } catch (error) {
      console.error('Failed to scan folder:', error);
      throw new Error(`Failed to scan folder: ${error}`);
    }
  }

  /**
   * Request full GGUF metadata for a specific file.
   */
  static async parseMetadata(filePath: string) {
    try {
      return await invoke('parse_gguf_metadata', { filePath });
    } catch (error) {
      console.error('Failed to parse GGUF metadata:', error);
      throw new Error(`Failed to parse GGUF metadata: ${error}`);
    }
  }

  /**
   * Delete a local model file.
   */
  static async deleteModel(modelPath: string): Promise<void> {
    try {
      await invoke('delete_local_model', { modelPath });
    } catch (error) {
      console.error('Failed to delete model:', error);
      throw new Error(`Failed to delete model: ${error}`);
    }
  }

  /**
   * Search Hugging Face Hub for GGUF models.
   */
  static async searchRemote(
    query: string,
    filters: RemoteModelFilters = {},
  ): Promise<RemoteModelInfo[]> {
    try {
      return await invoke<RemoteModelInfo[]>('search_huggingface_gguf', { query, filters });
    } catch (error) {
      console.error('Failed to search Hugging Face:', error);
      throw new Error(`Failed to search Hugging Face: ${error}`);
    }
  }

  /**
   * Download a remote GGUF file and place it in destination directory.
   */
  static async downloadRemoteModel(
    repoId: string,
    filename: string,
    destinationDir: string,
  ): Promise<DownloadedFileInfo> {
    try {
      return await invoke<DownloadedFileInfo>('download_hf_model_file', {
        repoId,
        filename,
        destinationDir,
      });
    } catch (error) {
      console.error('Failed to download model:', error);
      throw new Error(`Failed to download model: ${error}`);
    }
  }

  /**
   * Subscribe to backend download progress events.
   */
  static async onDownloadProgress(
    handler: (payload: DownloadProgressPayload) => void,
  ): Promise<UnlistenFn> {
    return listen<DownloadProgressPayload>('model-download-progress', (event) => {
      handler(event.payload);
    });
  }

  /**
   * Format bytes into a human-readable string.
   */
  static formatFileSize(bytes: number): string {
    if (!Number.isFinite(bytes)) return '—';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return unitIndex === 0 ? `${size} ${units[unitIndex]}` : `${size.toFixed(2)} ${units[unitIndex]}`;
  }

  /**
   * Format ISO date string for UI display.
   */
  static formatDate(isoString: string): string {
    if (!isoString) return '—';
    const date = new Date(isoString);
    if (Number.isNaN(date.getTime())) return isoString;
    return date.toLocaleString('ru-RU', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  /**
   * Sort models by selected field and order.
   */
  static sortModels(models: ModelInfo[], field: SortField, order: SortOrder): ModelInfo[] {
    const sorted = [...models].sort((a, b) => {
      const extractValue = (model: ModelInfo) => {
        switch (field) {
          case 'file_size':
            return model.file_size;
          case 'created_at':
            return new Date(model.created_at).getTime();
          case 'parameter_count':
            return model.parameter_count ? parseFloat(model.parameter_count) : 0;
          case 'architecture':
            return model.architecture ?? '';
          default:
            return model.name;
        }
      };

      const aVal = extractValue(a);
      const bVal = extractValue(b);

      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return aVal - bVal;
      }

      return String(aVal).localeCompare(String(bVal));
    });

    return order === 'desc' ? sorted.reverse() : sorted;
  }

  /**
   * Apply filters to the provided models list.
   */
  static filterModels(models: ModelInfo[], options: FilterOptions): ModelInfo[] {
    const searchText = options.searchText?.trim().toLowerCase() ?? '';
    return models.filter((model) => {
      if (options.architecture && model.architecture !== options.architecture) {
        return false;
      }

      if (options.quantization && model.quantization !== options.quantization) {
        return false;
      }

      if (options.candleOnly && !model.candle_compatible) {
        return false;
      }

      if (options.validation && options.validation !== 'all') {
        if (model.validation_status.level !== options.validation) {
          return false;
        }
      }

      if (searchText) {
        const haystack = [
          model.name,
          model.model_name ?? '',
          model.architecture ?? '',
          model.quantization ?? '',
          model.parameter_count ?? '',
        ]
          .join(' ')
          .toLowerCase();
        if (!haystack.includes(searchText)) {
          return false;
        }
      }

      return true;
    });
  }

  /**
   * Get unique string values for filter dropdowns.
   */
  static getUniqueValues(models: ModelInfo[], field: keyof ModelInfo): string[] {
    const values = models
      .map((model) => model[field])
      .filter((value): value is string => typeof value === 'string' && value.length > 0);

    return Array.from(new Set(values)).sort((a, b) => a.localeCompare(b));
  }
}
