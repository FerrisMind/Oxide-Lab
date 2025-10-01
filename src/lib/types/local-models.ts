/**
 * TypeScript interfaces for local models management
 */

/**
 * Information about a local model file
 */
export interface LocalModelInfo {
  /** Full path to the model file */
  path: string;

  /** Model name (filename without extension) */
  name: string;

  /** Model architecture (e.g., "llama", "gpt2", "bert") */
  architecture?: string;

  /** Number of parameters (e.g., "7B", "13B") */
  parameters?: string;

  /** Model author/publisher */
  author?: string;

  /** Quantization level (e.g., "Q4_K_M", "Q5_K_S", "F16") */
  quantization?: string;

  /** File size in bytes */
  size_bytes: number;

  /** File format ("gguf" or "safetensors") */
  format: string;

  /** Last modified timestamp (Unix epoch in seconds) */
  last_modified: number;
}

/**
 * Cache entry for local models
 */
export interface LocalModelsCache {
  /** Folder path that was scanned */
  folder_path: string;

  /** List of models found in the folder */
  models: LocalModelInfo[];

  /** Timestamp when the cache was created (Unix epoch in milliseconds) */
  cached_at: number;

  /** Cache validity duration in milliseconds (default: 5 minutes) */
  cache_duration?: number;
}

/**
 * Sort options for local models list
 */
export type SortField = 'name' | 'size_bytes' | 'last_modified' | 'parameters';
export type SortOrder = 'asc' | 'desc';

export interface SortOptions {
  field: SortField;
  order: SortOrder;
}

/**
 * Filter options for local models list
 */
export interface FilterOptions {
  /** Filter by format (gguf, safetensors, or empty for all) */
  format?: string;

  /** Filter by architecture */
  architecture?: string;

  /** Filter by quantization level */
  quantization?: string;

  /** Search text to filter by name */
  searchText?: string;
}
