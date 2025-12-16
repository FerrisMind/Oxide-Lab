import '@testing-library/jest-dom';

import { vi } from 'vitest';

vi.mock('@tauri-apps/plugin-store', () => {
  const store = {
    get: vi.fn(async () => null),
    set: vi.fn(async () => undefined),
    save: vi.fn(async () => undefined),
  };

  return {
    Store: {
      load: vi.fn(async () => store),
    },
  };
});

vi.mock('phosphor-svelte/lib/context.js', () => ({
  setIconContext: vi.fn(),
  getIconContext: vi.fn(() => ({})),
}));
