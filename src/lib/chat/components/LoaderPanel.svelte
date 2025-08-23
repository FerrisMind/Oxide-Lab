<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import DeviceSelector from "./loader/DeviceSelector.svelte";
  import ContextLengthSelector from "./loader/ContextLengthSelector.svelte";
  import ThinkingModeToggle from "./loader/ThinkingModeToggle.svelte";
  import HubModelForm from "./loader/HubModelForm.svelte";
  import LoadButton from "./loader/LoadButton.svelte";
  import LoadingStatus from "./loader/LoadingStatus.svelte";
  
  const dispatch = createEventDispatcher();
  
  export let format: "gguf" | "hub_gguf" | "hub_safetensors" = "gguf";
  export let modelPath = "";
  export let repoId = "";
  export let revision = "";
  export let hubGgufFilename = "";
  // deprecated: токенизатор берётся из GGUF
  // export let tokenizerPath = "";
  export let enable_thinking = false;
  export let ctx_limit_value = 4096;
  // Убран offloading: настройка слоёв на GPU удалена
  export let isLoadingModel = false;
  export let isUnloadingModel = false;
  export let isCancelling = false;
  export let loadingStage = "";
  export let loadingProgress = 0;
  export let unloadingProgress = 0;
  export let errorText = "";
  export let busy = false;
  export let isLoaded = false;
  // Устройство инференса
  export let use_gpu = false; // CPU по умолчанию
  export let cuda_available = false;
  export let cuda_build = false;
  // current_device больше не используется (совпадает с выбранным сегментом)

  // Коллбеки, реализуются родителем
  export let onMainAction: () => void;

  // reference-only export (used by parent) — prevent Svelte warning by using const alias
  const _modelPath_ref = modelPath;
</script>

<section class="loader">
  <!-- format selection buttons removed (GGUF / HF Hub) — upload controls moved to header -->

  {#if format === 'gguf'}
    <DeviceSelector 
      bind:use_gpu 
      bind:cuda_available 
      bind:cuda_build 
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)} 
    />
    
    <ThinkingModeToggle bind:enable_thinking />
    
    <ContextLengthSelector bind:ctx_limit_value />
    
    <LoadButton 
      bind:isLoadingModel
      bind:isUnloadingModel
      bind:isCancelling
      bind:isLoaded
      bind:busy
      bind:loadingStage
      bind:loadingProgress
      bind:unloadingProgress
      bind:repoId
      bind:hubGgufFilename
      {format}
      {onMainAction}
    />
    
    <LoadingStatus 
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
      {format}
    />
  {:else if format === 'hub_gguf'}
    <DeviceSelector 
      bind:use_gpu 
      bind:cuda_available 
      bind:cuda_build 
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)} 
    />
    
    <HubModelForm 
      bind:repoId 
      bind:revision 
      bind:hubGgufFilename 
    />
    
    <ContextLengthSelector bind:ctx_limit_value />
    
    <LoadButton 
      bind:isLoadingModel
      bind:isUnloadingModel
      bind:isCancelling
      bind:isLoaded
      bind:busy
      bind:loadingStage
      bind:loadingProgress
      bind:unloadingProgress
      bind:repoId
      bind:hubGgufFilename
      {format}
      {onMainAction}
    />
    
    <LoadingStatus 
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
      {format}
    />
  {:else}
    <DeviceSelector 
      bind:use_gpu 
      bind:cuda_available 
      bind:cuda_build 
      on:device-toggle={(e) => dispatch('device-toggle', e.detail)} 
    />
    
    <HubModelForm 
      bind:repoId 
      bind:revision 
      isSafetensors={true}
    />
    
    <ContextLengthSelector bind:ctx_limit_value />
    
    <LoadButton 
      bind:isLoadingModel
      bind:isUnloadingModel
      bind:isCancelling
      bind:isLoaded
      bind:busy
      bind:loadingStage
      bind:loadingProgress
      bind:unloadingProgress
      bind:repoId
      bind:hubGgufFilename
      {format}
      {onMainAction}
    />
    
    <LoadingStatus 
      bind:isLoadingModel
      bind:isCancelling
      bind:loadingStage
      bind:loadingProgress
      bind:errorText
      {format}
    />
  {/if}
  <slot />
</section>