<script lang="ts">
  // no direct event subscriptions in Chat; streaming handled via controller
  import { onDestroy, onMount } from 'svelte';
  import LoaderPanel from '$lib/chat/components/LoaderPanel.svelte';
  import MessageList from '$lib/chat/components/MessageList.svelte';
  import Composer from '$lib/chat/components/Composer.svelte';
  import * as Sheet from '$lib/components/ui/sheet/index.js';
  // Chat styles are loaded globally from layout to avoid UI changes when navigating between pages
  // Убрали переключатель «сырого» Markdown
  import type { ChatMessage } from '$lib/chat/types';
  import { createChatController } from '$lib/chat/controller';
  import InferenceParams from '$lib/chat/components/InferenceParams.svelte';
  import { chatState, chatUiMounted, getDefaultChatState } from '$lib/stores/chat';
  import { currentSession } from '$lib/stores/chat-history';
  import { showChatHistory } from '$lib/stores/sidebar';
  import { get as getStore } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { performanceService } from '$lib/services/performance-service';
  import { inferenceMetricsStore } from '$lib/stores/inference-metrics';
  import type { InferenceMetrics } from '$lib/types/performance';

  // Добавляем состояние видимости лоадер панели
  let isLoaderPanelVisible = $state(false);

  let modelPath = $state('');
  let repoId = $state<string>('');
  let revision = $state<string>('');
  let hubGgufFilename = $state<string>('');
  let prompt = $state('');
  let messages = $state<ChatMessage[]>([]);
  let messagesEl = $state<HTMLDivElement | null>(null);
  let busy = $state(false);
  let format = $state<'gguf' | 'hub_gguf' | 'hub_safetensors' | 'local_safetensors'>('gguf');
  let pendingModelPath = $state('');
  let pendingFormat = $state<'gguf' | 'hub_gguf' | 'hub_safetensors' | 'local_safetensors'>('gguf');
  let isLoaded = $state(false);
  let errorText = $state('');
  // Устройство
  let use_gpu = $state<boolean>(false); // CPU по умолчанию
  let cuda_available = $state<boolean>(false);
  let cuda_build = $state<boolean>(false);
  let current_device = $state<string>('CPU');
  let avx = $state<boolean>(false);
  let neon = $state<boolean>(false);
  let simd128 = $state<boolean>(false);
  let f16c = $state<boolean>(false);

  // Поддержка модальностей (эвристика по имени модели)
  let supports_text = $state<boolean>(true);
  let supports_image = $state<boolean>(false);
  let supports_audio = $state<boolean>(false);
  let supports_video = $state<boolean>(false);

  async function refreshModalities() {
    try {
      const r: { text: boolean; image: boolean; audio: boolean; video: boolean } =
        await invoke('get_modality_support');
      supports_text = !!r.text;
      supports_image = !!r.image;
      supports_audio = !!r.audio;
      supports_video = !!r.video;
    } catch {
      // default to text-only
      supports_text = true;
      supports_image = false;
      supports_audio = false;
      supports_video = false;
    }
  }

  $effect(() => {
    if (isLoaded) {
      void refreshModalities();
    } else {
      supports_text = true;
      supports_image = false;
      supports_audio = false;
      supports_video = false;
    }
  });

  function detectModalities() {
    try {
      const s = `${modelPath} ${repoId} ${hubGgufFilename}`.toLowerCase();
      const has = (hints: string[]) => hints.some((h) => s.includes(h));
      const videoHints = ['vtt', 'video', 'onevision', 'llava'];
      const imageHints = ['itt', 'image', 'vision', 'gemma3', 'siglip'];
      const audioHints = ['att', 'audio', 'qwen2audio', 'whisper'];
      const any2anyHints = ['ata', 'any-to-any', 'multi_modality', 'multimodal', 'omni'];

      supports_text = true;
      supports_video = has(videoHints);
      supports_image = has(imageHints) || supports_video || has(any2anyHints);
      supports_audio = has(audioHints) || has(any2anyHints);
    } catch {}
  }

  $effect(() => {
    detectModalities();
  });

  const METRICS_KEY_PREFIX = 'oxide-infer-metrics-';

  function saveMetricsToStorage(sessionId: string, index: number, metrics: InferenceMetrics) {
    try {
      const key = `${METRICS_KEY_PREFIX}${sessionId}`;
      const raw = localStorage.getItem(key);
      const parsed = raw ? JSON.parse(raw) : {};
      parsed[index] = metrics;
      localStorage.setItem(key, JSON.stringify(parsed));
      // #region agent log
      void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run2',
          hypothesisId: 'H8',
          location: 'Chat.svelte:saveMetrics',
          message: 'persist metrics',
          data: { sessionId, index },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
      // #endregion
    } catch {}
  }

  function loadMetricsFromStorage(sessionId: string): Map<number, InferenceMetrics> {
    try {
      const key = `${METRICS_KEY_PREFIX}${sessionId}`;
      const raw = localStorage.getItem(key);
      if (!raw) return new Map();
      const parsed = JSON.parse(raw) as Record<string, InferenceMetrics>;
      const entries = Object.entries(parsed).map(([k, v]) => [Number(k), v] as [number, InferenceMetrics]);
      // #region agent log
      void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run2',
          hypothesisId: 'H9',
          location: 'Chat.svelte:loadMetrics',
          message: 'restore metrics from storage',
          data: { sessionId, restored: entries.length },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
      // #endregion
      return new Map(entries);
    } catch {
      return new Map();
    }
  }

  // Состояние загрузки модели
  let isLoadingModel = $state(false);
  let loadingProgress = $state(0);
  let loadingStage = $state(''); // "model" | "tokenizer" | "complete"
  let isCancelling = $state(false); // Флаг для отмены загрузки

  // Состояние выгрузки модели
  let isUnloadingModel = $state(false);
  let unloadingProgress = $state(0);

  // Параметры инференса
  let temperature = $state<number>(0.8);
  let temperature_enabled = $state(false);
  let top_k_enabled = $state(false);
  let top_k_value = $state<number>(40);
  let top_p_enabled = $state(false);
  let top_p_value = $state<number>(0.9);
  let min_p_enabled = $state(false);
  let min_p_value = $state<number>(0.05);
  let repeat_penalty_enabled = $state(false);
  let repeat_penalty_value = $state<number>(1.1);
  // Длина контекста (всегда активна и передаётся при загрузке)
  let ctx_limit_value = $state<number>(4096);
  // Управление размышлениями (enable_thinking)
  // removed: enable_thinking (no_think detection removed)
  // Режим использования пользовательских параметров
  let use_custom_params = $state<boolean>(false);
  let split_prompt = $state<boolean>(false);
  let verbose_prompt = $state<boolean>(false);
  let tracing = $state<boolean>(false);
  // Убран offloading: слои на GPU больше не настраиваются

  const controller = createChatController({
    get modelPath() {
      return modelPath;
    },
    set modelPath(v) {
      modelPath = v;
    },
    get format() {
      return format;
    },
    set format(v) {
      format = v;
    },
    get repoId() {
      return repoId;
    },
    set repoId(v) {
      repoId = v;
    },
    get revision() {
      return revision;
    },
    set revision(v) {
      revision = v;
    },
    get hubGgufFilename() {
      return hubGgufFilename;
    },
    set hubGgufFilename(v) {
      hubGgufFilename = v;
    },

    get prompt() {
      return prompt;
    },
    set prompt(v) {
      prompt = v;
    },
    get messages() {
      return messages;
    },
    set messages(v) {
      messages = v;
    },
    get messagesEl() {
      return messagesEl;
    },
    get busy() {
      return busy;
    },
    set busy(v) {
      busy = v;
    },
    get isLoaded() {
      return isLoaded;
    },
    set isLoaded(v) {
      isLoaded = v;
    },
    get errorText() {
      return errorText;
    },
    set errorText(v) {
      errorText = v;
    },
    get isLoadingModel() {
      return isLoadingModel;
    },
    set isLoadingModel(v) {
      isLoadingModel = v;
    },
    get loadingProgress() {
      return loadingProgress;
    },
    set loadingProgress(v) {
      loadingProgress = v;
    },
    get loadingStage() {
      return loadingStage;
    },
    set loadingStage(v) {
      loadingStage = v;
    },
    get isCancelling() {
      return isCancelling;
    },
    set isCancelling(v) {
      isCancelling = v;
    },
    get isUnloadingModel() {
      return isUnloadingModel;
    },
    set isUnloadingModel(v) {
      isUnloadingModel = v;
    },
    get unloadingProgress() {
      return unloadingProgress;
    },
    set unloadingProgress(v) {
      unloadingProgress = v;
    },
    get temperature() {
      return temperature;
    },
    set temperature(v) {
      temperature = v;
    },
    get temperature_enabled() {
      return temperature_enabled;
    },
    set temperature_enabled(v) {
      temperature_enabled = v;
    },
    get top_k_enabled() {
      return top_k_enabled;
    },
    set top_k_enabled(v) {
      top_k_enabled = v;
    },
    get top_k_value() {
      return top_k_value;
    },
    set top_k_value(v) {
      top_k_value = v;
    },
    get top_p_enabled() {
      return top_p_enabled;
    },
    set top_p_enabled(v) {
      top_p_enabled = v;
    },
    get top_p_value() {
      return top_p_value;
    },
    set top_p_value(v) {
      top_p_value = v;
    },
    get min_p_enabled() {
      return min_p_enabled;
    },
    set min_p_enabled(v) {
      min_p_enabled = v;
    },
    get min_p_value() {
      return min_p_value;
    },
    set min_p_value(v) {
      min_p_value = v;
    },
    get repeat_penalty_enabled() {
      return repeat_penalty_enabled;
    },
    set repeat_penalty_enabled(v) {
      repeat_penalty_enabled = v;
    },
    get repeat_penalty_value() {
      return repeat_penalty_value;
    },
    set repeat_penalty_value(v) {
      repeat_penalty_value = v;
    },
    get ctx_limit_value() {
      return ctx_limit_value;
    },
    set ctx_limit_value(v) {
      ctx_limit_value = v;
    },
    // enable_thinking removed from controller API
    get use_custom_params() {
      return use_custom_params;
    },
    set use_custom_params(v) {
      use_custom_params = v;
    },
    get use_gpu() {
      return use_gpu;
    },
    set use_gpu(v) {
      use_gpu = v;
    },
    get cuda_available() {
      return cuda_available;
    },
    set cuda_available(v) {
      cuda_available = v;
    },
    get cuda_build() {
      return cuda_build;
    },
    set cuda_build(v) {
      cuda_build = v;
    },
    get current_device() {
      return current_device;
    },
    set current_device(v) {
      current_device = v;
    },
    get avx() {
      return avx;
    },
    set avx(v) {
      avx = v;
    },
    get neon() {
      return neon;
    },
    set neon(v) {
      neon = v;
    },
    get simd128() {
      return simd128;
    },
    set simd128(v) {
      simd128 = v;
    },
    get f16c() {
      return f16c;
    },
    set f16c(v) {
      f16c = v;
    },
    // Модальности
    get supports_text() {
      return supports_text;
    },
    set supports_text(v) {
      supports_text = v;
    },
    get supports_image() {
      return supports_image;
    },
    set supports_image(v) {
      supports_image = v;
    },
    get supports_audio() {
      return supports_audio;
    },
    set supports_audio(v) {
      supports_audio = v;
    },
    get supports_video() {
      return supports_video;
    },
    set supports_video(v) {
      supports_video = v;
    },
    get split_prompt() {
      return split_prompt;
    },
    set split_prompt(v) {
      split_prompt = v;
    },
    get verbose_prompt() {
      return verbose_prompt;
    },
    set verbose_prompt(v) {
      verbose_prompt = v;
    },
    get tracing() {
      return tracing;
    },
    set tracing(v) {
      tracing = v;
    },
  });

  const cancelLoading = controller.cancelLoading;
  const _refreshDeviceInfo = controller.refreshDeviceInfo;
  const setDeviceByToggle = controller.setDeviceByToggle;

  const loadGGUF = controller.loadGGUF;

  const unloadGGUF = controller.unloadGGUF;
  const sendMessage = controller.handleSend;
  const stopGenerate = controller.stopGenerate;
  const _regenerateFromHistory = controller.generateFromHistory;
  const attachFileToPrompt = controller.handleAttachFile;

  // Формирование промпта вынесено в $lib/chat/prompts

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

  function loadModelFromManager(args: { path: string; format: 'gguf' | 'local_safetensors' }) {
    if (!args?.path) return;
    pendingModelPath = args.path;
    pendingFormat = args.format;

    if (isLoaded || isLoadingModel) {
      return;
    }

    format = args.format;
    modelPath = args.path;
    repoId = '';
    revision = '';
    hubGgufFilename = '';
    pendingModelPath = '';
    pendingFormat = 'gguf';
    void loadGGUF?.();
  }

  async function reloadSelectedModel() {
    if (!pendingModelPath || pendingModelPath === modelPath) return;
    try {
      await stopGenerate();
    } catch {}

    await unloadGGUF();
    format = pendingFormat;
    modelPath = pendingModelPath;
    repoId = '';
    revision = '';
    hubGgufFilename = '';
    pendingModelPath = '';
    pendingFormat = 'gguf';
    void loadGGUF?.();
  }

  function toggleLoaderPanelVisibility() {
    isLoaderPanelVisible = !isLoaderPanelVisible;
  }

  function toggleChatHistoryVisibility() {
    showChatHistory.update((value) => !value);
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
      pendingModelPath,
      pendingFormat,

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
      avx,
      neon,
      simd128,
      f16c,
      split_prompt,
      verbose_prompt,
      tracing,
    });
    controller.destroy();
  });

  const _pickModel = controller.pickModel;
  // expose minimal controller API to window for header GGUF control
  if (typeof window !== 'undefined') {
    // @ts-ignore
    window.__oxide = {
      pickModel: controller.pickModel,
      loadModelFromManager,
      reloadSelectedModel,
      loadGGUF: controller.loadGGUF,
      unloadGGUF: controller.unloadGGUF,
      cancelLoading: controller.cancelLoading,
      getState: () => ({
        currentModelPath: modelPath,
        currentFormat: format,
        modelPath,
        format,
        isLoaded,
        isLoadingModel,
        isUnloadingModel,
        isCancelling,
        loadingStage,
        loadingProgress,
        unloadingProgress,
        busy,
        pendingModelPath,
        pendingFormat,
      }),
    };
  }

  // Load persisted chat/model state when component mounts
  onMount(async () => {
    chatUiMounted.set(true);
    try {
      const s = getStore(chatState) ?? getDefaultChatState();
      modelPath = s.modelPath;
      repoId = s.repoId;
      revision = s.revision;
      hubGgufFilename = s.hubGgufFilename;
      format = s.format;

      prompt = s.prompt;
      // Загружаем сообщения из истории чатов, если есть текущая сессия
      const session = getStore(currentSession);
      messages = session?.messages ?? (Array.isArray(s.messages) ? s.messages : []);
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
      top_k_enabled = s.top_k_enabled;
      top_k_value = s.top_k_value;
      top_p_enabled = s.top_p_enabled;
      top_p_value = s.top_p_value;
      min_p_enabled = s.min_p_enabled;
      min_p_value = s.min_p_value;
      repeat_penalty_enabled = s.repeat_penalty_enabled;
      repeat_penalty_value = s.repeat_penalty_value;
      ctx_limit_value = s.ctx_limit_value;
      // enable_thinking removed from persisted state
      use_custom_params = s.use_custom_params;

      use_gpu = s.use_gpu;
      cuda_available = s.cuda_available;
      cuda_build = s.cuda_build;
      current_device = s.current_device;
      avx = s.avx;
      neon = s.neon;
      simd128 = s.simd128;
      f16c = s.f16c;
      split_prompt = s.split_prompt;
      verbose_prompt = s.verbose_prompt;
      tracing = s.tracing;
    } catch {
      // ignore, fall back to defaults
    }

    // Initialize stream listener to handle incoming tokens
    try {
      await controller.ensureStreamListener();
    } catch (err) {
      console.warn('Failed to initialize stream listener:', err);
    }

    // Подписываемся на события метрик производительности
    await performanceService.setupEventListeners(
      undefined, // Не обрабатываем загрузку модели здесь
      (inferenceMetrics: InferenceMetrics) => {
        // Даём небольшую задержку на случай, если сообщение ещё не добавлено
        setTimeout(() => {
          const lastAssistantIndex = messages.findLastIndex((m) => m.role === 'assistant');
          if (lastAssistantIndex !== -1) {
            inferenceMetricsStore.setMetrics(lastAssistantIndex, inferenceMetrics);
            const cs = getStore(currentSession);
            const sid = cs?.id;
            if (sid) {
              saveMetricsToStorage(sid, lastAssistantIndex, inferenceMetrics);
            }
          }
        }, 150);
      },
    );
  });

  // Флаг для предотвращения циклических обновлений
  // let isLoadingFromHistory = false; // Удалено как неиспользуемое
  let lastSessionId: string | null = null;

  // Загрузка сообщений при переключении сессии
  $effect(() => {
    if ($currentSession && $currentSession.id !== lastSessionId) {
      // #region agent log
      void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run2',
          hypothesisId: 'H5',
          location: 'Chat.svelte:$effect',
          message: 'load currentSession into view',
          data: {
            sessionId: $currentSession.id,
            messagesCount: $currentSession.messages.length,
            roles: $currentSession.messages.map((m) => m.role),
          },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
      // #endregion
      // isLoadingFromHistory = true; // Удалено
      messages = [...$currentSession.messages]; // Создаем новый массив для триггера реактивности
      lastSessionId = $currentSession.id;

      // Восстанавливаем метрики из localStorage для выбранной сессии
      const restoredMetrics = loadMetricsFromStorage($currentSession.id);
      if (restoredMetrics.size > 0) {
        inferenceMetricsStore.set(restoredMetrics);
      } else {
        inferenceMetricsStore.clear();
      }

      // Сбрасываем флаг после небольшой задержки - удалено
      // setTimeout(() => {
      //   isLoadingFromHistory = false;
      // }, 100);
    }
  });

  // Синхронизация сообщений с историей чатов теперь происходит явно в actions.ts и listener.ts
  // $effect(() => {
  //   if (messages && !isLoadingFromHistory && $currentSession) {
  //     console.log('Синхронизируем сообщения в историю:', messages.length);
  //     chatHistory.updateMessages(messages);
  //   }
  // });

  let _canRegenerate = false;

  let isChatHistoryVisible = $derived(!!$showChatHistory);
  let hasMessages = $derived((messages?.length ?? 0) > 0);
  let canStopGeneration = $derived(busy && isLoaded);

  // Очищаем слушатели событий при размонтировании
  onDestroy(() => {
    performanceService.cleanup();
  });

  // Keep shared chatState in sync so header and other views get instant truth (no polling flicker)
  $effect(() => {
    chatState.update((s) => ({
      ...s,
      modelPath,
      repoId,
      revision,
      hubGgufFilename,
      format,
      pendingModelPath,
      pendingFormat,
      busy,
      isLoaded,
      isLoadingModel,
      isUnloadingModel,
      isCancelling,
      loadingStage,
      loadingProgress,
      unloadingProgress,
    }));
  });
