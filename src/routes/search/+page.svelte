<script lang="ts">
  import { onMount } from 'svelte';
  import ModelSearchFilters from '$lib/components/ModelSearchFilters.svelte';
  import ModelCard from '$lib/components/ModelCard.svelte';
  import { rightSidebarOpen } from '$lib/stores/sidebar';
  import { huggingFaceService, type HFModel } from '$lib/services/huggingface';

  // Состояние поиска
  let searchQuery = '';
  let selectedFormats: string[] = [];
  let isLoading = false;
  let models: HFModel[] = [];
  let error: string | null = null;
  let hasSearched = false;

  // Убрали mock данные - теперь используем сервис

  // Функция поиска моделей
  async function searchModels(query: string, formats: string[]) {
    isLoading = true;
    error = null;
    hasSearched = true;

    try {
      // Используем сервис для поиска моделей
      const searchParams = {
        query: query.trim(),
        formats: formats,
        limit: 50,
        sort: 'downloads' as const,
        order: 'desc' as const
      };

      models = await huggingFaceService.searchModels(searchParams);
    } catch (err) {
      error = 'Ошибка при поиске моделей. Попробуйте еще раз.';
      console.error('Search error:', err);
    } finally {
      isLoading = false;
    }
  }

  // Обработчик поиска
  function handleSearch(event: CustomEvent) {
    const { query, formats } = event.detail;
    searchQuery = query;
    selectedFormats = formats;
    searchModels(query, formats);
  }

  // Добавление модели в менеджер
  function addModelToManager(modelId: string) {
    // Здесь будет логика добавления модели в менеджер
    console.log('Adding model to manager:', modelId);
    // Можно открыть правую панель с настройками для этой модели
    rightSidebarOpen.set(true);
  }

  // Загрузка популярных моделей при монтировании
  onMount(() => {
    // Можно загрузить популярные модели по умолчанию
  });
</script>

<main class="wrap" class:sidebar-open={$rightSidebarOpen}>
  <div class="search-page">
    <div class="page-header">
      <h1>Поиск моделей</h1>
      <p>Найдите и добавьте модели из Hugging Face Hub в ваш менеджер моделей</p>
    </div>

    <ModelSearchFilters
      bind:searchQuery
      bind:selectedFormats
      {isLoading}
      on:search={handleSearch}
    />

    <!-- Результаты поиска -->
    {#if hasSearched}
      {#if isLoading}
        <div class="loading-state">
          <div class="loading-spinner"></div>
          <p>Поиск моделей...</p>
        </div>
      {:else if error}
        <div class="error-state">
          <p class="error-message">{error}</p>
          <button class="retry-btn" on:click={() => searchModels(searchQuery, selectedFormats)}>
            Попробовать снова
          </button>
        </div>
      {:else if models.length === 0}
        <div class="no-results">
          <p>По вашему запросу ничего не найдено.</p>
          <p>Попробуйте изменить поисковый запрос или фильтры.</p>
        </div>
      {:else}
        <div class="results-section">
          <div class="results-header">
            <h2>Найдено моделей: {models.length}</h2>
            {#if searchQuery || selectedFormats.length > 0}
              <button class="clear-search-btn" on:click={() => { searchQuery = ''; selectedFormats = []; searchModels('', []); }}>
                Очистить поиск
              </button>
            {/if}
          </div>
          
          <div class="models-grid">
            {#each models as model (model.id)}
              <ModelCard {model} on:click={() => addModelToManager(model.id)} />
            {/each}
          </div>
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
              <button class="suggestion-btn" on:click={() => { searchQuery = 'llama'; searchModels('llama', selectedFormats); }}>
                llama
              </button>
              <button class="suggestion-btn" on:click={() => { searchQuery = 'mistral'; searchModels('mistral', selectedFormats); }}>
                mistral
              </button>
              <button class="suggestion-btn" on:click={() => { searchQuery = 'gemma'; searchModels('gemma', selectedFormats); }}>
                gemma
              </button>
              <button class="suggestion-btn" on:click={() => { searchQuery = 'qwen'; searchModels('qwen', selectedFormats); }}>
                qwen
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</main>

<style>
  .search-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 16px;
  }

  .page-header {
    text-align: center;
    margin-bottom: 32px;
    padding: 32px 0;
  }

  .page-header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 16px 0;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .page-header p {
    font-size: 1.1rem;
    color: var(--muted);
    margin: 0;
    max-width: 600px;
    margin: 0 auto;
  }

  .loading-state {
    text-align: center;
    padding: 60px 20px;
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
    text-align: center;
    padding: 40px 20px;
    background: var(--panel-alt-bg);
    border-radius: 12px;
    border: 1px solid var(--border-color);
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
    text-align: center;
    padding: 60px 20px;
    color: var(--muted);
  }

  .no-results p {
    margin: 8px 0;
    font-size: 1.1rem;
  }

  .results-section {
    margin-top: 32px;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    flex-wrap: wrap;
    gap: 16px;
  }

  .results-header h2 {
    margin: 0;
    color: var(--text);
    font-size: 1.5rem;
  }

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

  .clear-search-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .models-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 24px;
  }

  .initial-state {
    padding: 40px 20px;
  }

  .welcome-card {
    background: var(--card);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    padding: 40px;
    text-align: center;
    max-width: 800px;
    margin: 0 auto;
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
  @media (max-width: 768px) {
    .search-page {
      padding: 0 12px;
    }

    .page-header h1 {
      font-size: 2rem;
    }

    .page-header p {
      font-size: 1rem;
    }

    .models-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .welcome-card {
      padding: 24px;
    }

    .search-suggestions {
      flex-direction: column;
      align-items: center;
    }

    .suggestion-btn {
      width: 200px;
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