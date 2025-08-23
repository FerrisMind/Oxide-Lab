// Types and utilities for ModelSearchFilters components

export interface FilterOption {
  id: string;
  label: string;
  color?: string;
}

export interface FilterState {
  selectedFormats: string[];
  selectedPipelineTags: string[];
  selectedLibraries: string[];
  selectedLanguages: string[];
  selectedLicenses: string[];
  authorFilter: string;
  searchQuery: string;
}

export interface SearchEvent {
  query: string;
  formats: string[];
  pipelineTags: string[];
  libraries: string[];
  languages: string[];
  licenses: string[];
  author: string;
}

// Available filter options
export const availableFormats: FilterOption[] = [
  { id: 'gguf', label: 'GGUF', color: '#10b981' },
  { id: 'safetensors', label: 'Safetensors', color: '#3b82f6' }
];

export const availablePipelineTags: FilterOption[] = [
  { id: 'text-generation', label: 'Text Generation' },
  { id: 'text2text-generation', label: 'Text-to-Text' },
  { id: 'conversational', label: 'Conversational' },
  { id: 'question-answering', label: 'Q&A' },
  { id: 'summarization', label: 'Summarization' },
  { id: 'translation', label: 'Translation' },
  { id: 'text-classification', label: 'Classification' },
  { id: 'feature-extraction', label: 'Embeddings' }
];

export const availableLibraries: FilterOption[] = [
  { id: 'transformers', label: 'Transformers' },
  { id: 'pytorch', label: 'PyTorch' },
  { id: 'tensorflow', label: 'TensorFlow' },
  { id: 'jax', label: 'JAX' },
  { id: 'onnx', label: 'ONNX' },
  { id: 'safetensors', label: 'SafeTensors' }
];

export const availableLanguages: FilterOption[] = [
  { id: 'en', label: 'English' },
  { id: 'ru', label: 'Russian' },
  { id: 'zh', label: 'Chinese' },
  { id: 'es', label: 'Spanish' },
  { id: 'fr', label: 'French' },
  { id: 'de', label: 'German' },
  { id: 'ja', label: 'Japanese' },
  { id: 'ko', label: 'Korean' }
];

export const availableLicenses: FilterOption[] = [
  { id: 'apache-2.0', label: 'Apache 2.0' },
  { id: 'mit', label: 'MIT' },
  { id: 'cc-by-4.0', label: 'CC BY 4.0' },
  { id: 'cc-by-sa-4.0', label: 'CC BY-SA 4.0' },
  { id: 'gpl-3.0', label: 'GPL 3.0' },
  { id: 'other', label: 'Other' }
];

// Utility functions
export function toggleArrayItem<T>(array: T[], item: T): T[] {
  if (array.includes(item)) {
    return array.filter(i => i !== item);
  } else {
    return [...array, item];
  }
}

export function hasActiveFilters(state: FilterState): boolean {
  return (
    state.selectedFormats.length > 0 ||
    state.selectedPipelineTags.length > 0 ||
    state.selectedLibraries.length > 0 ||
    state.selectedLanguages.length > 0 ||
    state.selectedLicenses.length > 0 ||
    !!state.searchQuery ||
    !!state.authorFilter
  );
}

export function clearAllFilters(): FilterState {
  return {
    selectedFormats: [],
    selectedPipelineTags: [],
    selectedLibraries: [],
    selectedLanguages: [],
    selectedLicenses: [],
    authorFilter: '',
    searchQuery: ''
  };
}