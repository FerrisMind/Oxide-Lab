<script lang="ts">
  import { onMount } from 'svelte';
  import SearchManager from '$lib/components/search/SearchManager.svelte';
  import SearchLayout from '$lib/components/SearchLayout.svelte';
  import ModelStatus from '$lib/components/model-manager/ModelStatus.svelte';
  import ModelActions from '$lib/components/model-manager/ModelActions.svelte';
  import MemoryMonitor from '$lib/components/model-manager/MemoryMonitor.svelte';
  import type { HFModel } from '$lib/services/huggingface';
  
  // Состояние модели - получаем из ModelStatus
  let modelStatusComponent: any;
  let modelStatus = $state({
    isLoaded: false,
    isLoading: false,
    loadingProgress: 0,
    error: null as string | null
  });
  
  // Выбранная модель из поиска
  let selectedModel = $state<HFModel | null>(null);
  
  // Обработчики событий
  function handleModelLoad() {
    console.log('Model load initiated');
  }
  
  function handleModelUnload() {
    console.log('Model unload initiated');
    selectedModel = null;
  }
  
  // Обновляем статус модели
  function updateModelStatus() {
    if (modelStatusComponent) {
      const status = modelStatusComponent.getStatus();
      modelStatus = {
        isLoaded: status.isLoaded,
        isLoading: status.isLoading,
        loadingProgress: status.loadingProgress,
        error: status.error
      };
    }
  }
  
  // Периодически обновляем статус
  onMount(() => {
    const interval = setInterval(updateModelStatus, 100);
    return () => clearInterval(interval);
  });
</script>

<div class="models-page">
  <div class="page-header">
    <h1>Управление моделями</h1>
    <p class="page-description">
      Выберите модель из HuggingFace или загрузите локальную модель для работы
    </p>
  </div>
  
  <div class="models-layout">
    <!-- Левая панель - статус и действия -->
    <aside class="control-panel">
      <ModelStatus bind:this={modelStatusComponent} />
      <MemoryMonitor />
      <ModelActions
        bind:selectedModel
        isLoaded={modelStatus.isLoaded}
        isLoading={modelStatus.isLoading}
        onModelLoad={handleModelLoad}
        onModelUnload={handleModelUnload}
      />
    </aside>
    
    <!-- Правая панель - поиск и выбор моделей -->
    <main class="search-panel">
      <SearchManager let:models let:isLoading let:searchModels let:hasMore let:totalCount>
        <div class="search-section">
          <div class="search-header">
            <h2>Поиск моделей HuggingFace</h2>
            <button
              class="btn btn-search"
              onclick={() => searchModels()}
              disabled={isLoading}
            >
              {isLoading ? 'Поиск...' : 'Обновить'}
            </button>
          </div>
          
          <SearchLayout
            {models}
            {selectedModel}
            {isLoading}
            {hasMore}
            {totalCount}
          />
        </div>
      </SearchManager>
    </main>
  </div>
</div>

<style>
  .models-page {
    min-height: 100vh;
    background: var(--bg);
    padding: 2rem;
  }
  
  .page-header {
    max-width: 1400px;
    margin: 0 auto 2rem;
  }
  
  .page-header h1 {
    margin: 0 0 0.5rem 0;
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
  }
  
  .page-description {
    margin: 0;
    font-size: 1rem;
    color: var(--muted);
  }
  
  .models-layout {
    display: grid;
    grid-template-columns: 380px 1fr;
    gap: 2rem;
    max-width: 1400px;
    margin: 0 auto;
  }
  
  .control-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    position: sticky;
    top: 2rem;
    align-self: start;
    max-height: calc(100vh - 4rem);
    overflow-y: auto;
  }
  
  .search-panel {
    min-height: 600px;
  }
  
  .search-section {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
  }
  
  .search-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }
  
  .search-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }
  
  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .btn-search {
    background: var(--accent, #3498db);
    color: white;
  }
  
  .btn-search:hover:not(:disabled) {
    background: var(--accent-hover, #2980b9);
  }
  
  /* Адаптивность для мобильных */
  @media (max-width: 1024px) {
    .models-layout {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }
    
    .control-panel {
      position: static;
      max-height: none;
    }
  }
  
  @media (max-width: 768px) {
    .models-page {
      padding: 1rem;
    }
    
    .page-header h1 {
      font-size: 1.5rem;
    }
    
    .search-section {
      padding: 1rem;
    }
  }
  
  /* Скроллбар для control panel */
  .control-panel::-webkit-scrollbar {
    width: 8px;
  }
  
  .control-panel::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 4px;
  }
  
  .control-panel::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.6);
    border-radius: 4px;
    transition: background 0.3s ease;
  }
  
  .control-panel::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 0.8);
  }
</style>