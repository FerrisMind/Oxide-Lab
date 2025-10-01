/**
 * Service for managing local model files
 */

import { invoke } from '@tauri-apps/api/core';
import type { LocalModelInfo } from '$lib/types/local-models';

/**
 * Local models service
 */
export class LocalModelsService {
  /**
   * Scan a folder for local model files
   * @param folderPath - Path to the folder to scan
   * @returns Promise with array of found models
   */
  static async scanFolder(folderPath: string): Promise<LocalModelInfo[]> {
    try {
      const models = await invoke<LocalModelInfo[]>('scan_local_models_folder', {
        folderPath,
      });
      return models;
    } catch (error) {
      console.error('Failed to scan folder:', error);
      throw new Error(`Failed to scan folder: ${error}`);
    }
  }

  /**
   * Delete a local model file
   * @param modelPath - Full path to the model file
   */
  static async deleteModel(modelPath: string): Promise<void> {
    try {
      await invoke('delete_local_model', {
        modelPath,
      });
    } catch (error) {
      console.error('Failed to delete model:', error);
      throw new Error(`Failed to delete model: ${error}`);
    }
  }

  /**
   * Format file size to human-readable string
   * @param bytes - File size in bytes
   * @returns Formatted string (e.g., "1.5 GB", "256.3 MB")
   */
  static formatFileSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    // Format with 2 decimal places for sizes >= 1 KB
    if (unitIndex > 0) {
      return `${size.toFixed(2)} ${units[unitIndex]}`;
    }

    return `${size} ${units[unitIndex]}`;
  }

  /**
   * Format timestamp to readable date string
   * @param unixTimestamp - Unix timestamp in seconds
   * @returns Formatted date string
   */
  static formatDate(unixTimestamp: number): string {
    const date = new Date(unixTimestamp * 1000);
    return date.toLocaleString('ru-RU', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  /**
   * Sort models by specified field and order
   * @param models - Array of models to sort
   * @param field - Field to sort by
   * @param order - Sort order ('asc' or 'desc')
   * @returns Sorted array of models
   */
  static sortModels(
    models: LocalModelInfo[],
    field: keyof LocalModelInfo,
    order: 'asc' | 'desc',
  ): LocalModelInfo[] {
    const sorted = [...models].sort((a, b) => {
      const aValue = a[field];
      const bValue = b[field];

      // Handle undefined values
      if (aValue === undefined && bValue === undefined) return 0;
      if (aValue === undefined) return 1;
      if (bValue === undefined) return -1;

      // Compare values
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return aValue.localeCompare(bValue);
      }

      if (typeof aValue === 'number' && typeof bValue === 'number') {
        return aValue - bValue;
      }

      return 0;
    });

    return order === 'desc' ? sorted.reverse() : sorted;
  }

  /**
   * Filter models by criteria
   * @param models - Array of models to filter
   * @param options - Filter options
   * @returns Filtered array of models
   */
  static filterModels(
    models: LocalModelInfo[],
    options: {
      format?: string;
      architecture?: string;
      quantization?: string;
      searchText?: string;
    },
  ): LocalModelInfo[] {
    return models.filter((model) => {
      // Filter by format
      if (options.format && model.format !== options.format) {
        return false;
      }

      // Filter by architecture
      if (options.architecture && model.architecture !== options.architecture) {
        return false;
      }

      // Filter by quantization
      if (options.quantization && model.quantization !== options.quantization) {
        return false;
      }

      // Filter by search text
      if (options.searchText) {
        const searchLower = options.searchText.toLowerCase();
        const matchesName = model.name.toLowerCase().includes(searchLower);
        const matchesAuthor = model.author?.toLowerCase().includes(searchLower) || false;
        const matchesArchitecture =
          model.architecture?.toLowerCase().includes(searchLower) || false;

        if (!matchesName && !matchesAuthor && !matchesArchitecture) {
          return false;
        }
      }

      return true;
    });
  }

  /**
   * Get unique values for a field across all models (for filter dropdowns)
   * @param models - Array of models
   * @param field - Field to extract unique values from
   * @returns Array of unique values
   */
  static getUniqueValues(models: LocalModelInfo[], field: keyof LocalModelInfo): string[] {
    const values = models
      .map((model) => model[field])
      .filter((value) => value !== undefined && value !== null) as string[];

    return Array.from(new Set(values)).sort();
  }
}
