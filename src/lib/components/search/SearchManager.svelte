<script lang="ts">
  import { onMount } from 'svelte';
  import { headerSearchQuery, searchTrigger } from '$lib/stores/search';
  import { huggingFaceService } from '$lib/services/huggingface';
  import type { HFModel, SearchResult } from '$lib/services/huggingface';
  import { rightSidebarOpen } from '$lib/stores/sidebar';

  export let searchQuery = '';
  export let selectedFormats: string[] = [];
  export let selectedPipelineTags: string[] = [];
  export let selectedLibraries: string[] = [];
  export let selectedLanguages: string[] = [];
  export let selectedLicenses: string[] = [];
  export let authorFilter = '';
  
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

  // Функция поиска моделей
  export async function searchModels({
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

  export async function loadMoreModels() {
    if (!hasMore || isLoadingMore) return;
    await searchModels({ loadMore: true });
  }

  // Добавление модели в менеджер
  export function addModelToManager(modelId: string) {
    // Здесь будет логика добавления модели в менеджер
    console.log('Adding model to manager:', modelId);
    // Можно открыть правую панель с настройками для этой модели
    rightSidebarOpen.set(true);
  }

  // Обработка выбора модели
  export function handleModelSelect(event: CustomEvent<{ model: HFModel }>) {
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

<slot 
  {searchQuery}
  {selectedFormats}
  {selectedPipelineTags}
  {selectedLibraries}
  {selectedLanguages}
  {selectedLicenses}
  {authorFilter}
  {isLoading}
  {isLoadingMore}
  {models}
  {error}
  {hasSearched}
  {hasMore}
  {nextOffset}
  {totalCount}
  {selectedModel}
  {modelDetailLoading}
  {searchModels}
  {loadMoreModels}
  {addModelToManager}
  {handleModelSelect}
/>