<script lang="ts">
  import type { FilterOption } from './filter-types';
  import { toggleInArray } from './filter-types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let label: string;
  export let options: FilterOption[];
  export let selected: string[];
  export let isLoading = false;

  function toggleOption(optionId: string) {
    selected = toggleInArray(selected, optionId);
    dispatch('change');
  }
</script>

<div class="generic-filter">
  <span class="filter-label">{label}:</span>
  <div class="filter-buttons">
    {#each options as option}
      <button
        class="filter-btn {selected.includes(option.id) ? 'active' : ''}"
        on:click={() => toggleOption(option.id)}
        disabled={isLoading}
      >
        {option.label}
      </button>
    {/each}
  </div>
</div>

<style>
  .generic-filter {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .filter-label {
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    min-width: 80px;
  }

  .filter-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .filter-btn {
    padding: 8px 16px;
    border: 2px solid var(--border-color);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-weight: 500;
    cursor: default;
    transition: all 0.2s ease;
    font-size: 14px;
  }

  .filter-btn:hover:not(:disabled) {
    border-color: var(--accent);
    background: rgba(179, 205, 224, 0.1);
  }

  .filter-btn.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .filter-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Responsive styles */
  @media (max-width: 1024px) {
    .generic-filter {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .filter-buttons {
      width: 100%;
      justify-content: flex-start;
    }

    .filter-label {
      min-width: 100px;
      font-size: 14px;
    }
  }

  @media (max-width: 768px) {
    .generic-filter {
      gap: 10px;
      margin-bottom: 16px;
    }

    .filter-buttons {
      gap: 6px;
    }

    .filter-label {
      min-width: unset;
      font-weight: 600;
    }

    .filter-btn {
      font-size: 13px;
      padding: 6px 12px;
    }
  }

  @media (max-width: 480px) {
    .generic-filter {
      margin-bottom: 12px;
      gap: 8px;
    }

    .filter-buttons {
      gap: 4px;
    }

    .filter-btn {
      font-size: 12px;
      padding: 5px 10px;
    }
  }
</style>