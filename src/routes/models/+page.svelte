<script lang="ts">
  /**
   * Models Page
   *
   * Manage local and remote GGUF models.
   */
  import { onMount, onDestroy } from 'svelte';
  import * as Tabs from '$lib/components/ui/tabs';
  import HardDrive from 'phosphor-svelte/lib/HardDrive';
  import Globe from 'phosphor-svelte/lib/Globe';
  import Star from 'phosphor-svelte/lib/Star';
  import { t } from '$lib/i18n';
  import { folderPath, scanFolder } from '$lib/stores/local-models';
  import { setPageTabs, clearPageTabs, activePageTab } from '$lib/stores/page-tabs.svelte';
  import LocalModelsPanel from '$lib/components/model-manager/LocalModelsPanel.svelte';
  import RemoteModelsPanel from '$lib/components/model-manager/RemoteModelsPanel.svelte';
  import RecommendationsPanel from '$lib/components/model-manager/RecommendationsPanel.svelte';

  type TabId = 'local' | 'remote' | 'recommendations';

  let activeTab = $state<TabId>('local');

  const tabs = $derived([
    { id: 'local' as TabId, label: $t('models.tabs.local') || 'Local Models' },
    { id: 'remote' as TabId, label: $t('models.tabs.remote') || 'Remote Models' },
    {
      id: 'recommendations' as TabId,
      label: $t('models.tabs.recommendations') || 'Recommendations',
    },
  ]);

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
    activeTab = ($activePageTab || 'local') as TabId;
  });

  function handleTabChange(value: string) {
    activeTab = value as TabId;
    activePageTab.set(value);
  }
</script>

<div class="h-full overflow-hidden p-4 flex flex-col">
  <!-- Mobile tabs (shown only on small screens) -->
  <div class="sm:hidden mb-4">
    <Tabs.Root value={activeTab} onValueChange={handleTabChange}>
      <Tabs.List class="w-full">
        <Tabs.Trigger value="local" class="flex-1 gap-2">
          <HardDrive class="size-4" />
          {$t('models.tabs.local') || 'Local'}
        </Tabs.Trigger>
        <Tabs.Trigger value="remote" class="flex-1 gap-2">
          <Globe class="size-4" />
          {$t('models.tabs.remote') || 'Remote'}
        </Tabs.Trigger>
        <Tabs.Trigger value="recommendations" class="flex-1 gap-2">
          <Star class="size-4" />
          {$t('models.tabs.recommendations') || 'Rec'}
        </Tabs.Trigger>
      </Tabs.List>
    </Tabs.Root>
  </div>

  <!-- Main Content -->
  <div class="flex-1 min-h-0">
    {#if activeTab === 'local'}
      <LocalModelsPanel />
    {:else if activeTab === 'remote'}
      <RemoteModelsPanel />
    {:else if activeTab === 'recommendations'}
      <RecommendationsPanel />
    {/if}
  </div>
</div>
