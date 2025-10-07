<script lang="ts">
  import { onMount } from 'svelte';
  import Tabs from '$lib/components/ui/Tabs.svelte';
  import LocalModelsPanel from '$lib/components/model-manager/LocalModelsPanel.svelte';
  import RemoteModelsPanel from '$lib/components/model-manager/RemoteModelsPanel.svelte';
  import { folderPath, scanFolder } from '$lib/stores/local-models';

  type TabId = 'local' | 'remote';

  let activeTab: TabId = 'local';
  const tabs = [
    { id: 'local', label: 'Мои модели' },
    { id: 'remote', label: 'Поиск моделей' },
  ];

  onMount(() => {
    if ($folderPath) {
      scanFolder($folderPath).catch((error) => console.error(error));
    }
  });
</script>

<section class="models-page">
  <Tabs bind:activeTab={activeTab} {tabs}>
    <div class="models-panel">
      {#if activeTab === 'local'}
        <LocalModelsPanel />
      {:else if activeTab === 'remote'}
        <RemoteModelsPanel />
      {/if}
    </div>
  </Tabs>
</section>

<style>
  .models-page {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    padding: 0;
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
    padding: 1rem;
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
