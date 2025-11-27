import { invoke } from '@tauri-apps/api/core';
import type { ModelCardDownloadResult, ModelCardsResponse } from '$lib/types/model-cards';

export class ModelCardsService {
  static async getModelCards(): Promise<ModelCardsResponse> {
    return invoke<ModelCardsResponse>('get_model_cards');
  }

  static async importModelCards(path: string): Promise<ModelCardsResponse> {
    return invoke<ModelCardsResponse>('import_model_cards', { config_path: path });
  }

  static async resetModelCards(): Promise<ModelCardsResponse> {
    return invoke<ModelCardsResponse>('reset_model_cards');
  }

  static async downloadModelCardFormat(
    cardId: string,
    format: 'gguf' | 'safetensors',
    modelsRoot: string,
    quantization?: string,
  ): Promise<ModelCardDownloadResult> {
    const payload: Record<string, string> = {
      card_id: cardId,
      format,
      models_root: modelsRoot,
    };
    if (quantization) {
      payload.quantization = quantization;
    }
    return invoke<ModelCardDownloadResult>('download_model_card_format', {
      args: payload,
    });
  }
}
