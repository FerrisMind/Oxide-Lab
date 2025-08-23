// Interfaces for Hugging Face models and API responses

export interface HFModel {
  id: string;
  name: string;
  description?: string;
  downloads: number;
  likes: number;
  tags: string[];
  author: string;
  lastModified: string;
  modelType: string;
  formats: string[];
  // Поля из реального API
  pipeline_tag?: string;
  library_name?: string;
  language?: string | string[];
  license?: string;
  size?: number;
  private?: boolean;
  gated?: boolean;
  disabled?: boolean;
  sha?: string;
  created_at?: string;
  updated_at?: string;
}

export interface SearchParams {
  query?: string;
  author?: string;
  filter?: string;
  sort?: 'lastModified' | 'createdAt' | 'downloads' | 'likes';
  order?: 'asc' | 'desc';
  limit?: number;
  offset?: number;
  formats?: string[];
  pipeline_tag?: string;
  library?: string[];
  language?: string[];
  license?: string[];
}

export interface SearchResult {
  models: HFModel[];
  totalCount: number;
  hasMore: boolean;
  nextOffset?: number;
}

// Ответ от API поиска
export interface HFSearchResponse {
  models: any[];
  numItemsOnPage: number;
  numTotalItems: number;
  pageIndex: number;
}

// Информация о файлах модели
export interface HFModelFile {
  path: string;
  size: number;
  blob_id: string;
  lfs?: {
    oid: string;
    size: number;
    pointer_size: number;
  };
}