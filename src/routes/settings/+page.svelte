<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { PrecisionPolicy } from '$lib/types';
  import PerformanceMonitor from '$lib/components/PerformanceMonitor.svelte';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';

  let currentPolicy: PrecisionPolicy = { Default: null };
  let isLoading = $state(true);
  let error: string | null = $state(null);

  // Local reactive variable for experimental features checkbox
  let experimentalFeaturesEnabled = $state(false);

  onMount(async () => {
    await loadPrecisionPolicy();
  });

  // Sync local variable with store when experimental features are initialized
  $effect(() => {
    if (experimentalFeatures.initialized) {
      experimentalFeaturesEnabled = experimentalFeatures.enabled;
    }
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
    <h1>Настройки приложения</h1>
  </div>

  <div class="settings-section">
    <h2>Политика точности (Precision Policy)</h2>
    <p class="settings-description">
      Выберите политику точности для загрузки и выполнения моделей. Это влияет на использование
      памяти и производительность.<br />
      <span class="warning-text">
        <b>Внимание:</b> параметр precision влияет только на <b>не квантизованные</b> модели (float32/float16).
        Для квантизованных моделей (4-bit/8-bit) точность весов фиксирована, настройка влияет только
        на промежуточные вычисления.
      </span>
    </p>

    {#if isLoading}
      <div class="loading">Загрузка настроек...</div>
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
          <h3>Стандартная</h3>
          <p>CPU: F32, GPU: BF16</p>
          <p class="option-description">Оптимальный баланс между производительностью и точностью</p>
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
          <h3>Экономия памяти</h3>
          <p>CPU: F32, GPU: F16</p>
          <p class="option-description">Меньше использование памяти, немного ниже точность</p>
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
          <h3>Максимальная точность</h3>
          <p>CPU: F32, GPU: F32</p>
          <p class="option-description">Наивысшая точность, больше использование памяти</p>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-message">
        {error}
      </div>
    {/if}
  </div>

  <div class="settings-section experimental-section" class:enabled={experimentalFeatures.enabled}>
    <h2>Экспериментальные функции</h2>
    <p class="settings-description">
      Включите экспериментальные функции для тестирования новых возможностей. Эти функции могут быть
      нестабильными и содержать ошибки.
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
        <span class="toggle-text">Включить экспериментальные функции</span>
      </label>
      <p class="toggle-description">
        <span class="status-text" class:enabled={experimentalFeatures.enabled}>
          <span class="status-enabled">✓ Экспериментальные функции включены</span>
          <span class="status-disabled">✗ Экспериментальные функции отключены</span>
        </span>
      </p>
    </div>
  </div>

  <div class="settings-section">
    <h2>Мониторинг производительности</h2>
    <p class="settings-description">
      Отслеживание производительности приложения, включая время запуска, использование памяти и
      скорость работы моделей.
    </p>

    <PerformanceMonitor />
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
    transform: translateY(-2px);
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

  .error-message {
    margin-top: 16px;
    padding: 12px;
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 6px;
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
