// Сервис для работы с Hugging Face API
// В будущем здесь будет реальная интеграция с API

export interface HFModel {
  id: string;
  name: string;
  description: string;
  downloads: number;
  likes: number;
  tags: string[];
  author: string;
  lastModified: string;
  modelType: string;
  formats: string[];
  // Дополнительные поля для реального API
  pipeline_tag?: string;
  library_name?: string;
  language?: string;
  license?: string;
  size?: number;
}

export interface SearchParams {
  query: string;
  formats: string[];
  limit?: number;
  offset?: number;
  sort?: 'downloads' | 'likes' | 'updated';
  order?: 'asc' | 'desc';
}

export class HuggingFaceService {
  private baseUrl = 'https://huggingface.co/api';
  private apiKey?: string;

  constructor(apiKey?: string) {
    this.apiKey = apiKey;
  }

  // Поиск моделей
  async searchModels(params: SearchParams): Promise<HFModel[]> {
    try {
      // TODO: Реальная интеграция с HF API
      // const response = await fetch(`${this.baseUrl}/models`, {
      //   headers: this.getHeaders(),
      //   body: JSON.stringify(params)
      // });
      
      // Пока возвращаем mock данные
      return this.getMockModels(params);
    } catch (error) {
      console.error('Error searching models:', error);
      throw new Error('Failed to search models');
    }
  }

  // Получение информации о конкретной модели
  async getModelInfo(modelId: string): Promise<HFModel | null> {
    try {
      // TODO: Реальная интеграция с HF API
      // const response = await fetch(`${this.baseUrl}/models/${modelId}`, {
      //   headers: this.getHeaders()
      // });
      
      // Пока возвращаем mock данные
      const mockModels = this.getMockModels({ query: '', formats: [] });
      return mockModels.find(m => m.id === modelId) || null;
    } catch (error) {
      console.error('Error getting model info:', error);
      throw new Error('Failed to get model info');
    }
  }

  // Получение популярных моделей
  async getPopularModels(limit: number = 10): Promise<HFModel[]> {
    try {
      // TODO: Реальная интеграция с HF API
      const mockModels = this.getMockModels({ query: '', formats: [] });
      return mockModels.slice(0, limit);
    } catch (error) {
      console.error('Error getting popular models:', error);
      throw new Error('Failed to get popular models');
    }
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
    if (params.formats.length > 0) {
      filteredModels = filteredModels.filter(model => 
        params.formats.some(format => model.formats.includes(format))
      );
    }

    // Фильтрация по поисковому запросу
    if (params.query.trim()) {
      const searchLower = params.query.toLowerCase();
      filteredModels = filteredModels.filter(model =>
        model.name.toLowerCase().includes(searchLower) ||
        model.description.toLowerCase().includes(searchLower) ||
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
        case 'updated':
          // map 'updated' to `lastModified`
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
}

// Экспорт экземпляра сервиса
export const huggingFaceService = new HuggingFaceService();
