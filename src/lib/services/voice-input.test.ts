import { describe, expect, it } from 'vitest';
import { mixToMonoChannels, resampleLinear } from './voice-input';

describe('voice-input utils', () => {
  it('mixes multiple channels into mono', () => {
    const ch1 = new Float32Array([1, -1, 0.5]);
    const ch2 = new Float32Array([0, 1, -0.5]);
    const result = mixToMonoChannels([ch1, ch2]);
    expect(Array.from(result)).toEqual([0.5, 0, 0]);
  });

  it('resamples audio to a lower sample rate', () => {
    const input = new Float32Array([0, 1, 0, 1, 0, 1]);
    const output = resampleLinear(input, 48_000, 16_000);
    expect(output.length).toBe(2);
    expect(output[0]).toBeCloseTo(0, 5);
    expect(output[1]).toBeCloseTo(1, 5);
  });
});
