// Model-related operations for Hugging Face service

import type { HFModel, HFModelFile } from './interfaces';
import { extractFormats, calculateModelSize } from './utils';

// Трансформация данных API в наш формат
export function transformApiModel(apiModel: any, files: HFModelFile[]): HFModel {
  // Определяем доступные форматы на основе файлов
  const formats = extractFormats(files);
  
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
    size: calculateModelSize(files),
    private: apiModel.private,
    gated: apiModel.gated,
    disabled: apiModel.disabled,
    sha: apiModel.sha,
    created_at: apiModel.created_at,
    updated_at: apiModel.updated_at
  };
}

// Извлечение описания из README контента
export function extractDescriptionFromReadme(readmeContent: string): string | null {
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

// Удаляем YAML-фронт-маттер из начала README (строки между --- ... ---)
export function stripFrontMatter(content: string): string {
  return content.replace(/^---[\s\S]*?---\s*/m, "");
}