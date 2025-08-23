<script lang="ts">
  import Stop from "phosphor-svelte/lib/Stop";
  import Package from "phosphor-svelte/lib/Package";
  import TextT from "phosphor-svelte/lib/TextT";
  import CheckCircle from "phosphor-svelte/lib/CheckCircle";
  import Lightbulb from "phosphor-svelte/lib/Lightbulb";
  
  export let isLoadingModel = false;
  export let isCancelling = false;
  export let loadingStage = "";
  export let loadingProgress = 0;
  export let errorText = "";
  export let format: "gguf" | "hub_gguf" | "hub_safetensors" = "gguf";
</script>

{#if isLoadingModel}
  <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
    <div class="loading-stage">
      {#if isCancelling}
        <span class="stage-icon"><Stop size={16} weight="bold" /></span> Отмена загрузки...
      {:else if loadingStage === "model"}
        <span class="stage-icon"><Package size={16} weight="bold" /></span> 
        {#if format === "hub_gguf"}
          Загрузка GGUF из HF Hub...
        {:else if format === "hub_safetensors"}
          Кэширование файлов safetensors и метаданных...
        {:else}
          Загрузка модели GGUF в память...
        {/if}
      {:else if loadingStage === "tokenizer"}
        <span class="stage-icon"><TextT size={16} weight="bold" /></span> 
        {#if format === "hub_safetensors"}
          Инициализация токенизатора...
        {:else}
          Инициализация токенизатора из GGUF...
        {/if}
      {:else if loadingStage === "complete"}
        <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> Модель и токенизатор готовы к работе!
      {/if}
    </div>
    {#if !isCancelling}
      <div class="loading-progress-bar">
        <div class="progress-fill" style="width: {loadingProgress}%"></div>
      </div>
      <div class="loading-hint">
        <Lightbulb size={14} weight="duotone" style="vertical-align: -2px;" /> Нажмите кнопку еще раз для отмены загрузки
      </div>
    {/if}
  </div>
{:else if errorText}
  <div class="loading-status success">
    <div class="loading-stage">
      <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> {errorText}
    </div>
  </div>
{/if}