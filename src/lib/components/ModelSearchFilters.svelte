<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SearchEvent } from './model-search-filters/filter-types';
  import { 
    availablePipelineTags, 
    availableLibraries, 
    availableLanguages, 
    availableLicenses,
    hasActiveFilters,
    clearAllFilters
  } from './model-search-filters/filter-types';

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

  let filtersCollapsed = true;

  function handleSearch() {
    dispatch('search', { 
      query: searchQuery, 
      formats: selectedFormats,
      pipelineTags: selectedPipelineTags,
      libraries: selectedLibraries,
      languages: selectedLanguages,
      licenses: selectedLicenses,
      author: authorFilter
    } as SearchEvent);
  }

  function handleToggleFormat(event: CustomEvent<string>) {
    const formatId = event.detail;
    if (selectedFormats.includes(formatId)) {
      selectedFormats = selectedFormats.filter(f => f !== formatId);
    } else {
      selectedFormats = [...selectedFormats, formatId];
    }
    handleSearch();
  }

  function handleTogglePipelineTag(event: CustomEvent<string>) {
    const tagId = event.detail;
    if (selectedPipelineTags.includes(tagId)) {
      selectedPipelineTags = selectedPipelineTags.filter(t => t !== tagId);
    } else {
      selectedPipelineTags = [...selectedPipelineTags, tagId];
    }
    handleSearch();
  }

  function handleToggleLibrary(event: CustomEvent<string>) {
    const libraryId = event.detail;
    if (selectedLibraries.includes(libraryId)) {
      selectedLibraries = selectedLibraries.filter(l => l !== libraryId);
    } else {
      selectedLibraries = [...selectedLibraries, libraryId];
    }
    handleSearch();
  }

  function handleToggleLanguage(event: CustomEvent<string>) {
    const languageId = event.detail;
    if (selectedLanguages.includes(languageId)) {
      selectedLanguages = selectedLanguages.filter(l => l !== languageId);
    } else {
      selectedLanguages = [...selectedLanguages, languageId];
    }
    handleSearch();
  }

  function handleToggleLicense(event: CustomEvent<string>) {
    const licenseId = event.detail;
    if (selectedLicenses.includes(licenseId)) {
      selectedLicenses = selectedLicenses.filter(l => l !== licenseId);
    } else {
      selectedLicenses = [...selectedLicenses, licenseId];
    }
    handleSearch();
  }

  function handleClearFilters() {
    const clearedState = clearAllFilters();
    selectedFormats = clearedState.selectedFormats;
    selectedPipelineTags = clearedState.selectedPipelineTags;
    selectedLibraries = clearedState.selectedLibraries;
    selectedLanguages = clearedState.selectedLanguages;
    selectedLicenses = clearedState.selectedLicenses;
    searchQuery = clearedState.searchQuery;
    authorFilter = clearedState.authorFilter;
    handleSearch();
  }

  function handleRemoveSearchQuery() {
    searchQuery = '';
    handleSearch();
  }

  function handleRemoveAuthorFilter() {
    authorFilter = '';
    handleSearch();
  }

  function handleRemoveFormat(event: CustomEvent<string>) {
    const formatId = event.detail;
    selectedFormats = selectedFormats.filter(f => f !== formatId);
    handleSearch();
  }

  function handleRemovePipelineTag(event: CustomEvent<string>) {
    const tagId = event.detail;
    selectedPipelineTags = selectedPipelineTags.filter(t => t !== tagId);
    handleSearch();
  }

  function handleRemoveLibrary(event: CustomEvent<string>) {
    const libraryId = event.detail;
    selectedLibraries = selectedLibraries.filter(l => l !== libraryId);
    handleSearch();
  }

  function handleRemoveLanguage(event: CustomEvent<string>) {
    const languageId = event.detail;
    selectedLanguages = selectedLanguages.filter(l => l !== languageId);
    handleSearch();
  }

  function handleRemoveLicense(event: CustomEvent<string>) {
    const licenseId = event.detail;
    selectedLicenses = selectedLicenses.filter(l => l !== licenseId);
    handleSearch();
  }
</script>

<div class="search-filters">
  <div class="filters-section">
    <FilterHeader bind:collapsed={filtersCollapsed} />
    
    {#if !filtersCollapsed}
      <div class="filters-content">
        <FormatFilter 
          {selectedFormats} 
          {isLoading} 
          on:toggle={handleToggleFormat} 
        />
        
        <AuthorFilter 
          bind:authorFilter 
          {isLoading} 
          on:search={handleSearch} 
        />
        
        <GenericFilter 
          label="Тип задачи"
          options={availablePipelineTags}
          bind:selectedValues={selectedPipelineTags}
          {isLoading}
          on:toggle={handleTogglePipelineTag}
        />
        
        <GenericFilter 
          label="Библиотека"
          options={availableLibraries}
          bind:selectedValues={selectedLibraries}
          {isLoading}
          on:toggle={handleToggleLibrary}
        />
        
        <GenericFilter 
          label="Язык"
          options={availableLanguages}
          bind:selectedValues={selectedLanguages}
          {isLoading}
          on:toggle={handleToggleLanguage}
        />
        
        <GenericFilter 
          label="Лицензия"
          options={availableLicenses}
          bind:selectedValues={selectedLicenses}
          {isLoading}
          on:toggle={handleToggleLicense}
        />
        
        <FilterActions 
          {isLoading} 
          hasActiveFilters={hasActiveFilters({
            selectedFormats,
            selectedPipelineTags,
            selectedLibraries,
            selectedLanguages,
            selectedLicenses,
            authorFilter,
            searchQuery
          })}
          on:clear-filters={handleClearFilters}
          on:clear-search={clearSearch}
        />
      </div>
    {/if}
  </div>

  {#if hasActiveFilters({
    selectedFormats,
    selectedPipelineTags,
    selectedLibraries,
    selectedLanguages,
    selectedLicenses,
    authorFilter,
    searchQuery
  })}
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
    max-width: none;
    box-sizing: border-box;
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

  /* Responsive */
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
