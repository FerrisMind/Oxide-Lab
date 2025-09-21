<script lang="ts">
  import Stop from "phosphor-svelte/lib/Stop";
  import Package from "phosphor-svelte/lib/Package";
  import _TextT from "phosphor-svelte/lib/TextT";
  import CheckCircle from "phosphor-svelte/lib/CheckCircle";
  import Lightbulb from "phosphor-svelte/lib/Lightbulb";
  
  export let isLoadingModel = false;
  export let isCancelling = false;
  export let loadingStage = "";
  export let loadingProgress = 0;
  export let errorText = "";

  function stageText(stage: string): string {
    switch (stage) {
      case 'start': return 'Инициализация загрузки...';
      case 'device': return 'Выбор устройства...';
      case 'open_file': return 'Открытие файла модели...';
      case 'read_header': return 'Чтение метаданных GGUF...';
      case 'tokenizer': return 'Инициализация токенизатора...';
      case 'detect_arch': return 'Определение архитектуры...';
      case 'build_model': return 'Создание модели...';
      case 'build_model_done': return 'Модель создана';
      case 'hub_get': return 'Загрузка из HF Hub...';
      case 'hub_list': return 'Чтение списка весов...';
      case 'hub_cache': return 'Кэширование весов...';
      case 'scan_weights': return 'Проверка весов...';
      case 'config': return 'Чтение config.json...';
      case 'finalize': return 'Завершение...';
      case 'complete': return 'Готово';
      case 'model': return 'Загрузка модели...';
      case 'cancelling': return 'Отмена загрузки...';
      case 'cancel': return 'Загрузка отменена';
      case 'error': return 'Ошибка при загрузке';
      case 'unload_start': return 'Начало выгрузки...';
      case 'unload_model': return 'Освобождение модели...';
      case 'unload_tokenizer': return 'Освобождение токенизатора...';
      case 'unload_complete': return 'Выгружено';
      default:
        return stage ? `Этап: ${stage}` : '';
    }
  }
</script>

{#if isLoadingModel}
  <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
    <div class="loading-stage">
      {#if isCancelling}
        <span class="stage-icon"><Stop size={16} weight="bold" /></span> Отмена загрузки...
      {:else if loadingStage === "complete"}
        <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> Модель и токенизатор готовы к работе!
      {:else}
        <span class="stage-icon"><Package size={16} weight="bold" /></span> {stageText(loadingStage)}
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