</script>

<main class="wrap">
  <div class="chat-container">
    <!-- удалено дублирование заголовка -->
    <section class="chat">
      <MessageList
        bind:messages
        bind:messagesEl
        showModelNotice={!isLoaded && messages.length === 0}
      />
      <Composer
        bind:prompt
        {busy}
        {isLoaded}
        canStop={canStopGeneration}
        {supports_text}
        {supports_image}
        {supports_audio}
        {supports_video}
        {isLoaderPanelVisible}
        {isChatHistoryVisible}
        {hasMessages}
        onSend={sendMessage}
        onStop={stopGenerate}
        onAttach={attachFileToPrompt}
        onToggleLoaderPanel={toggleLoaderPanelVisibility}
        onToggleChatHistory={toggleChatHistoryVisibility}
      />
    </section>

    <Sheet.Root bind:open={isLoaderPanelVisible}>
      <Sheet.Content side="right" class="w-full sm:max-w-[450px] p-0">
        <Sheet.Header class="pb-2">
          <Sheet.Title>Loader panel</Sheet.Title>
        </Sheet.Header>
        <div class="loader-sheet-body flex-1 overflow-y-auto p-4 pt-0">
          <LoaderPanel
            class="h-full"
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
            bind:avx
            bind:neon
            bind:simd128
            bind:f16c
            {supports_text}
            {supports_image}
            {supports_audio}
            {supports_video}
            bind:split_prompt
            bind:verbose_prompt
            bind:tracing
            onMainAction={mainAction}
            onDeviceToggle={(detail: { checked: boolean }) => setDeviceByToggle(!!detail?.checked)}
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
        </div>
      </Sheet.Content>
    </Sheet.Root>
  </div>
</main>

<style>
  /* Контейнер для чата; панель настроек рендерится через Sheet поверх */
  .chat-container {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    height: 100%;
    background: #1a1f2e;
  }

  :global(main.wrap) {
    padding: 0 !important;
  }

  /* Левая колонка (сообщения+композер) занимает всё доступное */
  .chat {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    position: relative;
  }

  .loader-sheet-body :global(.loader) {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }

  /* Не затемнять хедер/сайдбар при открытии LoaderPanel */
  :global([data-slot='sheet-overlay']) {
    background: transparent !important;
  }
</style>
