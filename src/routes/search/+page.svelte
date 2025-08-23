<script lang="ts">
  import { onMount } from 'svelte';
  import { headerSearchQuery, searchTrigger } from '$lib/stores/search';
  import ModelSearchFilters from '$lib/components/ModelSearchFilters.svelte';
  import SearchLayout from '$lib/components/SearchLayout.svelte';
  import { huggingFaceService } from '$lib/services/huggingface';
  import type { HFModel, SearchResult } from '$lib/services/huggingface';
  import { rightSidebarOpen } from '$lib/stores/sidebar';

  let searchQuery = '';
  let selectedFormats: string[] = [];
  let selectedPipelineTags: string[] = [];
  let selectedLibraries: string[] = [];
  let selectedLanguages: string[] = [];
  let selectedLicenses: string[] = [];
  let authorFilter = '';
  let isLoading = false;
  let isLoadingMore = false;
  let models: HFModel[] = [];
  let error: string | null = null;
  let hasSearched = false;
  let hasMore = false;
  let nextOffset = 0;
  let totalCount = 0;
  
  // Состояние выбранной модели
  let selectedModel: HFModel | null = null;
  let modelDetailLoading = false;

  // Убрали mock данные - теперь используем сервис

  // Функция поиска моделей
  async function searchModels({
    query = searchQuery,
    formats = selectedFormats,
    pipelineTags = selectedPipelineTags,
    libraries = selectedLibraries,
    languages = selectedLanguages,
    licenses = selectedLicenses,
    author = authorFilter,
    loadMore = false
  } = {}) {
    if (loadMore) {
      isLoadingMore = true;
    } else {
      isLoading = true;
      models = [];
      nextOffset = 0;
    }
    
    error = null;
    
    try {
      const result: SearchResult = await huggingFaceService.searchModels({
        query: query.trim(),
        formats,
        pipeline_tag: pipelineTags[0], // API принимает только один тег
        library: libraries,
        language: languages,
        license: licenses,
        author: author?.trim(),
        limit: 20,
        offset: loadMore ? nextOffset : 0,
        sort: 'downloads',
        order: 'desc'
      });
      
      if (loadMore) {
        models = [...models, ...result.models];
      } else {
        models = result.models;
      }
      
      hasMore = result.hasMore;
      nextOffset = result.nextOffset || 0;
      totalCount = result.totalCount;
      hasSearched = true;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Произошла ошибка при поиске';
      if (!loadMore) {
        models = [];
      }
    } finally {
      isLoading = false;
      isLoadingMore = false;
    }
  }

  async function loadMoreModels() {
    if (!hasMore || isLoadingMore) return;
    await searchModels({ loadMore: true });
  }

  // Обработчик поиска
  function handleSearch(event: CustomEvent) {
    const { query, formats, pipelineTags, libraries, languages, licenses, author } = event.detail;
    searchQuery = query;
    selectedFormats = formats;
    selectedPipelineTags = pipelineTags;
    selectedLibraries = libraries;
    selectedLanguages = languages;
    selectedLicenses = licenses;
    authorFilter = author;
    
    searchModels({
      query,
      formats,
      pipelineTags,
      libraries,
      languages,
      licenses,
      author
    });
  }

  // Добавление модели в менеджер
  function addModelToManager(modelId: string) {
    // Здесь будет логика добавления модели в менеджер
    console.log('Adding model to manager:', modelId);
    // Можно открыть правую панель с настройками для этой модели
    rightSidebarOpen.set(true);
  }

  // Обработка выбора модели
  function handleModelSelect(event: CustomEvent<{ model: HFModel }>) {
    selectedModel = event.detail.model;
    modelDetailLoading = true;
    
    // Имитация загрузки деталей (в реальности детали загружаются в компоненте ModelDetail)
    setTimeout(() => {
      modelDetailLoading = false;
    }, 100);
  }

  // Загрузка популярных моделей при монтировании
  onMount(() => {
    // Можно загрузить популярные модели по умолчанию
  });

  // Слушаем поиск из хедера
  $: if ($searchTrigger) {
    searchQuery = $headerSearchQuery;
    searchModels({ query: searchQuery });
  }
  
  // Синхронизируем с хедером при изменении запроса на странице
  $: headerSearchQuery.set(searchQuery);
</script>

