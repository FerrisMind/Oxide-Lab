import { invoke } from '@tauri-apps/api/core';

export type VoiceCapture = {
  stop: () => Promise<string>;
  cancel: () => Promise<void>;
};

export async function startVoiceCapture(): Promise<VoiceCapture> {
  await invoke('start_voice_recording');
  return {
    stop: async () => invoke<string>('stop_voice_recording_and_transcribe'),
    cancel: async () => {
      await invoke('cancel_voice_recording');
    },
  };
}
