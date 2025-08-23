// Сервис для работы с Hugging Face API
// Полная интеграция с реальным API Hugging Face Hub

// Re-export from modular structure
export * from './huggingface/interfaces';
export * from './huggingface/models';
export * from './huggingface/utils';
export * from './huggingface/api';
export * from './huggingface/mock';
export { HuggingFaceService } from './huggingface/index';

// Экспорт экземпляра сервиса
import { huggingFaceService } from './huggingface/index';
export { huggingFaceService };