<script lang="ts">
  import PopularSearches from './PopularSearches.svelte';
  
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
</script>

<div class="initial-state">
  <div class="welcome-card">
    <h3>Добро пожаловать в поиск моделей!</h3>
    <p>Используйте поисковую строку выше, чтобы найти модели по названию, описанию или тегам.</p>
    <p>Фильтры помогут вам найти модели в нужных форматах (GGUF или Safetensors).</p>
    
    <PopularSearches 
      bind:searchQuery
      bind:selectedFormats
      bind:selectedPipelineTags
      bind:selectedLibraries
      bind:selectedLanguages
      bind:selectedLicenses
      bind:authorFilter
      {searchModels}
    />
  </div>
</div>

<style>
  .initial-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
    padding: 40px 20px;
    overflow: hidden;
  }

  .welcome-card {
    background: var(--panel-bg);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 48px 32px;
    max-width: 600px;
    width: 100%;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
    overflow: hidden;
  }

  .welcome-card h3 {
    font-size: 1.8rem;
    color: var(--text);
    margin: 0 0 20px 0;
  }

  .welcome-card p {
    color: var(--muted);
    font-size: 1.1rem;
    line-height: 1.6;
    margin: 0 0 16px 0;
  }

  @media (max-width: 1024px) {
    .welcome-card {
      padding: 32px 24px;
      max-width: 500px;
    }
  }

  @media (max-width: 768px) {
    .welcome-card {
      padding: 24px 16px;
      max-width: 400px;
      margin: 0;
    }
  }

  @media (max-width: 480px) {
    .welcome-card {
      padding: 20px 12px;
      max-width: 350px;
    }
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .welcome-card {
      background: var(--card);
      border-color: var(--border-color);
    }
  }
</style>