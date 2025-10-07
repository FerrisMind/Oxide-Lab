import { invoke } from '@tauri-apps/api/core';

/**
 * Global store for experimental features state
 * Provides reactive access to experimental features enabled/disabled state
 */
class ExperimentalFeaturesStore {
  private _enabled = $state(false);
  private _loading = $state(false);
  private _initialized = $state(false);

  constructor() {
    this.loadState();
  }

  get enabled() {
    return this._enabled;
  }

  get loading() {
    return this._loading;
  }

  get initialized() {
    return this._initialized;
  }

  async loadState() {
    try {
      this._loading = true;
      this._enabled = await invoke<boolean>('get_experimental_features_enabled');
      this._initialized = true;
    } catch (err) {
      console.warn('Failed to load experimental features state:', err);
      this._enabled = false; // Default to disabled
      this._initialized = true;
    } finally {
      this._loading = false;
    }
  }

  async setEnabled(enabled: boolean) {
    try {
      // Не показываем состояние загрузки при переключении
      await invoke('set_experimental_features_enabled', { enabled });
      this._enabled = enabled;
    } catch (err) {
      console.error('Failed to save experimental features state:', err);
      throw err;
    }
  }

  async toggle() {
    await this.setEnabled(!this._enabled);
  }
}

// Create singleton instance
export const experimentalFeatures = new ExperimentalFeaturesStore();
