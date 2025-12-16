<script lang="ts">
  import Stop from "phosphor-svelte/lib/Stop";
  import CheckCircle from "phosphor-svelte/lib/CheckCircle";
  import Lightbulb from "phosphor-svelte/lib/Lightbulb";
  import { Spinner } from '$lib/components/ui/spinner';
  import { t } from '$lib/i18n';
  
  interface Props {
    isLoadingModel?: boolean;
    isCancelling?: boolean;
    loadingStage?: string;
    loadingProgress?: number;
    errorText?: string;
  }

  let {
    isLoadingModel = $bindable(false),
    isCancelling = $bindable(false),
    loadingStage = $bindable(""),
    loadingProgress = $bindable(0),
    errorText = $bindable("")
  }: Props = $props();

  let stageTextValue = $derived((stage: string) => {
    const key = `chat.loading.stages.${stage}`;
    const translated = $t(key);
    // Если перевод не найден (вернулся ключ), используем дефолтный
    if (translated === key) {
      return stage ? $t('chat.loading.stages.default', { stage }) : '';
    }
    return translated;
  });
  
  function stageText(stage: string): string {
    return stageTextValue(stage);
  }
</script>

{#if isLoadingModel}
  <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
    <div class="loading-stage">
      {#if isCancelling}
        <span class="stage-icon"><Stop size={16} weight="bold" /></span> {$t('chat.loading.cancelling')}
      {:else if loadingStage === "complete"}
        <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> {$t('chat.loading.complete')}
      {:else}
        <span class="stage-icon"><Spinner size={16} testId="model-loading-spinner" /></span>
        {stageText(loadingStage)}
      {/if}
    </div>
    {#if !isCancelling}
      <div class="loading-progress-bar">
        <div class="progress-fill" style="width: {loadingProgress}%"></div>
      </div>
      <div class="loading-hint">
        <span class="hint-icon"><Lightbulb size={14} weight="duotone" /></span> {$t('chat.loading.cancelHint')}
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

<style>
  /* Icon alignment using flexbox instead of hardcoded pixels */
  .hint-icon {
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
  }
</style>
