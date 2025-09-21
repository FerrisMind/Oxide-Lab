<script lang="ts">
  import { headerSearchQuery as _headerSearchQuery } from '$lib/stores/search';
  import ModelSearchFilters from '$lib/components/ModelSearchFilters.svelte';
  import SearchManager from '$lib/components/search/SearchManager.svelte';
  import SearchResults from '$lib/components/search/SearchResults.svelte';
  import WelcomeCard from '$lib/components/search/WelcomeCard.svelte';
</script>

<SearchManager let:searchQuery
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
               let:handleModelSelect>
  <main class="wrap">
    <div class="search-page">
      <!-- Фильтры поиска на всю ширину -->
      <div class="filters-container">
        <ModelSearchFilters
          searchQuery={searchQuery}
          selectedFormats={selectedFormats}
          selectedPipelineTags={selectedPipelineTags}
          selectedLibraries={selectedLibraries}
          selectedLanguages={selectedLanguages}
          selectedLicenses={selectedLicenses}
          authorFilter={authorFilter}
          {isLoading}
          clearSearch={() => {
            // Clear all filters by emitting events
          }}
          on:search={(event) => {
            const { query, formats, pipelineTags, libraries, languages, licenses, author } = event.detail;
            searchModels({
              query,
              formats,
              pipelineTags,
              libraries,
              languages,
              licenses,
              author
            });
          }}
        />
      </div>

      <!-- Секция результатов -->
      <div class="results-section">
        <SearchResults {models}
                       {selectedModel}
                       {isLoading}
                       {hasMore}
                       {totalCount}
                       {error}
                       {hasSearched}
                       handleModelSelect={handleModelSelect}
                       loadMoreModels={loadMoreModels}>
          <WelcomeCard 
            searchQuery={searchQuery}
            selectedFormats={selectedFormats}
            selectedPipelineTags={selectedPipelineTags}
            selectedLibraries={selectedLibraries}
            selectedLanguages={selectedLanguages}
            selectedLicenses={selectedLicenses}
            authorFilter={authorFilter}
            {searchModels}
          />
        </SearchResults>
      </div>
    </div>
  </main>
</SearchManager>

<style>
  .search-page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 56px - 32px);
    max-height: calc(100vh - 56px - 32px);
    background: var(--bg);
    width: 100%;
    overflow: hidden;
  }

  .filters-container {
    flex-shrink: 0;
    width: 100%;
    max-width: none;
    background: var(--panel-bg);
    border-bottom: 1px solid var(--border);
    padding: 0;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: sticky;
    top: 0;
    z-index: 10;
    border-radius: 12px 12px 12px 12px;
  }

  .results-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 20px;
    width: 100%;
    min-width: 0;
  }

  /* Адаптивность */
  @media (max-width: 1024px) {
    .filters-container {
      padding: 16px;
    }

    .results-section {
      padding: 16px;
    }
  }

  @media (max-width: 768px) {
    .search-page {
      height: auto;
      min-height: calc(100vh - 60px);
    }

    .filters-container {
      padding: 12px;
    }

    .results-section {
      padding: 12px;
    }
  }

  @media (max-width: 480px) {
    .results-section {
      padding: 8px;
    }
  }
</style>