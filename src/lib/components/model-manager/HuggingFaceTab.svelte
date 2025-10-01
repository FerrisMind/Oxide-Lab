<script lang="ts">
  /**
   * Hugging Face models search tab
   * Contains model search and loading functionality from Hugging Face Hub
   */

  import { onMount } from 'svelte';
  import ModelStatus from './ModelStatus.svelte';
  import MemoryMonitor from './MemoryMonitor.svelte';
  import ModelActions from './ModelActions.svelte';
  import ModelSearchFilters from '$lib/components/ModelSearchFilters.svelte';
  import SearchManager from '$lib/components/search/SearchManager.svelte';
  import SearchResults from '$lib/components/search/SearchResults.svelte';
  import WelcomeCard from '$lib/components/search/WelcomeCard.svelte';
  import type { HFModel } from '$lib/services/huggingface';

  // Model status state
  let modelStatusComponent: any;
  let modelStatus = $state({
    isLoaded: false,
    isLoading: false,
    loadingProgress: 0,
    error: null as string | null,
  });

  // Обработчики событий
  function handleModelLoad() {
    console.log('Model load initiated from HF tab');
  }

  function handleModelUnload() {
    console.log('Model unload initiated from HF tab');
  }

  // Обновляем статус модели
  function updateModelStatus() {
    if (modelStatusComponent) {
      const status = modelStatusComponent.getStatus();
      modelStatus = {
        isLoaded: status.isLoaded,
        isLoading: status.isLoading,
        loadingProgress: status.loadingProgress,
        error: status.error,
      };
    }
  }

  // Периодически обновляем статус
  onMount(() => {
    const interval = setInterval(updateModelStatus, 100);
    return () => clearInterval(interval);
  });
</script>

<SearchManager
  let:searchQuery
  let:selectedFormats
  let:selectedPipelineTags
  let:selectedLibraries
  let:selectedLanguages
  let:selectedLicenses
  let:authorFilter
  let:isLoading
  let:models
  let:error
  let:hasSearched
  let:hasMore
  let:totalCount
  let:selectedModel
  let:searchModels
  let:loadMoreModels
  let:handleModelSelect
>
  <div class="huggingface-tab">
    <!-- Left panel - Controls and status -->
    <aside class="control-panel">
      <ModelStatus bind:this={modelStatusComponent} />
      <MemoryMonitor />
      <ModelActions
        {selectedModel}
        isLoaded={modelStatus.isLoaded}
        isLoading={modelStatus.isLoading}
        onModelLoad={handleModelLoad}
        onModelUnload={handleModelUnload}
      />
    </aside>

    <!-- Right panel - Search and results -->
    <main class="search-panel">
      <!-- Filters section -->
      <div class="filters-container">
        <ModelSearchFilters
          {searchQuery}
          {selectedFormats}
          {selectedPipelineTags}
          {selectedLibraries}
          {selectedLanguages}
          {selectedLicenses}
          {authorFilter}
          {isLoading}
          clearSearch={() => {
            // Clear all filters
          }}
          on:search={(event) => {
            const { query, formats, pipelineTags, libraries, languages, licenses, author } =
              event.detail;
            searchModels({
              query,
              formats,
              pipelineTags,
              libraries,
              languages,
              licenses,
              author,
            });
          }}
        />
      </div>

      <!-- Results section -->
      <div class="results-container">
        <SearchResults
          {models}
          {selectedModel}
          {isLoading}
          {hasMore}
          {totalCount}
          {error}
          {hasSearched}
          {handleModelSelect}
          {loadMoreModels}
        >
          <WelcomeCard
            {searchQuery}
            {selectedFormats}
            {selectedPipelineTags}
            {selectedLibraries}
            {selectedLanguages}
            {selectedLicenses}
            {authorFilter}
            {searchModels}
          />
        </SearchResults>
      </div>
    </main>
  </div>
</SearchManager>

<style>
  .huggingface-tab {
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

  .search-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    overflow: hidden;
  }

  .filters-container {
    flex-shrink: 0;
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .results-container {
    flex: 1;
    overflow: hidden;
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
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
    .huggingface-tab {
      grid-template-columns: 1fr;
      gap: 1.5rem;
    }

    .control-panel {
      max-height: 400px;
    }
  }

  @media (max-width: 768px) {
    .huggingface-tab {
      padding: 1rem;
      gap: 1rem;
    }
  }
</style>