<main class="wrap" class:sidebar-open={$rightSidebarOpen}>
  <div class="search-page">
    <!-- Фильтры поиска на всю ширину -->
    <div class="filters-container">
      <ModelSearchFilters
        bind:searchQuery
        bind:selectedFormats
        bind:selectedPipelineTags
        bind:selectedLibraries
        bind:selectedLanguages
        bind:selectedLicenses
        bind:authorFilter
        {isLoading}
        clearSearch={() => {
          searchQuery = '';
          selectedFormats = [];
          selectedPipelineTags = [];
          selectedLibraries = [];
          selectedLanguages = [];
          selectedLicenses = [];
          authorFilter = '';
          models = [];
          hasSearched = false;
          hasMore = false;
          nextOffset = 0;
          totalCount = 0;
          error = null;
        }}
        on:search={handleSearch}
      />
    </div>

    <!-- Секция результатов -->
    <div class="results-section">
      {#if hasSearched}
        {#if isLoading}
          <div class="loading-state">
            <div class="loading-spinner"></div>
            <p>Поиск моделей...</p>
          </div>
        {:else if error}
          <div class="error-state">
            <p class="error-message">{error}</p>
            <button class="retry-btn" on:click={() => searchModels()}>
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
        <!-- Начальное состояние -->
        <div class="initial-state">
          <div class="welcome-card">
            <h3>Добро пожаловать в поиск моделей!</h3>
            <p>Используйте поисковую строку выше, чтобы найти модели по названию, описанию или тегам.</p>
            <p>Фильтры помогут вам найти модели в нужных форматах (GGUF или Safetensors).</p>
            
            <div class="popular-searches">
              <h4>Популярные запросы:</h4>
              <div class="search-suggestions">
                <button class="suggestion-btn" on:click={() => { 
                  searchQuery = 'llama'; 
                  searchModels({ query: 'llama', formats: selectedFormats, pipelineTags: selectedPipelineTags, libraries: selectedLibraries, languages: selectedLanguages, licenses: selectedLicenses, author: authorFilter }); 
                }}>
                  llama
                </button>
                <button class="suggestion-btn" on:click={() => { 
                  searchQuery = 'mistral'; 
                  searchModels({ query: 'mistral', formats: selectedFormats, pipelineTags: selectedPipelineTags, libraries: selectedLibraries, languages: selectedLanguages, licenses: selectedLicenses, author: authorFilter }); 
                }}>
                  mistral
                </button>
                <button class="suggestion-btn" on:click={() => { 
                  searchQuery = 'gemma'; 
                  searchModels({ query: 'gemma', formats: selectedFormats, pipelineTags: selectedPipelineTags, libraries: selectedLibraries, languages: selectedLanguages, licenses: selectedLicenses, author: authorFilter }); 
                }}>
                  gemma
                </button>
                <button class="suggestion-btn" on:click={() => { 
                  searchQuery = 'qwen'; 
                  searchModels({ query: 'qwen', formats: selectedFormats, pipelineTags: selectedPipelineTags, libraries: selectedLibraries, languages: selectedLanguages, licenses: selectedLicenses, author: authorFilter }); 
                }}>
                  qwen
                </button>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</main>

<style>
  .search-page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 56px - 32px);
    max-height: calc(100vh - 56px - 32px);
    background: var(--bg);
    width: 100%;
    overflow: hidden;
  }

  .filters-container {
    flex-shrink: 0;
    width: 100%;
    max-width: none;
    background: var(--panel-bg);
    border-bottom: 1px solid var(--border);
    padding: 0;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: sticky;
    top: 0;
    z-index: 10;
    border-radius: 12px 12px 12px 12px;
  }

  .results-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0 20px;
    width: 100%;
    min-width: 0;
  }



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

  /* Адаптивность */
  @media (max-width: 1024px) {
    .filters-container {
      padding: 16px;
    }

    .results-section {
      padding: 16px;
    }

    .welcome-card {
      padding: 32px 24px;
      max-width: 500px;
    }

    .suggestion-btn {
      font-size: 14px;
      padding: 8px 16px;
    }
  }

  @media (max-width: 768px) {
    .search-page {
      height: auto;
      min-height: calc(100vh - 60px);
    }

    .filters-container {
      padding: 12px;
    }

    .results-section {
      padding: 12px;
    }

    .results-layout {
      gap: 12px;
    }

    .welcome-card {
      padding: 24px 16px;
      max-width: 400px;
      margin: 0;
    }

    .search-suggestions {
      flex-direction: column;
      align-items: center;
      gap: 8px;
    }

    .suggestion-btn {
      width: 100%;
      max-width: 200px;
      font-size: 13px;
      padding: 6px 12px;
    }


  }

  @media (max-width: 480px) {
    .results-section {
      padding: 8px;
    }



    .welcome-card {
      padding: 20px 12px;
      max-width: 350px;
    }

    .suggestion-btn {
      padding: 5px 10px;
      font-size: 12px;
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