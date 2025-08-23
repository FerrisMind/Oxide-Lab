<script lang="ts">
  import SearchLayout from '$lib/components/SearchLayout.svelte';
  import type { HFModel } from '$lib/services/huggingface';
  
  export let models: HFModel[] = [];
  export let selectedModel: HFModel | null = null;
  export let isLoading = false;
  export let hasMore = false;
  export let totalCount = 0;
  export let error: string | null = null;
  export let hasSearched = false;
  
  // Function types
  type ModelSelectFunction = (event: CustomEvent<{ model: HFModel }>) => void;
  type LoadMoreFunction = () => void;
  
  export let handleModelSelect: ModelSelectFunction;
  export let loadMoreModels: LoadMoreFunction;
</script>

{#if hasSearched}
  {#if isLoading}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Поиск моделей...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p class="error-message">{error}</p>
      <button class="retry-btn" on:click={() => loadMoreModels()}>
        Попробовать снова
      </button>
    </div>
  {:else if models.length === 0}
    <div class="no-results">
      <p>По вашему запросу ничего не найдено.</p>
      <p>Попробуйте изменить поисковый запрос или фильтры.</p>
    </div>
  {:else}
    <div class="results-layout">
      <SearchLayout 
        {models}
        selectedModel={selectedModel}
        isLoading={isLoading}
        hasMore={hasMore}
        totalCount={totalCount}
        on:selectModel={handleModelSelect}
        on:loadMore={loadMoreModels}
      />
    </div>
  {/if}
{:else}
  <slot />
{/if}

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    color: var(--muted);
    padding: 40px 20px;
    overflow: hidden;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border-color);
    border-top: 4px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 20px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    color: var(--muted);
    padding: 40px 20px;
    background: var(--panel-alt-bg);
    border-radius: 12px;
    border: 1px solid var(--border-color);
    overflow: hidden;
  }

  .error-message {
    color: #ef4444;
    font-size: 1.1rem;
    margin: 0 0 20px 0;
  }

  .retry-btn {
    background: var(--accent);
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: default;
    transition: all 0.2s ease;
  }

  .retry-btn:hover {
    background: var(--accent-2);
    transform: translateY(-1px);
  }

  .no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    color: var(--muted);
    padding: 40px 20px;
    overflow: hidden;
  }

  .no-results p {
    margin: 8px 0;
    font-size: 1.1rem;
  }

  .results-layout {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    min-height: 0;
    overflow: hidden;
    padding: 20px 0;
  }
</style>