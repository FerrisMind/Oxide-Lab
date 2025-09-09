<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { PrecisionPolicy } from '$lib/types';

  let currentPolicy: PrecisionPolicy = { Default: null };
  let isLoading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadPrecisionPolicy();
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

  function isPolicySelected(policyType: 'Default' | 'MemoryEfficient' | 'MaximumPrecision'): boolean {
    try {
      console.log('Checking policy selection:', policyType, 'currentPolicy:', currentPolicy, 'type:', typeof currentPolicy);
      
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
        if (typeof currentPolicy === 'object' && currentPolicy !== null && 'variant' in currentPolicy) {
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
      Выберите политику точности для загрузки и выполнения моделей. 
      Это влияет на использование памяти и производительность.
    </p>

    {#if isLoading}
      <div class="loading">Загрузка настроек...</div>
    {:else}
      <div class="precision-options">
        <div class="option-card {isPolicySelected('Default') ? 'selected' : ''}" 
             role="button"
             tabindex="0"
             on:click={() => selectPolicy('Default')}
             on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectPolicy('Default'); }}}>
          <h3>Стандартная</h3>
          <p>CPU: F32, GPU: BF16</p>
          <p class="option-description">Оптимальный баланс между производительностью и точностью</p>
        </div>

        <div class="option-card {isPolicySelected('MemoryEfficient') ? 'selected' : ''}" 
             role="button"
             tabindex="0"
             on:click={() => selectPolicy('MemoryEfficient')}
             on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectPolicy('MemoryEfficient'); }}}>
          <h3>Экономия памяти</h3>
          <p>CPU: F32, GPU: F16</p>
          <p class="option-description">Меньше использование памяти, немного ниже точность</p>
        </div>

        <div class="option-card {isPolicySelected('MaximumPrecision') ? 'selected' : ''}" 
             role="button"
             tabindex="0"
             on:click={() => selectPolicy('MaximumPrecision')}
             on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectPolicy('MaximumPrecision'); }}}>
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
</div>

<style>
  .settings-page {
    padding: 24px;
    max-width: 800px;
    margin: 0 auto;
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

  .loading {
    text-align: center;
    padding: 24px;
    color: var(--muted);
  }

  .precision-options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 16px;
  }

  .option-card {
    border: 2px solid var(--border-color);
    border-radius: 10px;
    padding: 20px;
    cursor: default;
    transition: all 0.2s ease;
    background: var(--panel-bg);
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
  }

  .option-card p {
    color: var(--muted);
    margin: 4px 0;
    font-size: 0.9rem;
  }

  .option-description {
    color: var(--text);
    font-size: 0.95rem;
    margin-top: 12px !important;
  }

  .error-message {
    margin-top: 16px;
    padding: 12px;
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 6px;
    color: #c33;
  }

  @media (max-width: 768px) {
    .settings-page {
      padding: 16px;
    }
    
    .settings-section {
      padding: 16px;
    }
    
    .precision-options {
      grid-template-columns: 1fr;
    }
  }
</style>