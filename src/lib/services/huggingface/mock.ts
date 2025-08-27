// Mock data for Hugging Face service

import type { HFModel, SearchParams } from './interfaces';

export function getMockModels(params: SearchParams): HFModel[] {
  const allModels: HFModel[] = [
    {
      id: 'microsoft/DialoGPT-medium',
      name: 'DialoGPT-medium',
      description:
        'A conversational language model based on the GPT-2 architecture, trained on a large dataset of conversations.',
      downloads: 2500000,
      likes: 4500,
      tags: ['dialogue', 'gpt', 'conversational', 'english'],
      author: 'microsoft',
      lastModified: '2023-12-15T10:30:00Z',
      modelType: 'causal-lm',
      formats: ['gguf', 'safetensors'],
    },
    {
      id: 'microsoft/DialoGPT-large',
      name: 'DialoGPT-large',
      description:
        'A large conversational language model based on the GPT-2 architecture, providing more nuanced responses.',
      downloads: 1800000,
      likes: 3200,
      tags: ['dialogue', 'gpt', 'conversational', 'english', 'large'],
      author: 'microsoft',
      lastModified: '2023-12-10T14:20:00Z',
      modelType: 'causal-lm',
      formats: ['gguf'],
    },
    {
      id: 'microsoft/DialoGPT-small',
      name: 'DialoGPT-small',
      description:
        'A smaller, faster conversational language model suitable for resource-constrained environments.',
      downloads: 3200000,
      likes: 2800,
      tags: ['dialogue', 'gpt', 'conversational', 'english', 'small'],
      author: 'microsoft',
      lastModified: '2023-12-12T09:15:00Z',
      modelType: 'causal-lm',
      formats: ['safetensors'],
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
      formats: ['gguf', 'safetensors'],
    },
    {
      id: 'mistralai/Mistral-7B-Instruct-v0.2',
      name: 'Mistral-7B-Instruct-v0.2',
      description:
        'A 7B parameter instruction-tuned model from Mistral AI, optimized for following user instructions.',
      downloads: 3800000,
      likes: 7200,
      tags: ['mistral', 'instruct', 'instruction', 'english', '7b'],
      author: 'mistralai',
      lastModified: '2023-12-18T12:30:00Z',
      modelType: 'causal-lm',
      formats: ['gguf'],
    },
    {
      id: 'google/gemma-2b-it',
      name: 'Gemma-2b-it',
      description:
        'A 2B parameter instruction-tuned model from Google, designed for efficient inference.',
      downloads: 2100000,
      likes: 4100,
      tags: ['gemma', 'instruct', 'instruction', 'english', '2b'],
      author: 'google',
      lastModified: '2023-12-22T15:20:00Z',
      modelType: 'causal-lm',
      formats: ['safetensors'],
    },
  ];

  let filteredModels = allModels;

  // Фильтрация по форматам
  if (params.formats && params.formats.length > 0) {
    filteredModels = filteredModels.filter((model) =>
      params.formats!.some((format) => model.formats.includes(format)),
    );
  }

  // Фильтрация по поисковому запросу
  if (params.query && params.query.trim()) {
    const searchLower = params.query.toLowerCase();
    filteredModels = filteredModels.filter(
      (model) =>
        model.name.toLowerCase().includes(searchLower) ||
        (model.description && model.description.toLowerCase().includes(searchLower)) ||
        model.tags.some((tag) => tag.toLowerCase().includes(searchLower)) ||
        model.author.toLowerCase().includes(searchLower),
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
