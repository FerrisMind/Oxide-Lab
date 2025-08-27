<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let authorFilter: string;
  export let isLoading = false;

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      dispatch('search');
    }
  }

  function handleInput() {
    dispatch('input');
  }
</script>

<div class="author-filter">
  <span class="filter-label">Автор:</span>
  <input
    type="text"
    placeholder="Например: microsoft, google, meta-llama..."
    bind:value={authorFilter}
    on:keypress={handleKeyPress}
    on:input={handleInput}
    class="author-input"
    disabled={isLoading}
  />
</div>

<style>
  .author-filter {
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
    min-width: 80px;
  }

  .author-input {
    flex: 1;
    min-width: 200px;
    padding: 8px 12px;
    border: 2px solid var(--border-color);
    border-radius: 8px;
    font-size: 14px;
    background: var(--bg);
    color: var(--text);
    outline: none;
    transition: all 0.2s ease;
  }

  .author-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(179, 205, 224, 0.1);
  }

  .author-input:disabled {
    opacity: 0.6;
      cursor: default;
  }

  /* Responsive styles */
  @media (max-width: 1024px) {
    .author-filter {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .filter-label {
      min-width: 100px;
      font-size: 14px;
    }
  }

  @media (max-width: 768px) {
    .author-filter {
      gap: 10px;
      margin-bottom: 16px;
    }

    .author-input {
      width: 100%;
      min-width: unset;
      font-size: 16px; /* Prevents zoom on iOS */
    }

    .filter-label {
      min-width: unset;
      font-weight: 600;
    }
  }

  @media (max-width: 480px) {
    .author-filter {
      margin-bottom: 12px;
      gap: 8px;
    }

    .author-input {
      padding: 8px 10px;
    }
  }
</style>