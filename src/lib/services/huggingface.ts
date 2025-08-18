// Сервис для работы с Hugging Face API
// Полная интеграция с реальным API Hugging Face Hub

// Интерфейс модели из Hugging Face API
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

// Расширенные параметры поиска
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
interface HFSearchResponse {
  models: any[];
  numItemsOnPage: number;
  numTotalItems: number;
  pageIndex: number;
}

// Информация о файлах модели
interface HFModelFile {
  path: string;
  size: number;
  blob_id: string;
  lfs?: {
    oid: string;
    size: number;
    pointer_size: number;
  };
}

export class HuggingFaceService {
  private baseUrl = 'https://huggingface.co/api';
  private apiKey?: string;
  private retryAttempts = 3;
  private retryDelay = 1000;

  constructor(apiKey?: string) {
    this.apiKey = apiKey;
  }

  // Поиск моделей через реальный API
  async searchModels(params: SearchParams): Promise<SearchResult> {
    try {
      const searchUrl = this.buildSearchUrl(params);
      const response = await this.fetchWithRetry(searchUrl);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      
      // API Hugging Face возвращает массив моделей напрямую, а не объект с полем models
      let modelsArray: any[];
      if (Array.isArray(data)) {
        modelsArray = data;
      } else if (data.models && Array.isArray(data.models)) {
        modelsArray = data.models;
      } else {
        throw new Error('Invalid API response: expected array of models or object with models field');
      }
      
      // Преобразуем данные API в наш формат
      const models = await Promise.all(
        modelsArray.map(async (model: any) => {
          const modelFiles = await this.getModelFiles(model.id).catch(() => []);
          return this.transformApiModel(model, modelFiles);
        })
      );
      
      const limit = params.limit || 20;
      const offset = params.offset || 0;
      const hasMore = models.length === limit;
      const nextOffset = hasMore ? offset + limit : undefined;
      
      return {
        models,
        totalCount: (typeof data === 'object' && !Array.isArray(data) && data.numTotalItems) 
          ? data.numTotalItems 
          : models.length + offset,
        hasMore,
        nextOffset
      };
    } catch (error) {
      console.error('Error searching models:', error);
      // Fallback на mock данные при ошибке API
      console.warn('Falling back to mock data due to API error');
      const mockModels = this.getMockModels(params);
      const limit = params.limit || 20;
      const offset = params.offset || 0;
      const hasMore = offset + limit < mockModels.length;
      const nextOffset = hasMore ? offset + limit : undefined;
      
      return {
        models: mockModels,
        totalCount: mockModels.length,
        hasMore,
        nextOffset
      };
    }
  }

