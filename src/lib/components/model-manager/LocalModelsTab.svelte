<script lang="ts">
  /**
   * Local models tab component
   * Combines all elements for local models management
   */

  import LocalModelActions from './LocalModelActions.svelte';
  import LocalModelsList from './LocalModelsList.svelte';
  import ModelStatus from './ModelStatus.svelte';
  import MemoryMonitor from './MemoryMonitor.svelte';
  import {
    folderPath,
    models,
    selectedModel,
    isLoading,
    error,
    scanFolder,
    deleteModel,
  } from '$lib/stores/local-models';

  // Handle folder scan
  async function handleScanFolder(path: string) {
    await scanFolder(path, false);
  }

  // Handle model deletion
  async function handleDeleteModel(model: any) {
    try {
      await deleteModel(model.path);
    } catch (err) {
      console.error('Failed to delete model:', err);
    }
  }

  // Handle model loading
  async function handleLoadModel(model: any) {
    console.log('Loading model:', model);
    // Model loading is handled in LocalModelActions
  }
</script>

<div class="local-models-tab">
  <!-- Left panel - Controls and status -->
  <aside class="control-panel">
    <ModelStatus />
    <MemoryMonitor />
    <LocalModelActions
      bind:selectedModel={$selectedModel}
      folderPath={$folderPath}
      isLoading={$isLoading}
      onScanFolder={handleScanFolder}
      onLoadModel={handleLoadModel}
    />
  </aside>

  <!-- Right panel - Models list -->
  <main class="models-panel">
    <div class="panel-header">
      <h2>Локальные модели</h2>
      {#if $models.length > 0}
        <span class="models-count">{$models.length} моделей</span>
      {/if}
    </div>

    <div class="panel-content">
      {#if $error}
        <div class="error-state" role="alert">
          <p class="error-message">{$error}</p>
        </div>
      {/if}

      {#if $isLoading}
        <div class="loading-state">
          <div class="loading-spinner"></div>
          <p>Сканирование папки...</p>
        </div>
      {:else}
        <LocalModelsList
          models={$models}
          bind:selectedModel={$selectedModel}
          onDelete={handleDeleteModel}
        />
      {/if}
    </div>
  </main>
</div>

<style>
  .local-models-tab {
    display: grid;
    grid-template-columns: 380px 1fr;
    gap: 2rem;
    height: 100%;
    padding: 2rem;
    overflow: hidden;
  }

  .control-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow-y: auto;
  }

  .models-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text);
  }

  .models-count {
    padding: 0.375rem 0.75rem;
    background: rgba(52, 152, 219, 0.1);
    color: var(--accent, #3498db);
    font-size: 0.875rem;
    font-weight: 600;
    border-radius: 12px;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .error-state {
    padding: 1.5rem;
    background: rgba(231, 76, 60, 0.05);
    border: 1px solid rgba(231, 76, 60, 0.2);
    border-radius: 8px;
    margin: 1.5rem;
  }

  .error-message {
    margin: 0;
    color: var(--error, #e74c3c);
    font-size: 0.9375rem;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 1rem;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(52, 152, 219, 0.1);
    border-top-color: var(--accent, #3498db);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-state p {
    margin: 0;
    color: var(--muted);
    font-size: 0.9375rem;
  }

  /* Скроллбар для control panel */
  .control-panel::-webkit-scrollbar {
    width: 8px;
  }

  .control-panel::-webkit-scrollbar-track {
    background: transparent;
  }

  .control-panel::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 4px;
  }

  .control-panel::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
  }

  /* Адаптивность */
  @media (max-width: 1024px) {
    .local-models-tab {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }

    .control-panel {
      max-height: 400px;
    }
  }

  @media (max-width: 768px) {
    .local-models-tab {
      padding: 1rem;
      gap: 1rem;
    }

    .panel-header h2 {
      font-size: 1.25rem;
    }
  }
</style>
