import { writable, type Writable } from 'svelte/store';
import type { ChatMessage } from '$lib/chat/types';

export type ChatPersistedState = {
  // Model selection
  modelPath: string;
  repoId: string;
  revision: string;
  hubGgufFilename: string;
  format: 'gguf' | 'hub_gguf' | 'hub_safetensors';

  // Chat state
  prompt: string;
  messages: ChatMessage[];
  busy: boolean;
  isLoaded: boolean;
  errorText: string;

  // Loading / unloading state
  isLoadingModel: boolean;
  loadingProgress: number;
  loadingStage: string; // 'model' | 'tokenizer' | 'complete' | ''
  isCancelling: boolean;
  isUnloadingModel: boolean;
  unloadingProgress: number;

  // Inference params
  temperature: number;
  temperature_enabled: boolean;
  top_k_enabled: boolean;
  top_k_value: number;
  top_p_enabled: boolean;
  top_p_value: number;
  min_p_enabled: boolean;
  min_p_value: number;
  repeat_penalty_enabled: boolean;
  repeat_penalty_value: number;
  ctx_limit_value: number;
  enable_thinking: boolean;
  use_custom_params: boolean;

  // Device state
  use_gpu: boolean;
  cuda_available: boolean;
  cuda_build: boolean;
  current_device: string;
};

export function getDefaultChatState(): ChatPersistedState {
  return {
    modelPath: '',
    repoId: '',
    revision: '',
    hubGgufFilename: '',
    format: 'gguf',

    prompt: '',
    messages: [],
    busy: false,
    isLoaded: false,
    errorText: '',

    isLoadingModel: false,
    loadingProgress: 0,
    loadingStage: '',
    isCancelling: false,
    isUnloadingModel: false,
    unloadingProgress: 0,

    temperature: 0.8,
    temperature_enabled: false,
    top_k_enabled: false,
    top_k_value: 40,
    top_p_enabled: false,
    top_p_value: 0.9,
    min_p_enabled: false,
    min_p_value: 0.05,
    repeat_penalty_enabled: false,
    repeat_penalty_value: 1.1,
    ctx_limit_value: 4096,
    enable_thinking: false,
    use_custom_params: false,

    use_gpu: false,
    cuda_available: false,
    cuda_build: false,
    current_device: 'CPU',
  };
}

export const chatState: Writable<ChatPersistedState> = writable(getDefaultChatState());

// Indicates whether the Chat UI component is currently mounted.
export const chatUiMounted = writable(false);
