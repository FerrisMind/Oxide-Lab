<script lang="ts">
  import DownloadSimple from "phosphor-svelte/lib/DownloadSimple";
  import UploadSimple from "phosphor-svelte/lib/UploadSimple";
  import CircleNotch from "phosphor-svelte/lib/CircleNotch";
  
  export let isLoadingModel = false;
  export let isUnloadingModel = false;
  export let isCancelling = false;
  export let isLoaded = false;
  export let busy = false;
  export let loadingStage = "";
  export let loadingProgress = 0;
  export let unloadingProgress = 0;
  export let repoId = "";
  export let hubGgufFilename = "";
  export let format: "gguf" | "hub_gguf" | "hub_safetensors" = "gguf";
  
  // Callback
  export let onMainAction: () => void;
</script>

<button 
  class="primary"
  on:click={onMainAction}
  disabled={busy || (!isLoaded && (
    (format === "gguf") || 
    (format === "hub_gguf" && (!repoId || !hubGgufFilename)) || 
    (format === "hub_safetensors" && !repoId)
  ))}
  style={(isLoadingModel || isUnloadingModel) ? `--progress-width: ${isLoadingModel ? loadingProgress : unloadingProgress}%` : ''}
  class:loading={isLoadingModel || isUnloadingModel}
  class:cancelling={isCancelling}
>
  {#if isLoadingModel}
    <div class="loading-button">
      {#if isCancelling}
        <CircleNotch size={18} class="spinning" />
        <span class="loading-text">Отмена...</span>
      {:else}
        <CircleNotch size={18} class="spinning" />
        <span class="loading-text">
          {#if loadingStage === "model"}
            {#if format === "hub_gguf"}
              Загрузка GGUF из HF Hub... {Math.round(loadingProgress)}%
            {:else if format === "hub_safetensors"}
              Загрузка из HF Hub... {Math.round(loadingProgress)}%
            {:else}
              Загрузка модели... {Math.round(loadingProgress)}%
            {/if}
          {:else if loadingStage === "tokenizer"}
            {#if format === "hub_safetensors"}
              Инициализация токенизатора... {Math.round(loadingProgress)}%
            {:else}
              Инициализация токенизатора из GGUF... {Math.round(loadingProgress)}%
            {/if}
          {:else if loadingStage === "complete"}
            Завершено! {Math.round(loadingProgress)}%
          {/if}
        </span>
      {/if}
    </div>
  {:else if isUnloadingModel}
    <div class="loading-button">
      <CircleNotch size={18} class="spinning" />
      <span class="loading-text">Выгрузка модели... {Math.round(unloadingProgress)}%</span>
    </div>
  {:else if isLoaded}
    <UploadSimple size={18} style="vertical-align: -3px;" /> Выгрузить
  {:else}
    <DownloadSimple size={18} style="vertical-align: -3px;" /> Загрузить
  {/if}
</button>