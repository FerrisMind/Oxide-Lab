import { invoke } from '@tauri-apps/api/core';

export type VoiceCapture = {
  stop: (language?: string | null) => Promise<string>;
  cancel: () => Promise<void>;
};

export async function startVoiceCapture(): Promise<VoiceCapture> {
  await invoke('start_voice_recording');
  return {
    stop: async (language?: string | null) =>
      invoke<string>('stop_voice_recording_and_transcribe', { language: language ?? null }),
    cancel: async () => {
      await invoke('cancel_voice_recording');
    },
  };
}
