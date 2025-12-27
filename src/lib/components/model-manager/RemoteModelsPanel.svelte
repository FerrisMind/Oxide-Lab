<script lang="ts">
  /**
   * Remote Models Panel
   *
   * Search and download models directly from HuggingFace.
   */
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import MagnifyingGlass from 'phosphor-svelte/lib/MagnifyingGlass';
  import Cube from 'phosphor-svelte/lib/Cube';
  import { t } from '$lib/i18n';

  let searchQuery = $state('');
  let isSearching = $state(false);

  async function handleSearch() {
    if (!searchQuery.trim()) return;
    isSearching = true;
    // TODO: Implement HuggingFace API search
    await new Promise((resolve) => setTimeout(resolve, 1000));
    isSearching = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="h-full flex flex-col gap-4">
  <!-- Search Bar -->
  <div class="flex items-center gap-3 p-3 rounded-lg border bg-card">
    <div class="flex-1 relative">
      <MagnifyingGlass
        class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground"
      />
      <Input
        type="search"
        placeholder={$t('models.remote.searchPlaceholderFull') ||
          'Search models on Hugging Face...'}
        bind:value={searchQuery}
        class="pl-10"
        onkeydown={handleKeydown}
      />
    </div>
    <Button variant="default" onclick={handleSearch} disabled={isSearching || !searchQuery.trim()}>
      {#if isSearching}
        <Spinner class="size-4 mr-2" />
        {$t('models.remote.searching') || 'Searching...'}
      {:else}
        <MagnifyingGlass class="size-4 mr-2" />
        {$t('models.remote.search') || 'Search'}
      {/if}
    </Button>
  </div>

  <!-- Results / Placeholder -->
  <div class="flex-1 flex items-center justify-center">
    <div class="text-center space-y-4">
      <Cube class="size-16 mx-auto text-muted-foreground/50" weight="light" />
      <div class="space-y-2">
        <p class="text-muted-foreground">
          {$t('models.remote.searchHint') || 'Search for models on Hugging Face'}
        </p>
        <p class="text-sm text-muted-foreground/70">
          {$t('models.remote.searchDescription') || 'Enter a model name or keyword to search'}
        </p>
      </div>
    </div>
  </div>
</div>
