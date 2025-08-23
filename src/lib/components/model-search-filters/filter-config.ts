import type { FilterOption } from './filter-types';

// Available filter options
export const filterConfig = {
  formats: [
    { id: 'gguf', label: 'GGUF', color: '#10b981' },
    { id: 'safetensors', label: 'Safetensors', color: '#3b82f6' }
  ] as FilterOption[],
  
  pipelineTags: [
    { id: 'text-generation', label: 'Text Generation' },
    { id: 'text2text-generation', label: 'Text-to-Text' },
    { id: 'conversational', label: 'Conversational' },
    { id: 'question-answering', label: 'Q&A' },
    { id: 'summarization', label: 'Summarization' },
    { id: 'translation', label: 'Translation' },
    { id: 'text-classification', label: 'Classification' },
    { id: 'feature-extraction', label: 'Embeddings' }
  ] as FilterOption[],
  
  libraries: [
    { id: 'transformers', label: 'Transformers' },
    { id: 'pytorch', label: 'PyTorch' },
    { id: 'tensorflow', label: 'TensorFlow' },
    { id: 'jax', label: 'JAX' },
    { id: 'onnx', label: 'ONNX' },
    { id: 'safetensors', label: 'SafeTensors' }
  ] as FilterOption[],
  
  languages: [
    { id: 'en', label: 'English' },
    { id: 'ru', label: 'Russian' },
    { id: 'zh', label: 'Chinese' },
    { id: 'es', label: 'Spanish' },
    { id: 'fr', label: 'French' },
    { id: 'de', label: 'German' },
    { id: 'ja', label: 'Japanese' },
    { id: 'ko', label: 'Korean' }
  ] as FilterOption[],
  
  licenses: [
    { id: 'apache-2.0', label: 'Apache 2.0' },
    { id: 'mit', label: 'MIT' },
    { id: 'cc-by-4.0', label: 'CC BY 4.0' },
    { id: 'cc-by-sa-4.0', label: 'CC BY-SA 4.0' },
    { id: 'gpl-3.0', label: 'GPL 3.0' },
    { id: 'other', label: 'Other' }
  ] as FilterOption[]
};