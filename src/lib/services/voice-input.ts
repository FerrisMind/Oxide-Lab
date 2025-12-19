export const TARGET_SAMPLE_RATE = 16_000;

export type VoiceCapture = {
  stop: () => Promise<Float32Array>;
  cancel: () => void;
};

export function mixToMonoChannels(channels: Float32Array[]): Float32Array {
  if (channels.length === 0) return new Float32Array();
  if (channels.length === 1) return channels[0];

  const length = channels[0].length;
  const mixed = new Float32Array(length);
  const scale = 1 / channels.length;

  for (let i = 0; i < length; i += 1) {
    let sum = 0;
    for (const channel of channels) {
      sum += channel[i] ?? 0;
    }
    mixed[i] = sum * scale;
  }
  return mixed;
}

export function resampleLinear(
  input: Float32Array,
  inputRate: number,
  outputRate: number,
): Float32Array {
  if (inputRate === outputRate) return input;
  const ratio = inputRate / outputRate;
  const outputLength = Math.max(1, Math.round(input.length / ratio));
  const output = new Float32Array(outputLength);

  for (let i = 0; i < outputLength; i += 1) {
    const position = i * ratio;
    const index = Math.floor(position);
    const nextIndex = Math.min(index + 1, input.length - 1);
    const fraction = position - index;
    output[i] = (1 - fraction) * input[index] + fraction * input[nextIndex];
  }
  return output;
}

export async function startVoiceCapture(): Promise<VoiceCapture> {
  if (typeof navigator === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
    throw new Error('Microphone API is not available');
  }
  if (typeof MediaRecorder === 'undefined') {
    throw new Error('MediaRecorder is not available');
  }
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  const mediaRecorder = new MediaRecorder(stream);
  const chunks: Blob[] = [];

  const stopTracks = () => {
    stream.getTracks().forEach((track) => track.stop());
  };

  mediaRecorder.addEventListener('dataavailable', (event) => {
    if (event.data.size > 0) {
      chunks.push(event.data);
    }
  });

  const started = new Promise<void>((resolve, reject) => {
    mediaRecorder.addEventListener('start', () => resolve(), { once: true });
    mediaRecorder.addEventListener('error', () => reject(new Error('Voice capture failed')), {
      once: true,
    });
  });

  mediaRecorder.start();
  await started;

  return {
    async stop() {
      const stopped = new Promise<void>((resolve, reject) => {
        mediaRecorder.addEventListener('stop', () => resolve(), { once: true });
        mediaRecorder.addEventListener('error', () => reject(new Error('Voice capture failed')), {
          once: true,
        });
      });
      mediaRecorder.stop();
      await stopped;
      stopTracks();

      const blob = new Blob(chunks, { type: mediaRecorder.mimeType || 'audio/webm' });
      const arrayBuffer = await blob.arrayBuffer();
      const audioContext = new AudioContext();
      const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
      await audioContext.close();

      const channels = Array.from({ length: audioBuffer.numberOfChannels }, (_, idx) =>
        audioBuffer.getChannelData(idx),
      );
      const mono = mixToMonoChannels(channels);
      return resampleLinear(mono, audioBuffer.sampleRate, TARGET_SAMPLE_RATE);
    },
    cancel() {
      if (mediaRecorder.state !== 'inactive') {
        mediaRecorder.stop();
      }
      stopTracks();
    },
  };
}
