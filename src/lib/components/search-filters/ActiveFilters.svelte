<script lang="ts">
  import type { FilterState } from './filter-types';
  import { 
    availableFormats, 
    availablePipelineTags, 
    availableLibraries, 
    availableLanguages, 
    availableLicenses 
  } from './filter-types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let filterState: FilterState;

  function removeFilter(type: string, value?: string) {
    dispatch('remove-filter', { type, value });
  }

  function findLabel(options: any[], id: string): string {
    return options.find(option => option.id === id)?.label || id;
  }
</script>

<div class="active-filters">
  <span class="active-filters-label">Активные фильтры:</span>
  
  {#if filterState.searchQuery}
    <span class="active-filter">
      Поиск: "{filterState.searchQuery}"
      <button class="remove-filter" on:click={() => removeFilter('searchQuery')}>
        ×
      </button>
    </span>
  {/if}
  
  {#if filterState.authorFilter}
    <span class="active-filter">
      Автор: "{filterState.authorFilter}"
      <button class="remove-filter" on:click={() => removeFilter('authorFilter')}>
        ×
      </button>
    </span>
  {/if}
  
  {#each filterState.selectedFormats as format}
    <span class="active-filter">
      Формат: {findLabel(availableFormats, format)}
      <button class="remove-filter" on:click={() => removeFilter('format', format)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each filterState.selectedPipelineTags as tag}
    <span class="active-filter">
      Задача: {findLabel(availablePipelineTags, tag)}
      <button class="remove-filter" on:click={() => removeFilter('pipelineTag', tag)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each filterState.selectedLibraries as library}
    <span class="active-filter">
      Библиотека: {findLabel(availableLibraries, library)}
      <button class="remove-filter" on:click={() => removeFilter('library', library)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each filterState.selectedLanguages as language}
    <span class="active-filter">
      Язык: {findLabel(availableLanguages, language)}
      <button class="remove-filter" on:click={() => removeFilter('language', language)}>
        ×
      </button>
    </span>
  {/each}
  
  {#each filterState.selectedLicenses as license}
    <span class="active-filter">
      Лицензия: {findLabel(availableLicenses, license)}
      <button class="remove-filter" on:click={() => removeFilter('license', license)}>
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