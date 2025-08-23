<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  
  export let searchQuery = '';
  export let selectedFormats: string[] = [];
  export let selectedPipelineTags: string[] = [];
  export let selectedLibraries: string[] = [];
  export let selectedLanguages: string[] = [];
  export let selectedLicenses: string[] = [];
  export let authorFilter = '';
  
  // Define the search function type
  type SearchFunction = (params: {
    query?: string;
    formats?: string[];
    pipelineTags?: string[];
    libraries?: string[];
    languages?: string[];
    licenses?: string[];
    author?: string;
  }) => void;
  
  export let searchModels: SearchFunction;
  
  // Popular search terms
  const popularSearches = ['llama', 'mistral', 'gemma', 'qwen'];
</script>

<div class="popular-searches">
  <h4>Популярные запросы:</h4>
  <div class="search-suggestions">
    {#each popularSearches as term}
      <button 
        class="suggestion-btn" 
        on:click={() => { 
          searchQuery = term; 
          searchModels({ 
            query: term, 
            formats: selectedFormats, 
            pipelineTags: selectedPipelineTags, 
            libraries: selectedLibraries, 
            languages: selectedLanguages, 
            licenses: selectedLicenses, 
            author: authorFilter 
          }); 
        }}
      >
        {term}
      </button>
    {/each}
  </div>
</div>

<style>
  .popular-searches {
    margin-top: 32px;
    padding-top: 32px;
    border-top: 1px solid var(--border-color);
  }

  .popular-searches h4 {
    font-size: 1.2rem;
    color: var(--text);
    margin: 0 0 20px 0;
  }

  .search-suggestions {
    display: flex;
    gap: 12px;
    justify-content: center;
    flex-wrap: wrap;
  }

  .suggestion-btn {
    background: var(--accent);
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: default;
    transition: all 0.2s ease;
  }

  .suggestion-btn:hover {
    background: var(--accent-2);
    transform: translateY(-1px);
  }

  @media (max-width: 768px) {
    .search-suggestions {
      flex-direction: column;
      align-items: center;
      gap: 8px;
    }

    .suggestion-btn {
      width: 100%;
      max-width: 200px;
      font-size: 14px;
      padding: 8px 16px;
    }
  }

  @media (max-width: 480px) {
    .suggestion-btn {
      padding: 5px 10px;
      font-size: 12px;
    }
  }
</style>