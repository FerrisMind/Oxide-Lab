<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { availableFormats } from './filter-types';

  export let selectedFormats: string[];
  export let isLoading = false;

  const dispatch = createEventDispatcher();

  function toggleFormat(formatId: string) {
    dispatch('toggle', formatId);
  }
</script>

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

<style>
  .format-filters {
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

  .format-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
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
    font-size: 14px;
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
      cursor: default;
  }

  /* Responsive */
  @media (max-width: 1024px) {
    .format-filters {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .format-buttons {
      width: 100%;
      justify-content: flex-start;
    }

    .filter-label {
      min-width: 100px;
      font-size: 14px;
    }
  }

  @media (max-width: 768px) {
    .format-filters {
      gap: 10px;
      margin-bottom: 16px;
    }

    .format-buttons {
      gap: 6px;
    }

    .filter-label {
      min-width: unset;
      font-size: 14px;
      font-weight: 600;
    }

    .format-btn {
      font-size: 13px;
      padding: 6px 12px;
    }
  }

  @media (max-width: 480px) {
    .format-filters {
      margin-bottom: 12px;
      gap: 8px;
    }

    .format-buttons {
      gap: 4px;
    }

    .format-btn {
      font-size: 12px;
      padding: 5px 10px;
    }
  }
</style>