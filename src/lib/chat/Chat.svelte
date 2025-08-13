<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import Composer from "$lib/chat/components/Composer.svelte";
  import LoaderPanel from "$lib/chat/components/LoaderPanel.svelte";
  import MessageList from "$lib/chat/components/MessageList.svelte";
  import ChatPlaceholder from "$lib/chat/components/ChatPlaceholder.svelte";
  import "./Chat.css";
  // Убрали переключатель «сырого» Markdown
  import type { ChatMessage } from "$lib/chat/types";
  import { createChatController } from "$lib/chat/controller";
  import InferenceParams from "$lib/chat/components/InferenceParams.svelte";

  let modelPath = "";
  let tokenizerPath = "";
  let prompt = "";
  let messages: ChatMessage[] = [];
  let messagesEl: HTMLDivElement | null = null;
  let busy = false;
  let format: "gguf" = "gguf";
  let isLoaded = false;
  let errorText = "";
  
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
  let enable_thinking: boolean = false;
  // Режим использования пользовательских параметров
  let use_custom_params: boolean = false;
  // Кол-во слоёв на GPU (ползунок)
  let n_gpu_layers: number = 0;

  const controller = createChatController({
    get modelPath() { return modelPath; }, set modelPath(v) { modelPath = v; },
    get tokenizerPath() { return tokenizerPath; }, set tokenizerPath(v) { tokenizerPath = v; },
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
    get enable_thinking() { return enable_thinking; }, set enable_thinking(v) { enable_thinking = v; },
    get use_custom_params() { return use_custom_params; }, set use_custom_params(v) { use_custom_params = v; },
    get n_gpu_layers() { return n_gpu_layers; }, set n_gpu_layers(v) { n_gpu_layers = v; },
  });

  const cancelLoading = controller.cancelLoading;

  const loadGGUF = controller.loadGGUF;

  const unloadGGUF = controller.unloadGGUF;

  // Формирование промпта вынесено в $lib/chat/prompts

  const handleSend = controller.handleSend;

  const generateFromHistory = controller.generateFromHistory;

  const stopGenerate = controller.stopGenerate;

  onDestroy(() => controller.destroy());

  const pickModel = controller.pickModel;
  const pickTokenizer = controller.pickTokenizer;
</script>

<main class="wrap">
  <!-- удалено дублирование заголовка -->
  <LoaderPanel
    bind:format
    bind:modelPath
    bind:tokenizerPath
    bind:enable_thinking
    bind:n_gpu_layers
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
    onPickModel={pickModel}
    onPickTokenizer={pickTokenizer}
    onMainAction={() => (isLoadingModel ? cancelLoading() : (isLoaded ? unloadGGUF() : loadGGUF()))}
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

  {#if isLoaded}
    <section class="chat">
      <MessageList bind:messages bind:messagesEl />
      <Composer 
        bind:prompt 
        {busy} 
        {isLoaded} 
        on:send={handleSend} 
        on:stop={stopGenerate}
        on:attach={() => console.log('Прикрепить файл')}
        on:voice={() => console.log('Голосовое сообщение')}
      />
    </section>
  {:else}
    <ChatPlaceholder />
  {/if}
</main>




