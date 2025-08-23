<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { createEventDispatcher } from 'svelte';
  import ModelItem from './model/ModelItem.svelte';
  import LoadingState from './model/LoadingState.svelte';
  import EmptyState from './model/EmptyState.svelte';

  export let models: HFModel[] = [];
  export let selectedModelId: string | null = null;
  export let loading = false;

  const dispatch = createEventDispatcher<{
    selectModel: { model: HFModel };
  }>();

  function handleModelSelect(event: CustomEvent<{ model: HFModel }>) {
    dispatch('selectModel', event.detail);
  }
</script>

<div class="model-list">
  <LoadingState {loading} />
  <EmptyState {models} {loading} />
  
  {#if !loading && models.length > 0}
    <div class="models-container">
      {#each models as model (model.id)}
        <ModelItem 
          {model} 
          {selectedModelId} 
          on:selectModel={handleModelSelect}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .model-list {
    height: 100%;
    overflow-y: auto;
    background: var(--card);
    border-right: 1px solid var(--border-color);
  }

  .models-container {
    padding: 8px;
  }

  /* Скроллбар */
  .model-list::-webkit-scrollbar {
    width: 12px;
  }

  .model-list::-webkit-scrollbar-track {
    background: transparent;
    border-radius: 6px;
  }

  .model-list::-webkit-scrollbar-thumb {
    background: rgba(179, 205, 224, 0.8);
    border-radius: 6px;
    border: 2px solid transparent;
    background-clip: content-box;
    transition: all 0.3s ease;
  }

  .model-list::-webkit-scrollbar-thumb:hover {
    background: rgba(179, 205, 224, 1);
  }
</style>