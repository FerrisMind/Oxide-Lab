import { invoke } from '@tauri-apps/api/core';

// Расширяем тип Window для поддержки __TAURI__
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

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
      console.log('Loading experimental features state...');

      // Проверяем, доступен ли invoke
      if (typeof window !== 'undefined' && window.__TAURI__) {
        console.log('Tauri context detected, invoking get_experimental_features_enabled');
        this._enabled = await invoke<boolean>('get_experimental_features_enabled');
        console.log('Experimental features loaded:', this._enabled);
      } else {
        console.log('Tauri context not available, using default (false)');
        this._enabled = false;
      }

      this._initialized = true;
    } catch (err) {
      const error = err as Error;
      console.warn('Failed to load experimental features state:', error);
      console.log('Error details:', error.message, error.stack);
      this._enabled = false; // Default to disabled
      this._initialized = true;
    } finally {
      this._loading = false;
    }
  }

  async setEnabled(enabled: boolean) {
    try {
      console.log('Setting experimental features to:', enabled);

      // Проверяем, доступен ли invoke
      if (typeof window !== 'undefined' && window.__TAURI__) {
        console.log('Tauri context detected, invoking set_experimental_features_enabled');
        await invoke('set_experimental_features_enabled', { enabled });
      } else {
        console.log('Tauri context not available, skipping save');
      }

      this._enabled = enabled;
      console.log('Experimental features set to:', this._enabled);
    } catch (err) {
      const error = err as Error;
      console.error('Failed to save experimental features state:', error);
      console.log('Error details:', error.message, error.stack);
      throw error;
    }
  }

  async toggle() {
    await this.setEnabled(!this._enabled);
  }
}

// Create singleton instance
export const experimentalFeatures = new ExperimentalFeaturesStore();
