// API-related operations for Hugging Face service

import type {
  HFModel as _HFModel,
  HFModelFile,
  HFSearchResponse as _HFSearchResponse,
} from './interfaces';
import { buildSearchUrl as _buildSearchUrl } from './utils';
import {
  transformApiModel as _transformApiModel,
  extractDescriptionFromReadme as _extractDescriptionFromReadme,
  stripFrontMatter,
} from './models';

export class HuggingFaceAPI {
  private baseUrl = 'https://huggingface.co/api';
  private apiKey?: string;
  private retryAttempts = 3;
  private retryDelay = 1000;

  constructor(apiKey?: string) {
    this.apiKey = apiKey;
  }

  // Fetch с retry логикой
  async fetchWithRetry(url: string, options?: RequestInit): Promise<Response> {
    let lastError: Error;

    for (let attempt = 0; attempt < this.retryAttempts; attempt++) {
      try {
        const response = await fetch(url, {
          ...options,
          headers: {
            ...this.getHeaders(),
            ...options?.headers,
          },
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

  // Получение README файла модели
  async getModelReadme(modelId: string): Promise<string | null> {
    try {
      // Сначала пытаемся получить README без авторизации
      const url = `https://huggingface.co/${modelId}/resolve/main/README.md`;

      // Делаем запрос без использования fetchWithRetry для лучшего контроля ошибок
      const response = await fetch(url, {
        headers: {
          'User-Agent': 'Mozilla/5.0 (compatible; HuggingFace-Client)',
        },
      });

      if (response.ok) {
        const readmeContent = await response.text();
        return stripFrontMatter(readmeContent);
      }

      // Если получили 401 или другую ошибку, пытаемся получить через API
      if (response.status === 401 || response.status === 403) {
        console.warn(
          `README недоступен для модели ${modelId} (${response.status}), используем API`,
        );
        return await this.getDescriptionFromApi(modelId);
      }

      return null;
    } catch (error) {
      console.warn('Ошибка при получении README модели, пытаемся через API:', error);
      return await this.getDescriptionFromApi(modelId);
    }
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

  // Задержка для retry
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
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
}
