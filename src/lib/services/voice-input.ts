/**
 * Voice Input Service
 * 
 * Service for managing voice recording and transcription through Tauri backend.
 */

export type VoiceCapture = {
    stop: (language?: string | null) => Promise<string>;
    cancel: () => Promise<void>;
};

export async function startVoiceCapture(): Promise<VoiceCapture> {
    // TODO: Integrate with Tauri backend
    // Command: invoke('start_voice_recording')
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('start_voice_recording');

    return {
        stop: async (language?: string | null) => {
            // TODO: Integrate with Tauri backend
            // Command: invoke('stop_voice_recording_and_transcribe', { language })
            return invoke<string>('stop_voice_recording_and_transcribe', { language: language ?? null });
        },
        cancel: async () => {
            // TODO: Integrate with Tauri backend
            // Command: invoke('cancel_voice_recording')
            await invoke('cancel_voice_recording');
        },
    };
}
