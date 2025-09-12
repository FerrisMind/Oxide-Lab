<script lang="ts">
  // no direct event subscriptions in Chat; streaming handled via controller
  import { onDestroy, onMount } from "svelte";
  import Composer from "$lib/chat/components/Composer.svelte";
  import LoaderPanel from "$lib/chat/components/LoaderPanel.svelte";
  import MessageList from "$lib/chat/components/MessageList.svelte";
  import ChatPlaceholder from "$lib/chat/components/ChatPlaceholder.svelte";
  // Chat styles are loaded globally from layout to avoid UI changes when navigating between pages
  // Убрали переключатель «сырого» Markdown
  import type { ChatMessage } from "$lib/chat/types";
  import { createChatController } from "$lib/chat/controller";
  import InferenceParams from "$lib/chat/components/InferenceParams.svelte";
  import { chatState, chatUiMounted, getDefaultChatState } from "$lib/stores/chat";
  import { get as getStore } from "svelte/store";

  let modelPath = "";
  let repoId: string = "";
  let revision: string = "";
  let hubGgufFilename: string = "";
  let prompt = "";
  let messages: ChatMessage[] = [];
  let messagesEl: HTMLDivElement | null = null;
  let busy = false;
  let format: "gguf" | "hub_gguf" | "hub_safetensors" = "gguf";
  let isLoaded = false;
  let errorText = "";
  // Устройство
  let use_gpu: boolean = false; // CPU по умолчанию
  let cuda_available: boolean = false;
  let cuda_build: boolean = false;
  let current_device: string = "CPU";
  
  // Состояние загрузки модели
  let isLoadingModel = false;
  let loadingProgress = 0;
  let loadingStage = ""; // "model" | "tokenizer" | "complete"
  let isCancelling = false; // Флаг для отмены загрузки
  
  // Состояние выгрузки модели
  let isUnloadingModel = false;
  let unloadingProgress = 0;

  // Параметры инференса
  let temperature: number = 0.8;
  let temperature_enabled = false;
  let top_k_enabled = false; let top_k_value: number = 40;
  let top_p_enabled = false; let top_p_value: number = 0.9;
  let min_p_enabled = false; let min_p_value: number = 0.05;
  let repeat_penalty_enabled = false; let repeat_penalty_value: number = 1.1;
  // Длина контекста (всегда активна и передаётся при загрузке)
  let ctx_limit_value: number = 4096;
  // Управление размышлениями (enable_thinking)
  // removed: enable_thinking (no_think detection removed)
  // Режим использования пользовательских параметров
  let use_custom_params: boolean = false;
  // Убран offloading: слои на GPU больше не настраиваются

  const controller = createChatController({
    get modelPath() { return modelPath; }, set modelPath(v) { modelPath = v; },
    get format() { return format; }, set format(v) { format = v; },
    get repoId() { return repoId; }, set repoId(v) { repoId = v; },
    get revision() { return revision; }, set revision(v) { revision = v; },
    get hubGgufFilename() { return hubGgufFilename; }, set hubGgufFilename(v) { hubGgufFilename = v; },
    
    get prompt() { return prompt; }, set prompt(v) { prompt = v; },
    get messages() { return messages; }, set messages(v) { messages = v; },
    get messagesEl() { return messagesEl; },
    get busy() { return busy; }, set busy(v) { busy = v; },
    get isLoaded() { return isLoaded; }, set isLoaded(v) { isLoaded = v; },
    get errorText() { return errorText; }, set errorText(v) { errorText = v; },
    get isLoadingModel() { return isLoadingModel; }, set isLoadingModel(v) { isLoadingModel = v; },
    get loadingProgress() { return loadingProgress; }, set loadingProgress(v) { loadingProgress = v; },
    get loadingStage() { return loadingStage; }, set loadingStage(v) { loadingStage = v; },
    get isCancelling() { return isCancelling; }, set isCancelling(v) { isCancelling = v; },
    get isUnloadingModel() { return isUnloadingModel; }, set isUnloadingModel(v) { isUnloadingModel = v; },
    get unloadingProgress() { return unloadingProgress; }, set unloadingProgress(v) { unloadingProgress = v; },
    get temperature() { return temperature; }, set temperature(v) { temperature = v; },
    get temperature_enabled() { return temperature_enabled; }, set temperature_enabled(v) { temperature_enabled = v; },
    get top_k_enabled() { return top_k_enabled; }, set top_k_enabled(v) { top_k_enabled = v; },
    get top_k_value() { return top_k_value; }, set top_k_value(v) { top_k_value = v; },
    get top_p_enabled() { return top_p_enabled; }, set top_p_enabled(v) { top_p_enabled = v; },
    get top_p_value() { return top_p_value; }, set top_p_value(v) { top_p_value = v; },
    get min_p_enabled() { return min_p_enabled; }, set min_p_enabled(v) { min_p_enabled = v; },
    get min_p_value() { return min_p_value; }, set min_p_value(v) { min_p_value = v; },
    get repeat_penalty_enabled() { return repeat_penalty_enabled; }, set repeat_penalty_enabled(v) { repeat_penalty_enabled = v; },
    get repeat_penalty_value() { return repeat_penalty_value; }, set repeat_penalty_value(v) { repeat_penalty_value = v; },
    get ctx_limit_value() { return ctx_limit_value; }, set ctx_limit_value(v) { ctx_limit_value = v; },
    // enable_thinking removed from controller API
    get use_custom_params() { return use_custom_params; }, set use_custom_params(v) { use_custom_params = v; },
    get use_gpu() { return use_gpu; }, set use_gpu(v) { use_gpu = v; },
    get cuda_available() { return cuda_available; }, set cuda_available(v) { cuda_available = v; },
    get cuda_build() { return cuda_build; }, set cuda_build(v) { cuda_build = v; },
    get current_device() { return current_device; }, set current_device(v) { current_device = v; },
  });

  const cancelLoading = controller.cancelLoading;
  const refreshDeviceInfo = controller.refreshDeviceInfo;
  const setDeviceByToggle = controller.setDeviceByToggle;

  const loadGGUF = controller.loadGGUF;

  const unloadGGUF = controller.unloadGGUF;

  // Формирование промпта вынесено в $lib/chat/prompts

  const handleSend = controller.handleSend;

  const generateFromHistory = controller.generateFromHistory;

  const stopGenerate = controller.stopGenerate;

  function mainAction() {
    try {
      if (isLoadingModel && typeof cancelLoading === 'function') {
        return cancelLoading();
      }
      if (isLoaded && typeof unloadGGUF === 'function') {
        return unloadGGUF();
      }
      if (typeof loadGGUF === 'function') {
        return loadGGUF();
      }
    } catch (e) {
      console.error('mainAction error', e);
    }
  }

  onDestroy(() => {
    chatUiMounted.set(false);
    // Persist chat/model state across navigation
    chatState.set({
      modelPath,
      repoId,
      revision,
      hubGgufFilename,
      format,

      prompt,
      messages,
      busy,
      isLoaded,
      errorText,

      isLoadingModel,
      loadingProgress,
      loadingStage,
      isCancelling,
      isUnloadingModel,
      unloadingProgress,

      temperature,
      temperature_enabled,
      top_k_enabled,
      top_k_value,
      top_p_enabled,
      top_p_value,
      min_p_enabled,
      min_p_value,
      repeat_penalty_enabled,
      repeat_penalty_value,
      ctx_limit_value,
      // enable_thinking removed from persisted state
      use_custom_params,

      use_gpu,
      cuda_available,
      cuda_build,
      current_device,
    });
    controller.destroy();
  });

  const pickModel = controller.pickModel;
  // expose minimal controller API to window for header GGUF control
  if (typeof window !== 'undefined') {
     
    // @ts-ignore
    window.__oxide = {
      pickModel: controller.pickModel,
      loadGGUF: controller.loadGGUF,
      unloadGGUF: controller.unloadGGUF,
      cancelLoading: controller.cancelLoading,
      getState: () => ({
        modelPath, isLoaded, isLoadingModel, isUnloadingModel, isCancelling, loadingStage, loadingProgress, unloadingProgress, busy
      })
    };
  }

  // Load persisted chat/model state when component mounts
  onMount(() => {
    chatUiMounted.set(true);
    try {
      const s = getStore(chatState) ?? getDefaultChatState();
      modelPath = s.modelPath;
      repoId = s.repoId;
      revision = s.revision;
      hubGgufFilename = s.hubGgufFilename;
      format = s.format;

      prompt = s.prompt;
      messages = Array.isArray(s.messages) ? s.messages : [];
      busy = s.busy;
      isLoaded = s.isLoaded;
      errorText = s.errorText;

      isLoadingModel = s.isLoadingModel;
      loadingProgress = s.loadingProgress;
      loadingStage = s.loadingStage;
      isCancelling = s.isCancelling;
      isUnloadingModel = s.isUnloadingModel;
      unloadingProgress = s.unloadingProgress;

      temperature = s.temperature;
      temperature_enabled = s.temperature_enabled;
      top_k_enabled = s.top_k_enabled; top_k_value = s.top_k_value;
      top_p_enabled = s.top_p_enabled; top_p_value = s.top_p_value;
      min_p_enabled = s.min_p_enabled; min_p_value = s.min_p_value;
      repeat_penalty_enabled = s.repeat_penalty_enabled; repeat_penalty_value = s.repeat_penalty_value;
      ctx_limit_value = s.ctx_limit_value;
      // enable_thinking removed from persisted state
      use_custom_params = s.use_custom_params;

      use_gpu = s.use_gpu;
      cuda_available = s.cuda_available;
      cuda_build = s.cuda_build;
      current_device = s.current_device;
    } catch (e) {
      // ignore, fall back to defaults
    }
    // UI остаётся смонтирован, восстановления и доп. переподключений не требуется
  });

  // Keep shared chatState in sync so header and other views get instant truth (no polling flicker)
  $: chatState.update((s) => ({
    ...s,
    modelPath,
    repoId,
    revision,
    hubGgufFilename,
    format,
    busy,
    isLoaded,
    isLoadingModel,
    isUnloadingModel,
    isCancelling,
    loadingStage,
    loadingProgress,
    unloadingProgress,
  }));
