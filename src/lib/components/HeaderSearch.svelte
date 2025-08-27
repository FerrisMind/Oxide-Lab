<script lang="ts">
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    search: { query: string };
  }>();

  export let searchQuery = '';
  export let placeholder = 'Поиск моделей...';
  export let disabled = false;

  function handleSearch() {
    dispatch('search', { query: searchQuery });
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="header-search">
  <div class="file-row">
    <div class="input-with-button">
      <input
        type="text"
        bind:value={searchQuery}
        on:keypress={handleKeyPress}
        {placeholder}
        {disabled}
        class="search-input"
      />
      <button 
        class="inside-btn" 
        on:click={handleSearch}
        {disabled}
        title="Поиск"
        aria-label="Поиск моделей"
      >
        <MagnifyingGlass size={16} weight="bold" />
      </button>
    </div>
  </div>
</div>

<style>
  .header-search { 
    display: inline-flex; 
    align-items: center; 
    gap: 12px; 
    height: 100%; 
  }
  
  .file-row { 
    display: flex; 
    gap: 8px; 
    align-items: center; 
    height: 100%; 
  }
  
  .input-with-button { 
    display: inline-flex; 
    align-items: center; 
    gap: 8px; 
    position: relative; 
  }
  
  .search-input { 
    width: 280px; 
    padding: 6px 8px; 
    padding-right: 44px; 
    border-radius: 6px; 
    border: 1px solid var(--border-color); 
    background: var(--card); 
    color: var(--text); 
    height: 36px; 
    line-height: 24px;
  }
  
  .search-input::placeholder {
    color: var(--muted);
  }
  
  .search-input:disabled {
    opacity: 0.6;
      cursor: default;
  }
  
  .inside-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%) !important; /* override global button hover transform */
    width: 24px !important;
    height: 24px !important;
    padding: 0 !important; /* remove extra internal padding */
    box-sizing: border-box;
    border-radius: 4px;
    border: none;
    background: transparent;
    cursor: default;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 0;
    transition: background 0.12s ease;
  }
  
  /* фиксируем размер svg-иконки чтобы она не масштабировалась */
  .inside-btn :global(svg) { 
    color: var(--muted); 
    width: 16px; 
    height: 16px; 
  }
  
  .inside-btn:hover {
    background: var(--accent);
    transform: translateY(-50%) !important;
  }
  
  .inside-btn:hover :global(svg) { 
    color: #fff; 
  }
  
  .inside-btn:active {
    background: color-mix(in srgb, var(--accent) 80%, black 10%);
  }
  
  .inside-btn:disabled {
    opacity: 0.6;
     cursor: default;
  }
</style>