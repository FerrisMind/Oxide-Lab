<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { 
    availablePipelineTags, 
    availableLibraries, 
    availableLanguages, 
    availableLicenses,
    hasActiveFilters
  } from './model-search-filters/filter-types';

  import { createEventHandlers } from './model-search-filters/event-handlers';

  import FilterHeader from './model-search-filters/FilterHeader.svelte';
  import FormatFilter from './model-search-filters/FormatFilter.svelte';
  import AuthorFilter from './model-search-filters/AuthorFilter.svelte';
  import GenericFilter from './model-search-filters/GenericFilter.svelte';
  import FilterActions from './model-search-filters/FilterActions.svelte';
  import ActiveFilters from './model-search-filters/ActiveFilters.svelte';

  const dispatch = createEventDispatcher();

  export let searchQuery = '';
  export let selectedFormats: string[] = [];
  export let selectedPipelineTags: string[] = [];
  export let selectedLibraries: string[] = [];
  export let selectedLanguages: string[] = [];
  export let selectedLicenses: string[] = [];
  export let authorFilter = '';
  export let isLoading = false;
  export let clearSearch: () => void;

  // Create setter functions for updating state and emitting events
  function setSearchQuery(value: string) { 
    dispatch('searchQueryChange', value);
  }
  
  function setSelectedFormats(value: string[]) { 
    dispatch('selectedFormatsChange', value);
  }
  
  function setSelectedPipelineTags(value: string[]) { 
    dispatch('selectedPipelineTagsChange', value);
  }
  
  function setSelectedLibraries(value: string[]) { 
    dispatch('selectedLibrariesChange', value);
  }
  
  function setSelectedLanguages(value: string[]) { 
    dispatch('selectedLanguagesChange', value);
  }
  
  function setSelectedLicenses(value: string[]) { 
    dispatch('selectedLicensesChange', value);
  }
  
  function setAuthorFilter(value: string) { 
    dispatch('authorFilterChange', value);
  }

  // Create event handlers
  const {
    handleSearch,
    handleToggleFormat,
    handleTogglePipelineTag,
    handleToggleLibrary,
    handleToggleLanguage,
    handleToggleLicense,
    handleClearFilters,
    handleRemoveSearchQuery,
    handleRemoveAuthorFilter,
    handleRemoveFormat,
    handleRemovePipelineTag,
    handleRemoveLibrary,
    handleRemoveLanguage,
    handleRemoveLicense
  } = createEventHandlers(
    dispatch,
    { searchQuery, selectedFormats, selectedPipelineTags, selectedLibraries, selectedLanguages, selectedLicenses, authorFilter },
    { setSearchQuery, setSelectedFormats, setSelectedPipelineTags, setSelectedLibraries, setSelectedLanguages, setSelectedLicenses, setAuthorFilter }
  );

  let filtersCollapsed = true;
  let activeFilters = false;

  // Reactive statement to update activeFilters when any filter changes
  $: activeFilters = hasActiveFilters({
    selectedFormats,
    selectedPipelineTags,
    selectedLibraries,
    selectedLanguages,
    selectedLicenses,
    authorFilter,
    searchQuery
  });
</script>

<div class="search-filters">
  <div class="filters-section">
    <FilterHeader bind:collapsed={filtersCollapsed} />
    {#if !filtersCollapsed}
      <div class="filters-content">
        <FormatFilter {selectedFormats} {isLoading} on:toggle={handleToggleFormat} />
        <AuthorFilter bind:authorFilter {isLoading} on:search={handleSearch} />
        <GenericFilter label="Тип задачи" options={availablePipelineTags} bind:selectedValues={selectedPipelineTags} {isLoading} on:toggle={handleTogglePipelineTag} />
        <GenericFilter label="Библиотека" options={availableLibraries} bind:selectedValues={selectedLibraries} {isLoading} on:toggle={handleToggleLibrary} />
        <GenericFilter label="Язык" options={availableLanguages} bind:selectedValues={selectedLanguages} {isLoading} on:toggle={handleToggleLanguage} />
        <GenericFilter label="Лицензия" options={availableLicenses} bind:selectedValues={selectedLicenses} {isLoading} on:toggle={handleToggleLicense} />
        <FilterActions {isLoading} hasActiveFilters={activeFilters} on:clear-filters={handleClearFilters} on:clear-search={clearSearch} />
      </div>
    {/if}
  </div>

  {#if activeFilters}
    <ActiveFilters 
      {searchQuery}
      {authorFilter}
      {selectedFormats}
      {selectedPipelineTags}
      {selectedLibraries}
      {selectedLanguages}
      {selectedLicenses}
      on:remove-search-query={handleRemoveSearchQuery}
      on:remove-author-filter={handleRemoveAuthorFilter}
      on:remove-format={handleRemoveFormat}
      on:remove-pipeline-tag={handleRemovePipelineTag}
      on:remove-library={handleRemoveLibrary}
      on:remove-language={handleRemoveLanguage}
      on:remove-license={handleRemoveLicense}
    />
  {/if}
</div>

<style>
  .search-filters {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 20px;
    margin-bottom: 0;
    width: 100%;
  }

  .filters-section {
    padding-top: 0;
    width: 100%;
  }

  .filters-content {
    animation: slideDown 0.3s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 1024px) {
    .search-filters {
      padding: 16px;
    }
  }

  @media (max-width: 768px) {
    .search-filters {
      padding: 12px;
    }
  }

  @media (max-width: 480px) {
    .search-filters {
      padding: 8px;
    }
  }
</style>