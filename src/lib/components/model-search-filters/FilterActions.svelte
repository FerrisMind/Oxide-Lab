<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isLoading = false;
  export let hasActiveFilters = false;

  const dispatch = createEventDispatcher();

  function clearFilters() {
    dispatch('clear-filters');
  }

  function clearSearch() {
    dispatch('clear-search');
  }
</script>

<div class="filter-actions">
  <button 
    class="clear-filters-btn" 
    on:click={clearFilters}
    disabled={isLoading || !hasActiveFilters}
  >
    Очистить фильтры
  </button>
  
  <button 
    class="clear-search-btn" 
    on:click={clearSearch}
    disabled={isLoading || !hasActiveFilters}
  >
    Очистить поиск
  </button>
</div>

<style>
  .filter-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .clear-filters-btn,
  .clear-search-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--muted);
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 14px;
    cursor: default;
    transition: all 0.2s ease;
  }

  .clear-filters-btn:hover:not(:disabled),
  .clear-search-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .clear-search-btn {
    background: var(--accent-2);
    color: white;
    border-color: var(--accent-2);
  }

  .clear-search-btn:hover:not(:disabled) {
    background: var(--accent);
    border-color: var(--accent);
  }

  .clear-filters-btn:disabled,
  .clear-search-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>