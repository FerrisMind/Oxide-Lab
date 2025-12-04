<script lang="ts">

  import type { HFModel } from '$lib/services/huggingface';
  import { huggingFaceService } from '$lib/services/huggingface';
  import EmptyState from './model-detail/EmptyState.svelte';
  import LoadingState from './model-detail/LoadingState.svelte';
  import ModelHeader from './model-detail/ModelHeader.svelte';
  import ModelStats from './model-detail/ModelStats.svelte';
  import ModelTags from './model-detail/ModelTags.svelte';
  import ModelFormats from './model-detail/ModelFormats.svelte';
  import ModelAdditionalInfo from './model-detail/ModelAdditionalInfo.svelte';
  import ModelDescription from './model-detail/ModelDescription.svelte';

  interface Props {
    model?: HFModel | null;
    loading?: boolean;
  }

  let { model = null, loading = false }: Props = $props();

  let detailedModel: HFModel | null = $state(null);
  let detailsLoading = $state(false);
  let currentModelId: string | null = null;

  
  function handleModelChange(newModel: HFModel | null) {
    const newModelId = newModel?.id || null;
    
    if (newModelId !== currentModelId) {
      // Model has changed, cleanup previous state
      if (currentModelId !== null) {
        detailedModel = null;
      }
      
      currentModelId = newModelId;
      
      if (newModel) {
        loadModelDetails();
      }
    }
  }

  async function loadModelDetails() {
    if (!model) return;
    
    const currentModelId = model.id;
    detailsLoading = true;
    
    try {
      const details = await huggingFaceService.getModelDetails(currentModelId);
      
      // Check if the model hasn't changed while we were loading
      if (model?.id === currentModelId && details) {
        detailedModel = details;
      }
    } catch (error) {
      console.error('Ошибка загрузки деталей модели:', error);
      // Only set detailedModel to null if we're still on the same model
      if (model?.id === currentModelId) {
        detailedModel = null;
      }
    } finally {
      // Only update loading state if we're still on the same model
      if (model?.id === currentModelId) {
        detailsLoading = false;
      }
    }
  }
  $effect(() => {
    handleModelChange(model);
  });
</script>

<div class="model-detail">
  {#if loading}
    <LoadingState />
  {:else if !model}
    <EmptyState />
  {:else}
    <div class="detail-content">
      <ModelHeader {model} />
      <ModelStats {model} />
      <ModelTags {model} />
      <ModelFormats {model} />
      <ModelAdditionalInfo {detailedModel} />
      <ModelDescription {model} {detailedModel} {detailsLoading} />
    </div>
  {/if}
</div>

<style>
  .model-detail {
    height: 100%;
    overflow-y: auto;
    background: var(--card);
    border: 1px solid var(--border-color);
  }

  .detail-content {
    padding: 24px;
  }
</style>
