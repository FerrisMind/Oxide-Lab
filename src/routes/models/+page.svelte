<script lang="ts">
  import Tabs from '$lib/components/ui/Tabs.svelte';
  import LocalModelsTab from '$lib/components/model-manager/LocalModelsTab.svelte';
  import HuggingFaceTab from '$lib/components/model-manager/HuggingFaceTab.svelte';

  // Active tab state
  let activeTab = $state('local');

  // Tab definitions
  const tabs = [
    { id: 'local', label: 'Локальные модели' },
    { id: 'huggingface', label: 'Hugging Face' },
  ];
</script>

<div class="models-page">
  <div class="page-header">
    <h1>Управление моделями</h1>
    <p class="page-description">
      Управляйте локальными моделями или выберите модель из Hugging Face Hub
    </p>
  </div>

  <div class="tabs-wrapper">
    <Tabs {tabs} bind:activeTab>
      {#if activeTab === 'local'}
        <div class="tab-panel" role="tabpanel" id="tabpanel-local" aria-labelledby="tab-local">
          <LocalModelsTab />
        </div>
      {:else if activeTab === 'huggingface'}
        <div
          class="tab-panel"
          role="tabpanel"
          id="tabpanel-huggingface"
          aria-labelledby="tab-huggingface"
        >
          <HuggingFaceTab />
        </div>
      {/if}
    </Tabs>
  </div>
</div>

<style>
  .models-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    height: 100vh;
    background: var(--bg);
    overflow: hidden;
  }

  .page-header {
    flex-shrink: 0;
    padding: 2rem 2rem 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .page-header h1 {
    margin: 0 0 0.5rem 0;
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
  }

  .page-description {
    margin: 0;
    font-size: 1rem;
    color: var(--muted);
  }

  .tabs-wrapper {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .tab-panel {
    flex: 1;
    overflow: hidden;
  }

  /* Адаптивность для мобильных */
  @media (max-width: 768px) {
    .page-header {
      padding: 1.5rem 1rem 0.75rem;
    }

    .page-header h1 {
      font-size: 1.5rem;
    }

    .page-description {
      font-size: 0.875rem;
    }
  }
</style>
