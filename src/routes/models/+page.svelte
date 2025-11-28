<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import LocalModelsPanel from '$lib/components/model-manager/LocalModelsPanel.svelte';
  import RemoteModelsPanel from '$lib/components/model-manager/RemoteModelsPanel.svelte';
  import { folderPath, scanFolder } from '$lib/stores/local-models';
  import { setPageTabs, clearPageTabs, activePageTab } from '$lib/stores/page-tabs.svelte';

  type TabId = 'local' | 'remote';

  let activeTab = $state<TabId>('local');
  const tabs: { id: TabId; label: string }[] = [
    { id: 'local', label: 'Мои модели' },
    { id: 'remote', label: 'Поиск моделей' },
  ];

  onMount(() => {
    setPageTabs(tabs, 'local');
    if ($folderPath) {
      scanFolder($folderPath).catch((error) => console.error(error));
    }
  });

  onDestroy(() => {
    clearPageTabs();
  });

  $effect(() => {
    activeTab = $activePageTab as TabId;
  });
</script>

<section class="models-page">
  <div class="models-panel">
    {#if activeTab === 'local'}
      <LocalModelsPanel />
    {:else if activeTab === 'remote'}
      <RemoteModelsPanel />
    {/if}
  </div>
</section>

<style>
  .models-page {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    padding: 0 1rem 1rem;
    box-sizing: border-box;
    background: transparent;
  }

  .models-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
    min-height: 0;
    width: 100%;
    padding: 0;
    box-sizing: border-box;
  }

  :global(.tabs-content) {
    padding: 0;
    display: flex;
    width: 100%;
    background: transparent !important;
  }

  :global(.tabs-container) {
    background: transparent !important;
  }

  :global(.tabs-list) {
    background: transparent !important;
  }

  :global(.page-container.active > .models-page) {
    max-width: none;
    margin: 0;
    width: 100%;
  }
</style>
