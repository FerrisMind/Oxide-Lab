import { invoke } from '@tauri-apps/api/core';
import type { SttDownloadRequest, SttDownloadResponse, SttSettings } from '$lib/types/stt';

export async function getSttSettings(): Promise<SttSettings> {
  try {
    return await invoke<SttSettings>('get_stt_settings');
  } catch (error) {
    console.error('Failed to load STT settings:', error);
    throw error;
  }
}

export async function setSttSettings(settings: SttSettings): Promise<void> {
  try {
    await invoke('set_stt_settings', { settings });
  } catch (error) {
    console.error('Failed to save STT settings:', error);
    throw error;
  }
}

export async function downloadSttModel(
  request: SttDownloadRequest,
): Promise<SttDownloadResponse> {
  try {
    return await invoke<SttDownloadResponse>('download_stt_model', { req: request });
  } catch (error) {
    console.error('Failed to download STT model:', error);
    throw error;
  }
}
