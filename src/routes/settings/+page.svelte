<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { PrecisionPolicy } from '$lib/types';
import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { modelSelectorSearchEnabled } from '$lib/stores/ui-preferences';
  import { t } from '$lib/i18n';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';

  let currentPolicy: PrecisionPolicy = { Default: null };
  let isLoading = $state(true);
  let error: string | null = $state(null);
  const hardwareConcurrency = (() => {
    if (typeof navigator !== 'undefined' && navigator.hardwareConcurrency) {
      return Math.max(1, navigator.hardwareConcurrency);
    }
    return 4;
  })();

  let threadLimit = $state<number | null>(null);
  let threadSliderValue = $state(hardwareConcurrency);
  let threadLimitLoading = $state(true);
  let threadLimitError = $state<string | null>(null);

  // Local reactive variable for experimental features checkbox
  let experimentalFeaturesEnabled = $state(false);
let modelSearchEnabled = $state(true);

  onMount(async () => {
    await loadPrecisionPolicy();
    await loadThreadLimit();
  });

  // Sync local variable with store when experimental features are initialized
  $effect(() => {
    if (experimentalFeatures.initialized) {
      experimentalFeaturesEnabled = experimentalFeatures.enabled;
    }
  });

$effect(() => {
  const unsubscribe = modelSelectorSearchEnabled.subscribe((value) => {
    modelSearchEnabled = value;
  });
  return unsubscribe;
});

  async function loadPrecisionPolicy() {
    try {
      isLoading = true;
      error = null;
      currentPolicy = await invoke('get_precision_policy');
      console.log('Loaded precision policy:', currentPolicy, typeof currentPolicy);
    } catch (err) {
      error = `Failed to load precision policy: ${err}`;
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  async function savePrecisionPolicy(policy: PrecisionPolicy) {
    try {
      isLoading = true;
      error = null;
      await invoke('set_precision_policy', { policy });
      currentPolicy = policy;
    } catch (err) {
      error = `Failed to save precision policy: ${err}`;
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  async function handleExperimentalFeaturesToggle(enabled: boolean) {
    try {
      // Сохраняем состояние без показа загрузки
      await experimentalFeatures.setEnabled(enabled);
      experimentalFeaturesEnabled = enabled; // Update local variable
      await tick();
    } catch (err) {
      console.error('Failed to save experimental features state:', err);
      // Revert local variable on error
      experimentalFeaturesEnabled = experimentalFeatures.enabled;
    }
  }

function handleModelSearchToggle(enabled: boolean) {
  modelSelectorSearchEnabled.set(enabled);
}


  async function loadThreadLimit() {
    threadLimitLoading = true;
    threadLimitError = null;
    try {
      const saved = await invoke<number | null>('get_rayon_thread_limit');
      threadLimit = saved;
      threadSliderValue = saved ?? hardwareConcurrency;
    } catch (err) {
      threadLimitError = `Failed to load thread limit: ${err}`;
      console.error(err);
    } finally {
      threadLimitLoading = false;
    }
  }

  async function applyCustomThreadLimit(limit: number | null) {
    threadLimitLoading = true;
    threadLimitError = null;
    try {
      await invoke('set_rayon_thread_limit', { limit });
      threadLimit = limit;
      threadSliderValue = limit ?? hardwareConcurrency;
    } catch (err) {
      threadLimitError = `Failed to save thread limit: ${err}`;
      console.error(err);
    } finally {
      threadLimitLoading = false;
    }
  }

  function selectPolicy(policyType: 'Default' | 'MemoryEfficient' | 'MaximumPrecision') {
    // Send the policy type as a string to match Rust enum serialization
    let policy: PrecisionPolicy;

    // Create the policy object in the correct format
    switch (policyType) {
      case 'Default':
        policy = { Default: null };
        break;
      case 'MemoryEfficient':
        policy = { MemoryEfficient: null };
        break;
      case 'MaximumPrecision':
        policy = { MaximumPrecision: null };
        break;
      default:
        policy = { Default: null };
    }

    console.log('Sending policy:', policy, 'policyType:', policyType);
    savePrecisionPolicy(policy);
  }

  function isPolicySelected(
    policyType: 'Default' | 'MemoryEfficient' | 'MaximumPrecision',
  ): boolean {
    try {
      console.log(
        'Checking policy selection:',
        policyType,
        'currentPolicy:',
        currentPolicy,
        'type:',
        typeof currentPolicy,
      );

      // Handle case where currentPolicy is a string (serialized enum variant)
      if (typeof currentPolicy === 'string') {
        return currentPolicy === policyType;
      }

      // Handle case where currentPolicy is an object with the policy type as a key
      if (typeof currentPolicy === 'object' && currentPolicy !== null) {
        // Check if it's a tagged enum object like { Default: null }
        if (policyType in currentPolicy) {
          return true;
        }

        // Check if it has a variant property (with proper type checking)
        if (
          typeof currentPolicy === 'object' &&
          currentPolicy !== null &&
          'variant' in currentPolicy
        ) {
          const policyObj = currentPolicy as { variant?: string };
          if (policyObj.variant === policyType) {
            return true;
          }
        }

        // For direct object comparison with { [policyType]: null }
        if (JSON.stringify(currentPolicy) === JSON.stringify({ [policyType]: null })) {
          return true;
        }
      }

      // Fallback: check if string representation contains the policy type
      return String(currentPolicy) === policyType;
    } catch (e) {
      console.warn('Error checking policy selection:', e, 'currentPolicy:', currentPolicy);
      // Last resort fallback
      return String(currentPolicy).includes(policyType);
    }
  }
</script>

<div class="settings-page">
  <div class="settings-header">
    <h1>{$t('settings.title')}</h1>
  </div>

  <div class="settings-section">
    <h2>{$t('settings.precisionPolicy.title')}</h2>
    <p class="settings-description">
      {$t('settings.precisionPolicy.description')}<br />
      <span class="warning-text">
        <b>{$t('settings.precisionPolicy.warning')}</b>
      </span>
    </p>

    {#if isLoading}
      <div class="loading">{$t('settings.precisionPolicy.loading')}</div>
    {:else}
      <div class="precision-options">
        <div
          class="option-card {isPolicySelected('Default') ? 'selected' : ''}"
          role="button"
          tabindex="0"
          onclick={() => selectPolicy('Default')}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              selectPolicy('Default');
            }
          }}
        >
          <h3>{$t('settings.precisionPolicy.options.default.title')}</h3>
          <p>{$t('settings.precisionPolicy.options.default.specs')}</p>
          <p class="option-description">{$t('settings.precisionPolicy.options.default.description')}</p>
        </div>

        <div
          class="option-card {isPolicySelected('MemoryEfficient') ? 'selected' : ''}"
          role="button"
          tabindex="0"
          onclick={() => selectPolicy('MemoryEfficient')}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              selectPolicy('MemoryEfficient');
            }
          }}
        >
          <h3>{$t('settings.precisionPolicy.options.memoryEfficient.title')}</h3>
          <p>{$t('settings.precisionPolicy.options.memoryEfficient.specs')}</p>
          <p class="option-description">{$t('settings.precisionPolicy.options.memoryEfficient.description')}</p>
        </div>

        <div
          class="option-card {isPolicySelected('MaximumPrecision') ? 'selected' : ''}"
          role="button"
          tabindex="0"
          onclick={() => selectPolicy('MaximumPrecision')}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              selectPolicy('MaximumPrecision');
            }
          }}
        >
          <h3>{$t('settings.precisionPolicy.options.maximumPrecision.title')}</h3>
          <p>{$t('settings.precisionPolicy.options.maximumPrecision.specs')}</p>
          <p class="option-description">{$t('settings.precisionPolicy.options.maximumPrecision.description')}</p>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-message">
        {error}
      </div>
    {/if}
  </div>

  <div class="settings-section">
    <h2>{$t('settings.threadLimit.title')}</h2>
    <p class="settings-description">
      {$t('settings.threadLimit.description')}
    </p>

    {#if threadLimitLoading}
      <div class="loading">{$t('settings.threadLimit.loading')}</div>
    {:else}
      <div class="thread-control">
        <label class="slider-label">
          <span>{$t('settings.threadLimit.maxThreads', { count: threadSliderValue })}</span>
          <input
            class="thread-slider"
            type="range"
            min="1"
            max={hardwareConcurrency}
            bind:value={threadSliderValue}
            onchange={(event) => applyCustomThreadLimit(Number((event.currentTarget as HTMLInputElement).value))}
          />
        </label>
        <div class="thread-actions">
          <button
            class="thread-reset"
            onclick={() => applyCustomThreadLimit(null)}
            disabled={threadLimit === null}
          >
            {$t('settings.threadLimit.useSystem', { count: hardwareConcurrency })}
          </button>
          <p class="thread-status">
            {$t('settings.threadLimit.currentMode', { 
                mode: $t(`settings.threadLimit.modes.${threadLimit === null ? 'automatic' : 'manual'}`),
                count: threadLimit ?? hardwareConcurrency 
            })}
          </p>
        </div>
      </div>
    {/if}

    {#if threadLimitError}
      <div class="error-message">
        {threadLimitError}
      </div>
    {/if}
  </div>

  <div class="settings-section experimental-section" class:enabled={experimentalFeatures.enabled}>
    <h2>{$t('settings.experimental.title')}</h2>
    <p class="settings-description">
      {$t('settings.experimental.description')}
    </p>

    <div class="experimental-features-toggle">
      <label class="toggle-label">
        <input
          type="checkbox"
          bind:checked={experimentalFeaturesEnabled}
          onchange={(event) =>
            handleExperimentalFeaturesToggle(
              (event.currentTarget as HTMLInputElement | null)?.checked ?? false,
            )}
        />
        <span class="toggle-slider"></span>
        <span class="toggle-text">{$t('settings.experimental.enable')}</span>
      </label>
      <p class="toggle-description">
        <span class="status-text" class:enabled={experimentalFeatures.enabled}>
          <span class="status-enabled">{$t('settings.experimental.enabled')}</span>
          <span class="status-disabled">{$t('settings.experimental.disabled')}</span>
        </span>
      </p>
    </div>
  </div>

  <div class="settings-section">
    <h2>{$t('settings.modelSelector.title')}</h2>
    <p class="settings-description">
      {$t('settings.modelSelector.description')}
    </p>
    <label class="toggle-label">
      <input
        type="checkbox"
        bind:checked={modelSearchEnabled}
        onchange={(event) =>
          handleModelSearchToggle((event.currentTarget as HTMLInputElement)?.checked ?? true)}
      />
      <span class="toggle-slider"></span>
      <span class="toggle-text">{$t('settings.modelSelector.enableSearch')}</span>
    </label>
    <p class="toggle-description">
      {modelSearchEnabled
        ? $t('settings.modelSelector.enabledDescription')
        : $t('settings.modelSelector.disabledDescription')}
    </p>
  </div>

  <div class="settings-section">
    <h2>{$t('settings.performance.title')}</h2>
    <p class="settings-description">
      {$t('settings.performance.description')}
    </p>

    <PerformanceMonitor />
  </div>

  <div class="settings-section">
    <h2>{$t('settings.language.title')}</h2>
    <p class="settings-description">
      {$t('settings.language.description')}
    </p>
    <LanguageSwitcher />
  </div>
</div>

<style>
  .settings-page {
    padding: 24px;
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
    box-sizing: border-box;
    height: 100%;
    overflow: auto;
  }

  .settings-header {
    margin-bottom: 32px;
  }

  .settings-header h1 {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 8px 0;
  }

  .settings-section {
    background: var(--card);
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 24px;
    border: 1px solid var(--border-color);
    box-sizing: border-box;
    width: 100%;
  }

  .settings-section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 12px 0;
  }

  .settings-description {
    color: var(--muted);
    margin: 0 0 24px 0;
    line-height: 1.5;
  }

  .warning-text {
    color: var(--warning, #eab308);
    font-size: 0.98em;
  }

  .loading {
    text-align: center;
    padding: 24px;
    color: var(--muted);
  }

  .precision-options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    width: 100%;
    box-sizing: border-box;
  }

  .option-card {
    border: 2px solid var(--border-color);
    border-radius: 10px;
    padding: 20px;
    cursor: default;
    transition: all 0.2s ease;
    background: var(--panel-bg);
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
  }

  .option-card:hover {
    border-color: var(--accent);
    transform: none;
  }

  .option-card.selected {
    border-color: var(--accent);
    background: rgba(59, 130, 246, 0.05);
  }

  .option-card h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 8px 0;
    word-wrap: break-word;
  }

  .option-card p {
    color: var(--muted);
    margin: 4px 0;
    font-size: 0.9rem;
    word-wrap: break-word;
  }

  .option-description {
    color: var(--text);
    font-size: 0.95rem;
    margin-top: 12px !important;
    word-wrap: break-word;
  }

  .thread-control {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .thread-slider {
    width: 100%;
  }

  .thread-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .thread-reset {
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--panel-bg);
    padding: 8px 14px;
    font-weight: 600;
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .thread-reset:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .thread-reset:not(:disabled):hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .thread-status {
    margin: 0;
    color: var(--muted);
    font-size: 0.95rem;
  }

  .error-message {
    margin-top: 16px;
    padding: 12px;
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 12px;
    color: #c33;
    word-wrap: break-word;
  }

  .experimental-features-toggle {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: default;
    position: relative;
  }

  .toggle-label input[type='checkbox'] {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: relative;
    display: inline-block;
    width: 52px;
    height: 28px;
    background: var(--border-color);
    border-radius: 14px;
    transition: background-color 0.3s ease;
    border: 2px solid transparent;
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: #ffffff;
    border-radius: 50%;
    transition: transform 0.3s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .toggle-label input:checked + .toggle-slider {
    background: var(--accent);
    border-color: var(--accent);
  }

  .toggle-label input:checked + .toggle-slider::before {
    transform: translateX(24px);
  }

  .toggle-label input:disabled + .toggle-slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-text {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text);
    user-select: none;
  }

  .toggle-description {
    margin: 0;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .status-enabled {
    color: var(--success, #22c55e);
    font-weight: 500;
  }

  .status-disabled {
    color: var(--muted);
    font-weight: 500;
  }

  /* Responsive styles */
  @media (max-width: 768px) {
    .settings-page {
      padding: 16px;
    }

    .settings-section {
      padding: 16px;
    }

    .toggle-label {
      gap: 8px;
    }

    .toggle-text {
      font-size: 0.9rem;
    }

    .settings-header h1 {
      font-size: 1.5rem;
    }

    .settings-section h2 {
      font-size: 1.25rem;
    }

    .precision-options {
      grid-template-columns: 1fr;
      gap: 12px;
    }

    .option-card {
      padding: 16px;
    }

    .option-card h3 {
      font-size: 1.1rem;
    }
  }

  @media (max-width: 480px) {
    .settings-page {
      padding: 12px;
    }

    .settings-section {
      padding: 16px;
    }

    .settings-header h1 {
      font-size: 1.3rem;
    }

    .settings-section h2 {
      font-size: 1.1rem;
    }

    .precision-options {
      grid-template-columns: 1fr;
      gap: 10px;
    }

    .option-card {
      padding: 12px;
    }

    .option-card h3 {
      font-size: 1rem;
    }

    .option-card p {
      font-size: 0.85rem;
    }
  }

  @media (min-width: 1200px) {
    .settings-page {
      max-width: 1000px;
    }

    .precision-options {
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    }
  }

  /* Анимация для секции экспериментальных функций */
  .experimental-section {
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    transform-origin: top;
  }

  .experimental-section:not(.enabled) {
    opacity: 0.7;
    transform: scale(0.98);
  }

  .experimental-section.enabled {
    opacity: 1;
    transform: scale(1);
  }

  .experimental-section h2 {
    transition: color 0.3s ease;
  }

  .experimental-section.enabled h2 {
    color: #3b82f6;
  }

  .experimental-section .settings-description {
    transition: opacity 0.3s ease;
  }

  .experimental-section:not(.enabled) .settings-description {
    opacity: 0.8;
  }

  .experimental-features-toggle {
    transition: all 0.3s ease;
  }

  .toggle-description {
    transition: all 0.3s ease;
  }

  .status-text {
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    color: #6b7280;
    position: relative;
    display: inline-block;
    overflow: hidden;
    height: 1.5em;
  }

  .status-text.enabled {
    color: #10b981;
    font-weight: 500;
  }

  .status-enabled,
  .status-disabled {
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1) !important;
    position: absolute !important;
    top: 0 !important;
    left: 0 !important;
    width: 100% !important;
    opacity: 0 !important;
    transform: translateY(20px) !important;
  }

  .status-text:not(.enabled) .status-disabled {
    opacity: 1 !important;
    transform: translateY(0) !important;
  }

  .status-text.enabled .status-enabled {
    opacity: 1 !important;
    transform: translateY(0) !important;
  }
</style>
