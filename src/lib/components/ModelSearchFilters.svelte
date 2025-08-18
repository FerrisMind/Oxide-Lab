<script lang="ts">
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import Funnel from 'phosphor-svelte/lib/Funnel';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import CaretUp from 'phosphor-svelte/lib/CaretUp';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let filtersCollapsed = true;

  export let searchQuery = '';
  export let selectedFormats: string[] = [];
  export let selectedPipelineTags: string[] = [];
  export let selectedLibraries: string[] = [];
  export let selectedLanguages: string[] = [];
  export let selectedLicenses: string[] = [];
  export let authorFilter = '';
  export let isLoading = false;
  export let clearSearch: () => void;

  const availableFormats = [
    { id: 'gguf', label: 'GGUF', color: '#10b981' },
    { id: 'safetensors', label: 'Safetensors', color: '#3b82f6' }
  ];

  const availablePipelineTags = [
    { id: 'text-generation', label: 'Text Generation' },
    { id: 'text2text-generation', label: 'Text-to-Text' },
    { id: 'conversational', label: 'Conversational' },
    { id: 'question-answering', label: 'Q&A' },
    { id: 'summarization', label: 'Summarization' },
    { id: 'translation', label: 'Translation' },
    { id: 'text-classification', label: 'Classification' },
    { id: 'feature-extraction', label: 'Embeddings' }
  ];

  const availableLibraries = [
    { id: 'transformers', label: 'Transformers' },
    { id: 'pytorch', label: 'PyTorch' },
    { id: 'tensorflow', label: 'TensorFlow' },
    { id: 'jax', label: 'JAX' },
    { id: 'onnx', label: 'ONNX' },
    { id: 'safetensors', label: 'SafeTensors' }
  ];

  const availableLanguages = [
    { id: 'en', label: 'English' },
    { id: 'ru', label: 'Russian' },
    { id: 'zh', label: 'Chinese' },
    { id: 'es', label: 'Spanish' },
    { id: 'fr', label: 'French' },
    { id: 'de', label: 'German' },
    { id: 'ja', label: 'Japanese' },
    { id: 'ko', label: 'Korean' }
  ];

  const availableLicenses = [
    { id: 'apache-2.0', label: 'Apache 2.0' },
    { id: 'mit', label: 'MIT' },
    { id: 'cc-by-4.0', label: 'CC BY 4.0' },
    { id: 'cc-by-sa-4.0', label: 'CC BY-SA 4.0' },
    { id: 'gpl-3.0', label: 'GPL 3.0' },
    { id: 'other', label: 'Other' }
  ];

  function handleSearch() {
    dispatch('search', { 
      query: searchQuery, 
      formats: selectedFormats,
      pipelineTags: selectedPipelineTags,
      libraries: selectedLibraries,
      languages: selectedLanguages,
      licenses: selectedLicenses,
      author: authorFilter
    });
  }

  function toggleFormat(format: string) {
    if (selectedFormats.includes(format)) {
      selectedFormats = selectedFormats.filter(f => f !== format);
    } else {
      selectedFormats = [...selectedFormats, format];
    }
    handleSearch();
  }

  function clearFilters() {
    selectedFormats = [];
    selectedPipelineTags = [];
    selectedLibraries = [];
    selectedLanguages = [];
    selectedLicenses = [];
    searchQuery = '';
    authorFilter = '';
    handleSearch();
  }

  function togglePipelineTag(tag: string) {
    if (selectedPipelineTags.includes(tag)) {
      selectedPipelineTags = selectedPipelineTags.filter(t => t !== tag);
    } else {
      selectedPipelineTags = [...selectedPipelineTags, tag];
    }
    handleSearch();
  }

  function toggleLibrary(library: string) {
    if (selectedLibraries.includes(library)) {
      selectedLibraries = selectedLibraries.filter(l => l !== library);
    } else {
      selectedLibraries = [...selectedLibraries, library];
    }
    handleSearch();
  }

  function toggleLanguage(language: string) {
    if (selectedLanguages.includes(language)) {
      selectedLanguages = selectedLanguages.filter(l => l !== language);
    } else {
      selectedLanguages = [...selectedLanguages, language];
    }
    handleSearch();
  }

  function toggleLicense(license: string) {
    if (selectedLicenses.includes(license)) {
      selectedLicenses = selectedLicenses.filter(l => l !== license);
    } else {
      selectedLicenses = [...selectedLicenses, license];
    }
    handleSearch();
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="search-filters">
  <div class="search-section">
    <div class="search-input-wrapper">
      <MagnifyingGlass size={20} class="search-icon" />
      <input
        type="text"
        placeholder="Поиск моделей (например: llama, mistral, gemma)..."
        bind:value={searchQuery}
        on:keypress={handleKeyPress}
        class="search-input"
        disabled={isLoading}
      />
      <button 
        class="search-btn" 
        on:click={handleSearch}
        disabled={isLoading}
      >
        {isLoading ? 'Поиск...' : 'Найти'}
      </button>
    </div>
  </div>

  <div class="filters-section">
    <div class="filters-header">
      <div class="filters-title">
        <Funnel size={16} />
        <span>Фильтры</span>
      </div>
      <button class="collapse-btn" on:click={() => filtersCollapsed = !filtersCollapsed}>
        {#if filtersCollapsed}
          <CaretDown size={16} />
        {:else}
          <CaretUp size={16} />
        {/if}
      </button>
    </div>
    
    {#if !filtersCollapsed}
      <div class="filters-content">
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

        <div class="author-filter">
          <span class="filter-label">Автор:</span>
          <input
            type="text"
            placeholder="Например: microsoft, google, meta-llama..."
            bind:value={authorFilter}
            on:keypress={handleKeyPress}
            class="author-input"
            disabled={isLoading}
          />
        </div>

        <div class="pipeline-filters">
          <span class="filter-label">Тип задачи:</span>
          <div class="filter-buttons">
            {#each availablePipelineTags as tag}
              <button
                class="filter-btn {selectedPipelineTags.includes(tag.id) ? 'active' : ''}"
                on:click={() => togglePipelineTag(tag.id)}
                disabled={isLoading}
              >
                {tag.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="library-filters">
          <span class="filter-label">Библиотека:</span>
          <div class="filter-buttons">
            {#each availableLibraries as library}
              <button
                class="filter-btn {selectedLibraries.includes(library.id) ? 'active' : ''}"
                on:click={() => toggleLibrary(library.id)}
                disabled={isLoading}
              >
                {library.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="language-filters">
          <span class="filter-label">Язык:</span>
          <div class="filter-buttons">
            {#each availableLanguages as language}
              <button
                class="filter-btn {selectedLanguages.includes(language.id) ? 'active' : ''}"
                on:click={() => toggleLanguage(language.id)}
                disabled={isLoading}
              >
                {language.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="license-filters">
          <span class="filter-label">Лицензия:</span>
          <div class="filter-buttons">
            {#each availableLicenses as license}
              <button
                class="filter-btn {selectedLicenses.includes(license.id) ? 'active' : ''}"
                on:click={() => toggleLicense(license.id)}
                disabled={isLoading}
              >
                {license.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="filter-actions">
          <button 
            class="clear-filters-btn" 
            on:click={clearFilters}
            disabled={isLoading || (selectedFormats.length === 0 && selectedPipelineTags.length === 0 && selectedLibraries.length === 0 && selectedLanguages.length === 0 && selectedLicenses.length === 0 && !searchQuery && !authorFilter)}
          >
            Очистить фильтры
          </button>
          
          <button 
            class="clear-search-btn" 
            on:click={clearSearch}
            disabled={isLoading || (selectedFormats.length === 0 && selectedPipelineTags.length === 0 && selectedLibraries.length === 0 && selectedLanguages.length === 0 && selectedLicenses.length === 0 && !searchQuery && !authorFilter)}
          >
            Очистить поиск
          </button>
        </div>
      </div>
    {/if}
  </div>

  {#if selectedFormats.length > 0 || selectedPipelineTags.length > 0 || selectedLibraries.length > 0 || selectedLanguages.length > 0 || selectedLicenses.length > 0 || searchQuery || authorFilter}
    <div class="active-filters">
      <span class="active-filters-label">Активные фильтры:</span>
      {#if searchQuery}
        <span class="active-filter">
          Поиск: "{searchQuery}"
          <button class="remove-filter" on:click={() => { searchQuery = ''; handleSearch(); }}>
            ×
          </button>
        </span>
      {/if}
      {#if authorFilter}
        <span class="active-filter">
          Автор: "{authorFilter}"
          <button class="remove-filter" on:click={() => { authorFilter = ''; handleSearch(); }}>
            ×
          </button>
        </span>
      {/if}
      {#each selectedFormats as format}
        <span class="active-filter">
          Формат: {availableFormats.find(f => f.id === format)?.label}
          <button class="remove-filter" on:click={() => toggleFormat(format)}>
            ×
          </button>
        </span>
      {/each}
      {#each selectedPipelineTags as tag}
        <span class="active-filter">
          Задача: {availablePipelineTags.find(t => t.id === tag)?.label}
          <button class="remove-filter" on:click={() => togglePipelineTag(tag)}>
            ×
          </button>
        </span>
      {/each}
      {#each selectedLibraries as library}
        <span class="active-filter">
          Библиотека: {availableLibraries.find(l => l.id === library)?.label}
          <button class="remove-filter" on:click={() => toggleLibrary(library)}>
            ×
          </button>
        </span>
      {/each}
      {#each selectedLanguages as language}
        <span class="active-filter">
          Язык: {availableLanguages.find(l => l.id === language)?.label}
          <button class="remove-filter" on:click={() => toggleLanguage(language)}>
            ×
          </button>
        </span>
      {/each}
      {#each selectedLicenses as license}
        <span class="active-filter">
          Лицензия: {availableLicenses.find(l => l.id === license)?.label}
          <button class="remove-filter" on:click={() => toggleLicense(license)}>
            ×
          </button>
        </span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-filters {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 20px;
    margin-bottom: 0;
    width: 100%;
    max-width: none;
    box-sizing: border-box;
  }

  .search-section {
    margin-bottom: 20px;
    width: 100%;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: 12px;
    position: relative;
  }

  /* keep icon visually present but avoid unused selector warning */
  .search-input-wrapper :global(svg) {
    color: var(--muted);
    position: absolute;
    left: 16px;
    z-index: 1;
    pointer-events: none;
  }

  .search-input {
    flex: 1;
    padding: 12px 16px 12px 48px;
    border: 2px solid var(--border-color);
    border-radius: 10px;
    font-size: 16px;
    background: var(--bg);
    color: var(--text);
    outline: none;
    transition: all 0.2s ease;
  }

  .search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(179, 205, 224, 0.1);
  }

  .search-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .search-btn {
    background: var(--accent-2);
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 10px;
    font-size: 16px;
    font-weight: 600;
     cursor: default;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .search-btn:hover:not(:disabled) {
    background: var(--accent);
    transform: translateY(-1px);
  }

  .search-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .filters-section {
    border-top: 1px solid var(--border);
    padding-top: 20px;
    width: 100%;
  }

  .filters-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .filters-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .collapse-btn:hover {
    background: var(--panel-alt-bg);
    color: var(--text);
  }

  .filters-content {
    animation: slideDown 0.3s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .format-filters,
  .pipeline-filters,
  .library-filters,
  .language-filters,
  .license-filters {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

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

  .format-buttons,
  .filter-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
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
    cursor: not-allowed;
  }

  .format-btn,
  .filter-btn {
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

  .filter-btn:hover:not(:disabled) {
    border-color: var(--accent);
    background: rgba(179, 205, 224, 0.1);
  }

  .filter-btn.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .format-btn:disabled,
  .filter-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .filter-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .clear-filters-btn,
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

  .clear-filters-btn:hover:not(:disabled),
  .clear-search-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .clear-search-btn {
    background: var(--accent-2);
    color: white;
    border-color: var(--accent-2);
  }

  .clear-search-btn:hover:not(:disabled) {
    background: var(--accent);
    border-color: var(--accent);
  }

  .clear-filters-btn:disabled,
  .clear-search-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .active-filters {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-top: 20px;
    padding-top: 20px;
    border-top: 1px solid var(--border-color);
  }

  .active-filters-label {
    font-size: 14px;
    color: var(--muted);
    font-weight: 500;
  }

  .active-filter {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--accent);
    color: white;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
  }

  .remove-filter {
    background: none;
    border: none;
    color: white;
    font-size: 16px;
    font-weight: bold;
     cursor: default;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: background 0.2s ease;
  }

  .remove-filter:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  /* Адаптивность */
  @media (max-width: 1024px) {
    .search-filters {
      padding: 16px;
    }

    .format-filters,
    .pipeline-filters,
    .library-filters,
    .language-filters,
    .license-filters,
    .author-filter {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .format-buttons,
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
    .search-filters {
      padding: 12px;
    }

    .search-input-wrapper {
      flex-direction: column;
      gap: 12px;
    }

    .search-input {
      width: 100%;
      font-size: 16px; /* Предотвращает зум на iOS */
    }

    .format-filters,
    .pipeline-filters,
    .library-filters,
    .language-filters,
    .license-filters,
    .author-filter {
      flex-direction: column;
      align-items: flex-start;
      gap: 10px;
      margin-bottom: 16px;
    }

    .format-buttons,
    .filter-buttons {
      width: 100%;
      justify-content: flex-start;
      gap: 6px;
    }

    .author-input {
      width: 100%;
      min-width: unset;
      font-size: 16px; /* Предотвращает зум на iOS */
    }

    .filter-label {
      min-width: unset;
      font-size: 14px;
      font-weight: 600;
    }

    .format-btn,
    .filter-btn {
      font-size: 13px;
      padding: 6px 12px;
    }

    .search-btn {
      width: 100%;
      font-size: 15px;
      padding: 12px 20px;
    }
  }

  @media (max-width: 480px) {
    .search-filters {
      padding: 8px;
    }

    .search-input {
      padding: 10px 12px;
    }

    .format-filters,
    .pipeline-filters,
    .library-filters,
    .language-filters,
    .license-filters,
    .author-filter {
      margin-bottom: 12px;
      gap: 8px;
    }

    .format-buttons,
    .filter-buttons {
      gap: 4px;
    }

    .format-btn,
    .filter-btn {
      font-size: 12px;
      padding: 5px 10px;
    }

    .author-input {
      padding: 8px 10px;
    }

    .search-btn {
      padding: 10px 16px;
      font-size: 14px;
    }
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .search-input {
      background: var(--card);
      border-color: var(--border-color);
    }

    .format-btn {
      border-color: var(--border-color);
    }
  }
</style>
