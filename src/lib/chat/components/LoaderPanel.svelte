<script lang="ts">
  import DownloadSimple from "phosphor-svelte/lib/DownloadSimple";
  import UploadSimple from "phosphor-svelte/lib/UploadSimple";
  import Stop from "phosphor-svelte/lib/Stop";
  import Package from "phosphor-svelte/lib/Package";
  import TextT from "phosphor-svelte/lib/TextT";
  import CheckCircle from "phosphor-svelte/lib/CheckCircle";
  import Lightbulb from "phosphor-svelte/lib/Lightbulb";
  // Binoculars removed: GGUF upload moved to header
  import CircleNotch from "phosphor-svelte/lib/CircleNotch";
  import Cpu from "phosphor-svelte/lib/Cpu";
  import GraphicsCard from "phosphor-svelte/lib/GraphicsCard";

  import { createEventDispatcher } from "svelte";
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
    <div class="param">
      <div class="row" style="align-items:center; gap: 12px; margin-bottom: 6px;">
        <label for="device-toggle">Устройство инференса</label>
        <div class="segmented-toggle" title={!cuda_build ? 'Сборка без CUDA' : ''}>
          <button
            type="button"
            class="segment" class:active={!use_gpu}
            aria-label="Процессор"
            on:click={() => dispatch('device-toggle', { checked: false })}
          >
            <Cpu size={18} />
            <span>ЦП</span>
          </button>
          <button
            type="button"
            class="segment" class:active={use_gpu} disabled={!cuda_build}
            aria-label="Графический процессор"
            on:click={() => dispatch('device-toggle', { checked: true })}
            title={!cuda_build ? 'Сборка без CUDA' : (!cuda_available ? 'Попытка переключить CUDA (может завершиться ошибкой)' : 'GPU (CUDA)')}
          >
            <GraphicsCard size={18} />
            <span>ГП</span>
          </button>
        </div>
      </div>
    </div>

    <!-- GGUF файл перемещён в шапку приложения -->
    <div class="param thinking-spacer">
      <div class="row">
        <input id="p-thinking" type="checkbox" bind:checked={enable_thinking} />
        <label for="p-thinking">Включить размышления</label>
      </div>
    </div>

    
    
    <div class="param">
      <label for="p-ctx">Длина контекста (токены)</label>
      <div class="row">
        <input id="p-ctx" type="range" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
        <input type="number" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
      </div>
    </div>
    <!-- primary load/unload button moved to header (GGUFUploadArea) -->

    {#if isLoadingModel}
      <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
        <div class="loading-stage">
          {#if isCancelling}
            <span class="stage-icon"><Stop size={16} weight="bold" /></span> Отмена загрузки...
          {:else if loadingStage === "model"}
            <span class="stage-icon"><Package size={16} weight="bold" /></span> Загрузка модели GGUF в память...
          {:else if loadingStage === "tokenizer"}
            <span class="stage-icon"><TextT size={16} weight="bold" /></span> Инициализация токенизатора из GGUF...
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
    {/if}
    {#if errorText}
      <div class="loading-status success">
        <div class="loading-stage">
          <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> {errorText}
        </div>
      </div>
    {/if}
  {:else if format === 'hub_gguf'}
    <div class="param">
      <div class="row" style="align-items:center; gap: 12px; margin-bottom: 6px;">
        <label for="device-toggle">Устройство инференса</label>
        <div class="segmented-toggle" title={!cuda_build ? 'Сборка без CUDA' : ''}>
          <button
            type="button"
            class="segment" class:active={!use_gpu}
            aria-label="Процессор"
            on:click={() => dispatch('device-toggle', { checked: false })}
          >
            <Cpu size={18} />
            <span>ЦП</span>
          </button>
          <button
            type="button"
            class="segment" class:active={use_gpu} disabled={!cuda_build}
            aria-label="Графический процессор"
            on:click={() => dispatch('device-toggle', { checked: true })}
            title={!cuda_build ? 'Сборка без CUDA' : (!cuda_available ? 'Попытка переключить CUDA (может завершиться ошибкой)' : 'GPU (CUDA)')}
          >
            <GraphicsCard size={18} />
            <span>ГП</span>
          </button>
        </div>
      </div>
    </div>

    <div class="field">
      <label for="hub-repo">HF Hub repoId</label>
      <input id="hub-repo" placeholder="owner/repo" bind:value={repoId} />
    </div>
    <div class="field">
      <label for="hub-rev">revision (опционально)</label>
      <input id="hub-rev" placeholder="main" bind:value={revision} />
    </div>
    <div class="field">
      <label for="hub-gguf">filename</label>
      <input id="hub-gguf" placeholder="model-Q4_K_M.gguf" bind:value={hubGgufFilename} />
    </div>

    <div class="param">
      <label for="p-ctx">Длина контекста (токены)</label>
      <div class="row">
        <input id="p-ctx" type="range" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
        <input type="number" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
      </div>
    </div>
    <button 
      class="primary"
      on:click={onMainAction}
      disabled={busy || (!isLoaded && (!repoId || !hubGgufFilename))}
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
                Загрузка модели... {Math.round(loadingProgress)}%
              {:else if loadingStage === "tokenizer"}
                Инициализация токенизатора из GGUF... {Math.round(loadingProgress)}%
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
    {#if isLoadingModel}
      <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
        <div class="loading-stage">
          {#if isCancelling}
            <span class="stage-icon"><Stop size={16} weight="bold" /></span> Отмена загрузки...
          {:else if loadingStage === "model"}
            <span class="stage-icon"><Package size={16} weight="bold" /></span> Загрузка GGUF из HF Hub...
          {:else if loadingStage === "tokenizer"}
            <span class="stage-icon"><TextT size={16} weight="bold" /></span> Инициализация токенизатора из GGUF...
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
    {/if}
  {:else}
    <div class="param">
      <div class="row" style="align-items:center; gap: 12px; margin-bottom: 6px;">
        <label for="device-toggle">Устройство инференса</label>
        <div class="segmented-toggle" title={!cuda_build ? 'Сборка без CUDA' : ''}>
          <button
            type="button"
            class="segment" class:active={!use_gpu}
            aria-label="Процессор"
            on:click={() => dispatch('device-toggle', { checked: false })}
          >
            <Cpu size={18} />
            <span>ЦП</span>
          </button>
          <button
            type="button"
            class="segment" class:active={use_gpu} disabled={!cuda_build}
            aria-label="Графический процессор"
            on:click={() => dispatch('device-toggle', { checked: true })}
            title={!cuda_build ? 'Сборка без CUDA' : (!cuda_available ? 'Попытка переключить CUDA (может завершиться ошибкой)' : 'GPU (CUDA)')}
          >
            <GraphicsCard size={18} />
            <span>ГП</span>
          </button>
        </div>
      </div>
    </div>
    <div class="field">
      <label for="hub-repo-st">HF Hub repoId</label>
      <input id="hub-repo-st" placeholder="owner/repo" bind:value={repoId} />
    </div>
    <div class="field">
      <label for="hub-rev-st">revision (опционально)</label>
      <input id="hub-rev-st" placeholder="main" bind:value={revision} />
    </div>
    <div class="param">
      <div class="hint">Скачиваются tokenizer.json и config.json; веса safetensors кэшируются. Инференс safetensors будет реализован в п.2 Roadmap.</div>
    </div>
    <div class="param">
      <label for="p-ctx">Длина контекста (токены)</label>
      <div class="row">
        <input id="p-ctx" type="range" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
        <input type="number" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
      </div>
    </div>
    <button 
      class="primary"
      on:click={onMainAction}
      disabled={busy || (!isLoaded && (!repoId))}
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
                Загрузка из HF Hub... {Math.round(loadingProgress)}%
              {:else if loadingStage === "tokenizer"}
                Инициализация токенизатора... {Math.round(loadingProgress)}%
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
    {#if isLoadingModel}
      <div class="loading-status" class:success={loadingStage === "complete"} class:cancelling={isCancelling}>
        <div class="loading-stage">
          {#if isCancelling}
            <span class="stage-icon"><Stop size={16} weight="bold" /></span> Отмена загрузки...
          {:else if loadingStage === "model"}
            <span class="stage-icon"><Package size={16} weight="bold" /></span> Кэширование файлов safetensors и метаданных...
          {:else if loadingStage === "tokenizer"}
            <span class="stage-icon"><TextT size={16} weight="bold" /></span> Инициализация токенизатора...
          {:else if loadingStage === "complete"}
            <span class="stage-icon"><CheckCircle size={16} weight="bold" /></span> Готово (веса подготовлены, инференс GGUF)
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
    {/if}
  {/if}
  <slot />
</section>


