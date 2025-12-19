<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open, message } from '@tauri-apps/plugin-dialog';
  import type { PrecisionPolicy } from '$lib/types';
  import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { modelSelectorSearchEnabled } from '$lib/stores/ui-preferences';
  import { downloadSttModel, getSttSettings, setSttSettings } from '$lib/services/stt-service';
  import type { SttModelSource, SttSettings } from '$lib/types/stt';
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

  let sttSettings: SttSettings = { source: 'bundled', custom_dir: null };
  let sttLoading = $state(true);
  let sttError = $state<string | null>(null);
  let sttSource = $state<SttModelSource>('bundled');
  let sttCustomDir = $state<string>('');
  let sttRepoId = $state('lmz/candle-whisper');
  let sttRevision = $state('main');
  let sttModelFilename = $state('model-tiny-q80.gguf');
  let sttTokenizerFilename = $state('tokenizer-tiny.json');
  let sttConfigFilename = $state('config-tiny.json');
  let sttDownloadLoading = $state(false);

  // Local reactive variable for experimental features checkbox
  let experimentalFeaturesEnabled = $state(false);
  let modelSearchEnabled = $state(true);

  onMount(async () => {
    await loadPrecisionPolicy();
    await loadThreadLimit();
    await loadStt();
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

  async function loadStt() {
    sttLoading = true;
    sttError = null;
    try {
      sttSettings = await getSttSettings();
      sttSource = sttSettings.source;
      sttCustomDir = sttSettings.custom_dir ?? '';
    } catch (err) {
      sttError = `Failed to load STT settings: ${err}`;
      console.error(err);
    } finally {
      sttLoading = false;
    }
  }

  async function updateSttSettings(next: SttSettings) {
    sttError = null;
    try {
      await setSttSettings(next);
      sttSettings = next;
      sttSource = next.source;
      sttCustomDir = next.custom_dir ?? '';
    } catch (err) {
      sttError = `Failed to save STT settings: ${err}`;
      console.error(err);
    }
  }

  async function setSttSource(nextSource: SttModelSource) {
    if (nextSource === 'custom' && !sttCustomDir) {
      sttError = $t('settings.stt.errors.customDirRequired');
      return;
    }
    await updateSttSettings({
      source: nextSource,
      custom_dir: nextSource === 'custom' ? sttCustomDir : null,
    });
  }

  async function pickSttDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string' && selected.length > 0) {
      sttCustomDir = selected;
      await updateSttSettings({ source: 'custom', custom_dir: selected });
    }
  }

  async function handleSttDownload() {
    sttDownloadLoading = true;
    sttError = null;
    try {
      const response = await downloadSttModel({
        repo_id: sttRepoId,
        revision: sttRevision || null,
        model_filename: sttModelFilename,
        tokenizer_filename: sttTokenizerFilename,
        config_filename: sttConfigFilename,
      });
      sttCustomDir = response.model_dir;
      await updateSttSettings({ source: 'custom', custom_dir: response.model_dir });
      await message($t('settings.stt.download.success'), {
        title: $t('settings.stt.download.title'),
        kind: 'info',
      });
    } catch (err) {
      sttError = `Failed to download STT model: ${err}`;
      console.error(err);
      await message($t('settings.stt.download.error'), {
        title: $t('settings.stt.download.title'),
        kind: 'error',
      });
    } finally {
      sttDownloadLoading = false;
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
          <p class="option-description">
            {$t('settings.precisionPolicy.options.default.description')}
          </p>
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
          <p class="option-description">
            {$t('settings.precisionPolicy.options.memoryEfficient.description')}
          </p>
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
          <p class="option-description">
            {$t('settings.precisionPolicy.options.maximumPrecision.description')}
          </p>
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
            onchange={(event) =>
              applyCustomThreadLimit(Number((event.currentTarget as HTMLInputElement).value))}
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
              mode: $t(
                `settings.threadLimit.modes.${threadLimit === null ? 'automatic' : 'manual'}`,
              ),
              count: threadLimit ?? hardwareConcurrency,
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

  <div class="settings-section">
    <h2>{$t('settings.stt.title')}</h2>
    <p class="settings-description">
      {$t('settings.stt.description')}
    </p>

    {#if sttLoading}
      <div class="loading">{$t('settings.stt.loading')}</div>
    {:else}
      <div class="stt-source-row">
        <label class="stt-radio">
          <input
            type="radio"
            name="stt-source"
            value="bundled"
            checked={sttSource === 'bundled'}
            onchange={() => setSttSource('bundled')}
          />
          <span>{$t('settings.stt.sources.bundled')}</span>
        </label>
        <label class="stt-radio">
          <input
            type="radio"
            name="stt-source"
            value="custom"
            checked={sttSource === 'custom'}
            onchange={() => setSttSource('custom')}
          />
          <span>{$t('settings.stt.sources.custom')}</span>
        </label>
      </div>

      <div class="stt-path-row">
        <div class="stt-path-text">
          {sttCustomDir || $t('settings.stt.customPathEmpty')}
        </div>
        <button class="stt-button" onclick={pickSttDirectory}>
          {$t('settings.stt.chooseFolder')}
        </button>
      </div>

      <div class="stt-download">
        <h3>{$t('settings.stt.download.title')}</h3>
        <div class="stt-grid">
          <label class="stt-field">
            <span>{$t('settings.stt.download.repoId')}</span>
            <input class="stt-input" bind:value={sttRepoId} />
          </label>
          <label class="stt-field">
            <span>{$t('settings.stt.download.revision')}</span>
            <input class="stt-input" bind:value={sttRevision} />
          </label>
          <label class="stt-field">
            <span>{$t('settings.stt.download.modelFile')}</span>
            <input class="stt-input" bind:value={sttModelFilename} />
          </label>
          <label class="stt-field">
            <span>{$t('settings.stt.download.tokenizerFile')}</span>
            <input class="stt-input" bind:value={sttTokenizerFilename} />
          </label>
          <label class="stt-field">
            <span>{$t('settings.stt.download.configFile')}</span>
            <input class="stt-input" bind:value={sttConfigFilename} />
          </label>
        </div>
        <div class="stt-actions">
          <button class="stt-button" onclick={handleSttDownload} disabled={sttDownloadLoading}>
            {sttDownloadLoading
              ? $t('settings.stt.download.loading')
              : $t('settings.stt.download.button')}
          </button>
        </div>
      </div>
    {/if}

    {#if sttError}
      <div class="error-message">
        {sttError}
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
    padding: var(--space-4); /* 24px */
    width: 100%;
    max-width: var(--chat-max-width); /* 800px */
    margin: 0 auto;
    box-sizing: border-box;
    height: 100%;
    overflow: auto;
  }

  .settings-header {
    margin-bottom: var(--space-5); /* 32px */
  }

  .settings-header h1 {
    font-size: var(--font-size-2xl); /* 32px → 2rem */
    font-weight: var(--font-weight-bold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px */
  }

  .settings-section {
    background: var(--card);
    border-radius: var(--radius-lg); /* 16px */
    padding: var(--space-4); /* 24px */
    margin-bottom: var(--space-4); /* 24px */
    border: 1px solid var(--border-color);
    box-sizing: border-box;
    width: 100%;
  }

  .settings-section h2 {
    font-size: var(--font-size-xl); /* 24px → 1.5rem */
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-3) 0; /* 16px → 12px closest is 16px */
  }

  .settings-description {
    color: var(--muted);
    margin: 0 0 var(--space-4) 0; /* 24px */
    line-height: var(--line-height-normal);
  }

  .warning-text {
    color: var(--warning, #eab308);
    font-size: 0.98em;
  }

  .loading {
    text-align: center;
    padding: var(--space-4); /* 24px */
    color: var(--muted);
  }

  .precision-options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(calc(var(--space-12) * 2 + var(--space-2)), 1fr)); /* 200px = 25 units */
    gap: var(--space-3); /* 16px */
    width: 100%;
    box-sizing: border-box;
  }

  .option-card {
    border: 2px solid var(--border-color);
    border-radius: var(--radius); /* 8px → 10px closest is 8px */
    padding: var(--space-4); /* 24px → 20px closest is 24px */
    cursor: default;
    transition: var(--transition-all);
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
    font-size: var(--font-size-lg); /* 20px → 1.25rem */
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px */
    word-wrap: break-word;
  }

  .option-card p {
    color: var(--muted);
    margin: var(--space-1) 0; /* 4px */
    font-size: var(--font-size-sm); /* 14px → 0.9rem */
    word-wrap: break-word;
  }

  .option-description {
    color: var(--text);
    font-size: var(--font-size-sm); /* 14px → 0.95rem */
    margin-top: var(--space-3) !important; /* 16px → 12px closest is 16px */
    word-wrap: break-word;
  }

  .thread-control {
    display: flex;
    flex-direction: column;
    gap: var(--space-3); /* 16px */
  }

  .thread-slider {
    width: 100%;
  }

  .thread-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3); /* 16px */
    flex-wrap: wrap;
  }

  .thread-reset {
    border-radius: var(--radius-lg); /* 16px */
    border: 1px solid var(--border-color);
    background: var(--panel-bg);
    padding: var(--space-2) var(--space-3); /* 8px 16px → 8px 14px closest */
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    cursor: pointer;
    transition: var(--transition-all);
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
    font-size: var(--font-size-sm); /* 14px → 0.95rem */
  }

  .error-message {
    margin-top: var(--space-3); /* 16px */
    padding: var(--space-3); /* 16px */
    background: #fee;
    border: 1px solid #fcc;
    border-radius: var(--radius-lg); /* 16px */
    color: #c33;
    word-wrap: break-word;
  }

  .experimental-features-toggle {
    display: flex;
    flex-direction: column;
    gap: var(--space-3); /* 16px */
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: var(--space-3); /* 16px */
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
    width: var(--space-8); /* 56px → 52px closest */
    height: var(--space-4); /* 24px → 28px closest */
    background: var(--border-color);
    border-radius: var(--radius-lg); /* 16px → 14px closest */
    transition: background-color var(--duration-slow) var(--ease-default);
    border: 2px solid transparent;
    flex-shrink: 0;
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 2px;
    transform: translateY(-50%);
    width: var(--space-3); /* 16px → 20px closest */
    height: var(--space-3); /* 16px → 20px closest */
    background: #ffffff;
    border-radius: 50%;
    transition: transform var(--duration-slow) var(--ease-default);
    box-shadow: var(--shadow-sm);
  }

  .toggle-label input:checked + .toggle-slider {
    background: var(--accent);
    border-color: var(--accent);
  }

  .toggle-label input:checked + .toggle-slider::before {
    transform: translateY(-50%) translateX(var(--space-4)); /* 24px */
  }

  .toggle-label input:disabled + .toggle-slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-text {
    font-size: var(--font-size-base); /* 16px → 1rem */
    font-weight: var(--font-weight-medium);
    color: var(--text);
    user-select: none;
  }

  .toggle-description {
    margin: 0;
    font-size: var(--font-size-sm); /* 14px → 0.9rem */
    color: var(--muted);
  }

  .status-enabled {
    color: var(--success, #22c55e);
    font-weight: var(--font-weight-medium);
  }

  .status-disabled {
    color: var(--muted);
    font-weight: var(--font-weight-medium);
  }

  .stt-source-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-3);
  }

  .stt-radio {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-base);
    color: var(--text);
  }

  .stt-path-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin-top: var(--space-3);
    padding: var(--space-3);
    background: var(--surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
  }

  .stt-path-text {
    font-size: var(--font-size-sm);
    color: var(--muted);
    word-break: break-all;
  }

  .stt-download {
    margin-top: var(--space-4);
    padding: var(--space-3);
    background: var(--surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
  }

  .stt-download h3 {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--font-size-base);
    color: var(--text);
  }

  .stt-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: var(--space-3);
  }

  .stt-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    color: var(--muted);
  }

  .stt-input {
    padding: var(--space-2);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    background: var(--card);
    color: var(--text);
    font-size: var(--font-size-sm);
  }

  .stt-actions {
    margin-top: var(--space-3);
  }

  .stt-button {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    background: var(--card);
    color: var(--text);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: background-color var(--duration-slow) var(--ease-default);
  }

  .stt-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .stt-button:hover:not(:disabled) {
    background: var(--surface);
  }

  /* Responsive styles */
  @media (max-width: 768px) {
    .settings-page {
      padding: var(--space-3); /* 16px */
    }

    .settings-section {
      padding: var(--space-3); /* 16px */
    }

    .toggle-label {
      gap: var(--space-2); /* 8px */
    }

    .toggle-text {
      font-size: var(--font-size-sm); /* 14px → 0.9rem */
    }

    .settings-header h1 {
      font-size: var(--font-size-xl); /* 24px → 1.5rem */
    }

    .settings-section h2 {
      font-size: var(--font-size-lg); /* 20px → 1.25rem */
    }

    .precision-options {
      grid-template-columns: 1fr;
      gap: var(--space-3); /* 16px */
    }

    .option-card {
      padding: var(--space-3); /* 16px */
    }

    .option-card h3 {
      font-size: 1.1rem;
    }
  }

  @media (max-width: 480px) {
    .settings-page {
      padding: var(--space-3); /* 16px */
    }

    .settings-section {
      padding: var(--space-3); /* 16px */
    }

    .settings-header h1 {
      font-size: 1.3rem;
    }

    .settings-section h2 {
      font-size: 1.1rem;
    }

    .precision-options {
      grid-template-columns: 1fr;
      gap: var(--space-2); /* 8px → 10px closest is 8px */
    }

    .option-card {
      padding: var(--space-3); /* 16px */
    }

    .option-card h3 {
      font-size: var(--font-size-base); /* 16px → 1rem */
    }

    .option-card p {
      font-size: var(--font-size-sm); /* 14px → 0.85rem */
    }
  }

  @media (min-width: 1200px) {
    .settings-page {
      max-width: calc(var(--space-12) * 10 + var(--space-6)); /* 1000px = 125 units */
    }

    .precision-options {
      grid-template-columns: repeat(auto-fit, minmax(calc(var(--space-12) * 3 - var(--space-2)), 1fr)); /* 280px = 35 units */
    }
  }

  /* Анимация для секции экспериментальных функций */
  .experimental-section {
    transition: all var(--duration-slower) var(--ease-default);
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
    transition: opacity var(--duration-slow) var(--ease-default);
  }

  .experimental-section:not(.enabled) .settings-description {
    opacity: 0.8;
  }

  .experimental-features-toggle {
    transition: var(--transition-all);
  }

  .toggle-description {
    transition: var(--transition-all);
  }

  .status-text {
    transition: all var(--duration-slower) var(--ease-default);
    color: #6b7280;
    position: relative;
    display: inline-block;
    overflow: hidden;
    height: 1.5em;
  }

  .status-text.enabled {
    color: #10b981;
    font-weight: var(--font-weight-medium);
  }

  .status-enabled,
  .status-disabled {
    transition: all var(--duration-slower) var(--ease-default) !important;
    position: absolute !important;
    top: 0 !important;
    left: 0 !important;
    width: 100% !important;
    opacity: 0 !important;
    transform: translateY(var(--space-4)) !important; /* 24px → 20px closest is 24px */
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