</script>

<main class="wrap">
  <!-- удалено дублирование заголовка -->
  {#if isLoaded}
    <section class="chat">
      <MessageList bind:messages bind:messagesEl />
      <Composer 
        bind:prompt 
        {busy} 
        {isLoaded} 
        on:send={handleSend} 
        on:stop={stopGenerate}
        on:attach={(e) => {
          // e.detail: { filename, content }
          // Forward to controller's attach handler
          // @ts-ignore
          const actions: any = controller;
          if (actions && typeof actions.handleAttachFile === 'function') {
            // @ts-ignore
            actions.handleAttachFile(e.detail);
          } else {
            console.log('Прикреплён файл', e.detail?.filename);
          }
        }}
        on:voice={() => console.log('Голосовое сообщение')}
      />
    </section>
  {:else}
    <section class="chat">
      <ChatPlaceholder />
    </section>
  {/if}

  <LoaderPanel
    bind:format
    bind:modelPath
    bind:repoId
    bind:revision
    bind:hubGgufFilename
    
    bind:ctx_limit_value
    bind:isLoadingModel
    bind:isUnloadingModel
    bind:isCancelling
    bind:loadingStage
    bind:loadingProgress
    bind:unloadingProgress
    bind:errorText
    bind:busy
    bind:isLoaded
    bind:use_gpu
    bind:cuda_available
    bind:cuda_build
    onMainAction={mainAction}
    on:device-toggle={(e: CustomEvent) => setDeviceByToggle(!!(e.detail as any)?.checked)}
  >
    <!-- Параметры инференса -->
    {#if isLoaded}
      <InferenceParams
        bind:use_custom_params
        bind:temperature_enabled
        bind:temperature
        bind:top_k_enabled
        bind:top_k_value
        bind:top_p_enabled
        bind:top_p_value
        bind:min_p_enabled
        bind:min_p_value
        bind:repeat_penalty_enabled
        bind:repeat_penalty_value
      />
    {/if}
  </LoaderPanel>
</main>




