// Utility functions for Hugging Face service

import type { SearchParams } from './interfaces';
import type { HFModelFile } from './interfaces';

// Построение URL для поиска
export function buildSearchUrl(params: SearchParams): string {
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

// Извлечение форматов из файлов модели
export function extractFormats(files: HFModelFile[]): string[] {
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
export function calculateModelSize(files: HFModelFile[]): number {
  return files.reduce((total, file) => total + (file.size || 0), 0);
}