  // Получение информации о конкретной модели
  async getModelInfo(modelId: string): Promise<HFModel | null> {
    try {
      const response = await this.fetchWithRetry(`${this.baseUrl}/models/${modelId}`);
      
      if (!response.ok) {
        if (response.status === 404) {
          return null;
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const model = await response.json();
      const modelFiles = await this.getModelFiles(modelId).catch(() => []);
      
      return this.transformApiModel(model, modelFiles);
    } catch (error) {
      console.error('Error getting model info:', error);
      // Fallback на mock данные
      const mockModels = this.getMockModels({ query: '', formats: [] });
      return mockModels.find(m => m.id === modelId) || null;
    }
  }

  // Получение популярных моделей
  async getPopularModels(limit: number = 10): Promise<HFModel[]> {
    try {
      const result = await this.searchModels({
        sort: 'downloads',
        order: 'desc',
        limit
      });
      return result.models;
    } catch (error) {
      console.error('Error getting popular models:', error);
      // Fallback на mock данные
      const mockModels = this.getMockModels({ query: '', formats: [] });
      return mockModels.slice(0, limit);
    }
  }

  // Построение URL для поиска
  private buildSearchUrl(params: SearchParams): string {
    const baseUrl = 'https://huggingface.co/api/models';
    const searchParams = new URLSearchParams();

    if (params.query) {
      searchParams.append('search', params.query);
    }

    if (params.author) {
      searchParams.append('author', params.author);
    }

    if (params.pipeline_tag) {
      searchParams.append('pipeline_tag', params.pipeline_tag);
    }

    // Handle formats using filter parameter (correct API approach)
    if (params.formats?.length) {
      console.log('Formats received:', params.formats);
      params.formats.forEach(format => {
        const formatMap: { [key: string]: string } = {
          'gguf': 'gguf',
          'safetensors': 'safetensors',
          'pytorch': 'pytorch',
          'tensorflow': 'tensorflow',
          'onnx': 'onnx'
        };
        
        const filterValue = formatMap[format.toLowerCase()] || format.toLowerCase();
        searchParams.append('filter', filterValue);
        console.log(`Format ${format} added as filter: ${filterValue}`);
      });
    }
    
    // Handle libraries using library parameter
    if (params.library?.length) {
      console.log('Libraries received:', params.library);
      params.library.forEach(lib => {
        searchParams.append('library', lib);
        console.log(`Library added: ${lib}`);
      });
    }

    if (params.language?.length) {
      params.language.forEach(lang => {
        searchParams.append('language', lang);
      });
    }

    if (params.license?.length) {
      params.license.forEach(lic => {
        searchParams.append('license', lic);
      });
    }

    if (params.sort) {
      const sortMap = {
        'lastModified': 'lastModified',
        'createdAt': 'createdAt',
        'downloads': 'downloads',
        'likes': 'likes'
      };
      searchParams.append('sort', sortMap[params.sort]);
    }

    if (params.order) {
      searchParams.append('direction', params.order === 'desc' ? '-1' : '1');
    }

    if (params.limit) {
      searchParams.append('limit', params.limit.toString());
    }

    if (params.offset) {
      searchParams.append('skip', params.offset.toString());
    }

    const url = `${baseUrl}?${searchParams.toString()}`;
    console.log('Final search URL:', url);
    return url;
  }

  // Fetch с retry логикой
  private async fetchWithRetry(url: string, options?: RequestInit): Promise<Response> {
    let lastError: Error;
    
    for (let attempt = 0; attempt < this.retryAttempts; attempt++) {
      try {
        const response = await fetch(url, {
          ...options,
          headers: {
            ...this.getHeaders(),
            ...options?.headers
          }
        });
        
        if (response.ok || response.status === 404) {
          return response;
        }
        
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      } catch (error) {
        lastError = error as Error;
        
        if (attempt < this.retryAttempts - 1) {
          await this.delay(this.retryDelay * Math.pow(2, attempt));
        }
      }
    }
    
    throw lastError!;
  }

  // Получение файлов модели
  async getModelFiles(modelId: string): Promise<HFModelFile[]> {
    try {
      const response = await this.fetchWithRetry(`${this.baseUrl}/models/${modelId}/tree/main`);
      
      if (!response.ok) {
        return [];
      }
      
      return await response.json();
    } catch (error) {
      console.error('Error getting model files:', error);
      return [];
    }
  }

  // Трансформация данных API в наш формат
  private transformApiModel(apiModel: any, files: HFModelFile[]): HFModel {
    // Определяем доступные форматы на основе файлов
    const formats = this.extractFormats(files);
    
    return {
      id: apiModel.id || apiModel.modelId,
      name: apiModel.id?.split('/').pop() || apiModel.modelId?.split('/').pop() || 'Unknown',
      description: apiModel.description || undefined,
      downloads: apiModel.downloads || 0,
      likes: apiModel.likes || 0,
      tags: apiModel.tags || [],
      author: apiModel.id?.split('/')[0] || apiModel.author || 'Unknown',
      lastModified: apiModel.lastModified || apiModel.updated_at || new Date().toISOString(),
      modelType: apiModel.pipeline_tag || 'text-generation',
      formats,
      pipeline_tag: apiModel.pipeline_tag,
      library_name: apiModel.library_name,
      language: apiModel.language,
      license: apiModel.license,
      size: this.calculateModelSize(files),
      private: apiModel.private,
      gated: apiModel.gated,
      disabled: apiModel.disabled,
      sha: apiModel.sha,
      created_at: apiModel.created_at,
      updated_at: apiModel.updated_at
    };
  }

  // Извлечение форматов из файлов модели
  private extractFormats(files: HFModelFile[]): string[] {
    const formats = new Set<string>();
    
    files.forEach(file => {
      const extension = file.path.split('.').pop()?.toLowerCase();
      
      if (extension === 'gguf') {
        formats.add('gguf');
      } else if (extension === 'safetensors') {
        formats.add('safetensors');
      } else if (extension === 'bin') {
        formats.add('pytorch');
      } else if (file.path.includes('pytorch_model')) {
        formats.add('pytorch');
      } else if (file.path.includes('tf_model')) {
        formats.add('tensorflow');
      }
    });
    
    return Array.from(formats);
  }

  // Вычисление размера модели
  private calculateModelSize(files: HFModelFile[]): number {
    return files.reduce((total, file) => total + (file.size || 0), 0);
  }

  // Задержка для retry
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // Получение заголовков для API запросов
  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    return headers;
  }

  // Mock данные для демонстрации
  private getMockModels(params: SearchParams): HFModel[] {
    const allModels: HFModel[] = [
      {
        id: 'microsoft/DialoGPT-medium',
        name: 'DialoGPT-medium',
        description: 'A conversational language model based on the GPT-2 architecture, trained on a large dataset of conversations.',
        downloads: 2500000,
        likes: 4500,
        tags: ['dialogue', 'gpt', 'conversational', 'english'],
        author: 'microsoft',
        lastModified: '2023-12-15T10:30:00Z',
        modelType: 'causal-lm',
        formats: ['gguf', 'safetensors']
      },
      {
        id: 'microsoft/DialoGPT-large',
        name: 'DialoGPT-large',
        description: 'A large conversational language model based on the GPT-2 architecture, providing more nuanced responses.',
        downloads: 1800000,
        likes: 3200,
        tags: ['dialogue', 'gpt', 'conversational', 'english', 'large'],
        author: 'microsoft',
        lastModified: '2023-12-10T14:20:00Z',
        modelType: 'causal-lm',
        formats: ['gguf']
      },
      {
        id: 'microsoft/DialoGPT-small',
        name: 'DialoGPT-small',
        description: 'A smaller, faster conversational language model suitable for resource-constrained environments.',
        downloads: 3200000,
        likes: 2800,
        tags: ['dialogue', 'gpt', 'conversational', 'english', 'small'],
        author: 'microsoft',
        lastModified: '2023-12-12T09:15:00Z',
        modelType: 'causal-lm',
        formats: ['safetensors']
      },
      {
        id: 'meta-llama/Llama-2-7b-chat-hf',
        name: 'Llama-2-7b-chat-hf',
        description: 'A 7B parameter chat model from Meta, fine-tuned for dialogue generation.',
        downloads: 5000000,
        likes: 8900,
        tags: ['llama', 'chat', 'conversational', 'english', '7b'],
        author: 'meta-llama',
        lastModified: '2023-12-20T08:45:00Z',
        modelType: 'causal-lm',
        formats: ['gguf', 'safetensors']
      },
      {
        id: 'mistralai/Mistral-7B-Instruct-v0.2',
        name: 'Mistral-7B-Instruct-v0.2',
        description: 'A 7B parameter instruction-tuned model from Mistral AI, optimized for following user instructions.',
        downloads: 3800000,
        likes: 7200,
        tags: ['mistral', 'instruct', 'instruction', 'english', '7b'],
        author: 'mistralai',
        lastModified: '2023-12-18T12:30:00Z',
        modelType: 'causal-lm',
        formats: ['gguf']
      },
      {
        id: 'google/gemma-2b-it',
        name: 'Gemma-2b-it',
        description: 'A 2B parameter instruction-tuned model from Google, designed for efficient inference.',
        downloads: 2100000,
        likes: 4100,
        tags: ['gemma', 'instruct', 'instruction', 'english', '2b'],
        author: 'google',
        lastModified: '2023-12-22T15:20:00Z',
        modelType: 'causal-lm',
        formats: ['safetensors']
      }
    ];

    let filteredModels = allModels;

    // Фильтрация по форматам
    if (params.formats && params.formats.length > 0) {
      filteredModels = filteredModels.filter(model => 
        params.formats!.some(format => model.formats.includes(format))
      );
    }

    // Фильтрация по поисковому запросу
    if (params.query && params.query.trim()) {
      const searchLower = params.query.toLowerCase();
      filteredModels = filteredModels.filter(model =>
        model.name.toLowerCase().includes(searchLower) ||
        (model.description && model.description.toLowerCase().includes(searchLower)) ||
        model.tags.some(tag => tag.toLowerCase().includes(searchLower)) ||
        model.author.toLowerCase().includes(searchLower)
      );
    }

    // Сортировка
    const sortBy = params.sort || 'downloads';
    const order = params.order || 'desc';

    filteredModels.sort((a, b) => {
      let aValue: any;
      let bValue: any;
      switch (sortBy) {
        case 'likes':
          aValue = a.likes;
          bValue = b.likes;
          break;
        case 'lastModified':
          aValue = a.lastModified;
          bValue = b.lastModified;
          break;
        case 'downloads':
        default:
          aValue = a.downloads;
          bValue = b.downloads;
          break;
      }

      if (typeof aValue === 'string') {
        aValue = aValue.toLowerCase();
        bValue = bValue.toLowerCase();
      }

      if (order === 'asc') {
        return aValue > bValue ? 1 : -1;
      } else {
        return aValue < bValue ? 1 : -1;
      }
    });

    // Пагинация
    const limit = params.limit || 20;
    const offset = params.offset || 0;
    
    return filteredModels.slice(offset, offset + limit);
  }

  // Получение README файла модели
  async getModelReadme(modelId: string): Promise<string | null> {
    try {
      // Сначала пытаемся получить README без авторизации
      const url = `https://huggingface.co/${modelId}/resolve/main/README.md`;
      
      // Делаем запрос без использования fetchWithRetry для лучшего контроля ошибок
      const response = await fetch(url, {
        headers: {
          'User-Agent': 'Mozilla/5.0 (compatible; HuggingFace-Client)'
        }
      });
      
      if (response.ok) {
        const readmeContent = await response.text();
        return this.extractDescriptionFromReadme(readmeContent);
      }
      
      // Если получили 401 или другую ошибку, пытаемся получить через API
      if (response.status === 401 || response.status === 403) {
        console.warn(`README недоступен для модели ${modelId} (${response.status}), используем API`);
        return await this.getDescriptionFromApi(modelId);
      }
      
      return null;
    } catch (error) {
      console.warn('Ошибка при получении README модели, пытаемся через API:', error);
      return await this.getDescriptionFromApi(modelId);
    }
  }

  // Извлечение описания из README контента
  private extractDescriptionFromReadme(readmeContent: string): string | null {
    const lines = readmeContent.split('\n');
    let description = '';
    let foundDescription = false;
    
    for (const line of lines) {
      const trimmedLine = line.trim();
      
      // Пропускаем заголовки, метаданные и пустые строки
      if (trimmedLine.startsWith('#') || 
          trimmedLine.startsWith('---') ||
          trimmedLine.startsWith('license:') ||
          trimmedLine.startsWith('language:') ||
          trimmedLine.startsWith('tags:') ||
          trimmedLine.startsWith('datasets:') ||
          trimmedLine.startsWith('pipeline_tag:') ||
          trimmedLine.startsWith('library_name:') ||
          trimmedLine === '') {
        continue;
      }
      
      // Если нашли первый содержательный абзац
      if (!foundDescription && trimmedLine.length > 0) {
        description = trimmedLine;
        foundDescription = true;
        continue;
      }
      
      // Продолжаем добавлять строки к описанию, пока не встретим пустую строку
      if (foundDescription && trimmedLine.length > 0) {
        description += ' ' + trimmedLine;
      } else if (foundDescription && trimmedLine === '') {
        break;
      }
    }
    
    return description || null;
  }

  // Получение описания через API как fallback
  private async getDescriptionFromApi(modelId: string): Promise<string | null> {
    try {
      const url = `${this.baseUrl}/models/${modelId}`;
      const response = await this.fetchWithRetry(url);
      
      if (response.ok) {
        const modelData = await response.json();
        return modelData.description || null;
      }
      
      return null;
    } catch (error) {
      console.error('Ошибка при получении описания через API:', error);
      return null;
    }
  }

  // Получение детальной информации о модели
  async getModelDetails(modelId: string): Promise<HFModel | null> {
    try {
      const url = `${this.baseUrl}/models/${modelId}`;
      const response = await this.fetchWithRetry(url);
      
      if (!response.ok) {
        if (response.status === 404) {
          return null;
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      
      // Загружаем описание из README
      const readmeDescription = await this.getModelReadme(modelId);
      
      // Если есть описание из README, используем его
      if (readmeDescription) {
        data.description = readmeDescription;
      }
      
      return this.transformApiModel(data, []);
    } catch (error) {
      console.error('Ошибка при получении деталей модели:', error);
      return null;
    }
  }


}

// Экспорт экземпляра сервиса
export const huggingFaceService = new HuggingFaceService();
