<script lang="ts">
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import Funnel from 'phosphor-svelte/lib/Funnel';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let searchQuery = '';
  export let selectedFormats: string[] = [];
  export let isLoading = false;

  const availableFormats = [
    { id: 'gguf', label: 'GGUF', color: '#10b981' },
    { id: 'safetensors', label: 'Safetensors', color: '#3b82f6' }
  ];

  function handleSearch() {
    dispatch('search', { query: searchQuery, formats: selectedFormats });
  }

  function toggleFormat(format: string) {
    if (selectedFormats.includes(format)) {
      selectedFormats = selectedFormats.filter(f => f !== format);
    } else {
      selectedFormats = [...selectedFormats, format];
    }
    handleSearch();
  }

  function clearFilters() {
    selectedFormats = [];
    searchQuery = '';
    handleSearch();
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="search-filters">
  <div class="search-section">
    <div class="search-input-wrapper">
      <MagnifyingGlass size={20} class="search-icon" />
      <input
        type="text"
        placeholder="Поиск моделей (например: llama, mistral, gemma)..."
        bind:value={searchQuery}
        on:keypress={handleKeyPress}
        class="search-input"
        disabled={isLoading}
      />
      <button 
        class="search-btn" 
        on:click={handleSearch}
        disabled={isLoading}
      >
        {isLoading ? 'Поиск...' : 'Найти'}
      </button>
    </div>
  </div>

  <div class="filters-section">
    <div class="filters-header">
      <Funnel size={16} />
      <span>Фильтры</span>
    </div>
    
    <div class="format-filters">
      <span class="filter-label">Форматы:</span>
      <div class="format-buttons">
        {#each availableFormats as format}
          <button
            class="format-btn {selectedFormats.includes(format.id) ? 'active' : ''}"
            style="--format-color: {format.color}"
            on:click={() => toggleFormat(format.id)}
            disabled={isLoading}
          >
            {format.label}
          </button>
        {/each}
      </div>
    </div>

    <button 
      class="clear-filters-btn" 
      on:click={clearFilters}
      disabled={isLoading || (selectedFormats.length === 0 && !searchQuery)}
    >
      Очистить фильтры
    </button>
  </div>

  {#if selectedFormats.length > 0 || searchQuery}
    <div class="active-filters">
      <span class="active-filters-label">Активные фильтры:</span>
      {#if searchQuery}
        <span class="active-filter">
          Поиск: "{searchQuery}"
          <button class="remove-filter" on:click={() => { searchQuery = ''; handleSearch(); }}>
            ×
          </button>
        </span>
      {/if}
      {#each selectedFormats as format}
        <span class="active-filter">
          {availableFormats.find(f => f.id === format)?.label}
          <button class="remove-filter" on:click={() => toggleFormat(format)}>
            ×
          </button>
        </span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-filters {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 24px;
    margin-bottom: 24px;
  }

  .search-section {
    margin-bottom: 24px;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: 12px;
    position: relative;
  }

  .search-icon {
    color: var(--muted);
    position: absolute;
    left: 16px;
    z-index: 1;
  }

  .search-input {
    flex: 1;
    padding: 12px 16px 12px 48px;
    border: 2px solid var(--border-color);
    border-radius: 10px;
    font-size: 16px;
    background: var(--bg);
    color: var(--text);
    outline: none;
    transition: all 0.2s ease;
  }

  .search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(179, 205, 224, 0.1);
  }

  .search-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .search-btn {
    background: var(--accent-2);
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 10px;
    font-size: 16px;
    font-weight: 600;
     cursor: default;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .search-btn:hover:not(:disabled) {
    background: var(--accent);
    transform: translateY(-1px);
  }

  .search-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .filters-section {
    border-top: 1px solid var(--border-color);
    padding-top: 24px;
  }

  .filters-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .format-filters {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .filter-label {
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
  }

  .format-buttons {
    display: flex;
    gap: 8px;
  }

  .format-btn {
    padding: 8px 16px;
    border: 2px solid var(--border-color);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-weight: 500;
     cursor: default;
    transition: all 0.2s ease;
    position: relative;
    overflow: hidden;
  }

  .format-btn::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--format-color);
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .format-btn:hover::before {
    opacity: 0.1;
  }

  .format-btn.active {
    background: var(--format-color);
    color: white;
    border-color: var(--format-color);
  }

  .format-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .clear-filters-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--muted);
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 14px;
     cursor: default;
    transition: all 0.2s ease;
  }

  .clear-filters-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .clear-filters-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

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

  /* Адаптивность */
  @media (max-width: 768px) {
    .search-filters {
      padding: 16px;
    }

    .search-input-wrapper {
      flex-direction: column;
      gap: 12px;
    }

    .search-input {
      width: 100%;
    }

    .format-filters {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .format-buttons {
      width: 100%;
      justify-content: flex-start;
    }
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .search-input {
      background: var(--card);
      border-color: var(--border-color);
    }

    .format-btn {
      border-color: var(--border-color);
    }
  }
</style>
