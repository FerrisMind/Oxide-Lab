<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { FilterOption } from './filter-types';
  import { 
    availableFormats, 
    availablePipelineTags, 
    availableLibraries, 
    availableLanguages, 
    availableLicenses 
  } from './filter-types';

  export let searchQuery: string;
  export let authorFilter: string;
  export let selectedFormats: string[];
  export let selectedPipelineTags: string[];
  export let selectedLibraries: string[];
  export let selectedLanguages: string[];
  export let selectedLicenses: string[];

  const dispatch = createEventDispatcher();

  function removeSearchQuery() {
    dispatch('remove-search-query');
  }

  function removeAuthorFilter() {
    dispatch('remove-author-filter');
  }

  function removeFormat(formatId: string) {
    dispatch('remove-format', formatId);
  }

  function removePipelineTag(tagId: string) {
    dispatch('remove-pipeline-tag', tagId);
  }

  function removeLibrary(libraryId: string) {
    dispatch('remove-library', libraryId);
  }

  function removeLanguage(languageId: string) {
    dispatch('remove-language', languageId);
  }

  function removeLicense(licenseId: string) {
    dispatch('remove-license', licenseId);
  }

  function getLabel(options: FilterOption[], id: string): string {
    const option = options.find(opt => opt.id === id);
    return option ? option.label : id;
  }
</script>

<div class="active-filters">
  <span class="active-filters-label">Активные фильтры:</span>
  
  {#if searchQuery}
    <span class="active-filter">
      Поиск: "{searchQuery}"
      <button class="remove-filter" on:click={removeSearchQuery}>
        ×
      </button>
    </span>
  {/if}
  
  {#if authorFilter}
    <span class="active-filter">
      Автор: "{authorFilter}"
      <button class="remove-filter" on:click={removeAuthorFilter}>
        ×
      </button>
    </span>
  {/if}
  
  {#each selectedFormats as format}
    <span class="active-filter">
      Формат: {getLabel(availableFormats, format)}
      <button class="remove-filter" on:click={() => removeFormat(format)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each selectedPipelineTags as tag}
    <span class="active-filter">
      Задача: {getLabel(availablePipelineTags, tag)}
      <button class="remove-filter" on:click={() => removePipelineTag(tag)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each selectedLibraries as library}
    <span class="active-filter">
      Библиотека: {getLabel(availableLibraries, library)}
      <button class="remove-filter" on:click={() => removeLibrary(library)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each selectedLanguages as language}
    <span class="active-filter">
      Язык: {getLabel(availableLanguages, language)}
      <button class="remove-filter" on:click={() => removeLanguage(language)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each selectedLicenses as license}
    <span class="active-filter">
      Лицензия: {getLabel(availableLicenses, license)}
      <button class="remove-filter" on:click={() => removeLicense(license)}>
        ×
      </button>
    </span>
  {/each}
</div>

<style>
  .active-filters {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-top: 20px;
    padding-top: 20px;
    border-top: 1px solid var(--border-color);
  }

  .active-filters-label {
    font-size: 14px;
    color: var(--muted);
    font-weight: 500;
  }

  .active-filter {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--accent);
    color: white;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
  }

  .remove-filter {
    background: none;
    border: none;
    color: white;
    font-size: 16px;
    font-weight: bold;
    cursor: default;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: background 0.2s ease;
  }

  .remove-filter:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>