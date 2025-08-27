// Main Hugging Face service module

import type { HFModel, SearchParams, SearchResult } from './interfaces';
import { buildSearchUrl } from './utils';
import { transformApiModel } from './models';
import { HuggingFaceAPI } from './api';
import { getMockModels } from './mock';

export class HuggingFaceService {
  private api: HuggingFaceAPI;

  constructor(apiKey?: string) {
    this.api = new HuggingFaceAPI(apiKey);
  }

  // Поиск моделей через реальный API
  async searchModels(params: SearchParams): Promise<SearchResult> {
    try {
      const searchUrl = buildSearchUrl(params);
      const response = await this.api.fetchWithRetry(searchUrl);

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
        throw new Error(
          'Invalid API response: expected array of models or object with models field',
        );
      }

      // Преобразуем данные API в наш формат
      const models = await Promise.all(
        modelsArray.map(async (model: any) => {
          const modelFiles = await this.api.getModelFiles(model.id).catch(() => []);
          return transformApiModel(model, modelFiles);
        }),
      );

      const limit = params.limit || 20;
      const offset = params.offset || 0;
      const hasMore = models.length === limit;
      const nextOffset = hasMore ? offset + limit : undefined;

      return {
        models,
        totalCount:
          typeof data === 'object' && !Array.isArray(data) && data.numTotalItems
            ? data.numTotalItems
            : models.length + offset,
        hasMore,
        nextOffset,
      };
    } catch (error) {
      console.error('Error searching models:', error);
      // Fallback на mock данные при ошибке API
      console.warn('Falling back to mock data due to API error');
      const mockModels = getMockModels(params);
      const limit = params.limit || 20;
      const offset = params.offset || 0;
      const hasMore = offset + limit < mockModels.length;
      const nextOffset = hasMore ? offset + limit : undefined;

      return {
        models: mockModels,
        totalCount: mockModels.length,
        hasMore,
        nextOffset,
      };
    }
  }

  // Получение информации о конкретной модели
  async getModelInfo(modelId: string): Promise<HFModel | null> {
    try {
      const response = await this.api.fetchWithRetry(
        `https://huggingface.co/api/models/${modelId}`,
      );

      if (!response.ok) {
        if (response.status === 404) {
          return null;
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const model = await response.json();
      const modelFiles = await this.api.getModelFiles(modelId).catch(() => []);

      return transformApiModel(model, modelFiles);
    } catch (error) {
      console.error('Error getting model info:', error);
      // Fallback на mock данные
      const mockModels = getMockModels({ query: '', formats: [] });
      return mockModels.find((m) => m.id === modelId) || null;
    }
  }

  // Получение популярных моделей
  async getPopularModels(limit: number = 10): Promise<HFModel[]> {
    try {
      const result = await this.searchModels({
        sort: 'downloads',
        order: 'desc',
        limit,
      });
      return result.models;
    } catch (error) {
      console.error('Error getting popular models:', error);
      // Fallback на mock данные
      const mockModels = getMockModels({ query: '', formats: [] });
      return mockModels.slice(0, limit);
    }
  }

  // Получение детальной информации о модели
  async getModelDetails(modelId: string): Promise<HFModel | null> {
    try {
      const url = `https://huggingface.co/api/models/${modelId}`;
      const response = await this.api.fetchWithRetry(url);

      if (!response.ok) {
        if (response.status === 404) {
          return null;
        }
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();

      // Загружаем описание из README
      const readmeDescription = await this.api.getModelReadme(modelId);

      // Если есть описание из README, используем его
      if (readmeDescription) {
        data.description = readmeDescription;
      }

      return transformApiModel(data, []);
    } catch (error) {
      console.error('Ошибка при получении деталей модели:', error);
      return null;
    }
  }
}

// Экспорт экземпляра сервиса
export const huggingFaceService = new HuggingFaceService();
