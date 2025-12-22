<script lang="ts">
  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import Paperclip from 'phosphor-svelte/lib/Paperclip';
  import Broom from 'phosphor-svelte/lib/Broom';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import SlidersHorizontal from 'phosphor-svelte/lib/SlidersHorizontal';
  import X from 'phosphor-svelte/lib/X';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import Check from 'phosphor-svelte/lib/Check';
  import { onDestroy, onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import {
    PromptInput,
    PromptInputTextarea,
    PromptInputTools,
    PromptInputButton,
    PromptInputAttachments,
    PromptInputAttachment,
    getPromptInputAttachments,
    type PromptInputMessage,
  } from '$lib/components/prompt-input';

  import { cn } from '$lib/utils.js';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import {
    startVoiceCapture as startVoiceCaptureSession,
    type VoiceCapture,
  } from '$lib/services/voice-input';
  import { t } from '$lib/i18n';

  type AttachDetail = {
    filename: string;
    content: string;
  };

  const TEXT_EXTENSIONS = [
    'txt',
    'md',
    'markdown',
    'json',
    'log',
    'csv',
    'yaml',
    'yml',
    'xml',
    'html',
  ];
  const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif'];
  const AUDIO_EXTENSIONS = ['wav', 'mp3', 'ogg', 'flac', 'm4a'];
  const VIDEO_EXTENSIONS = ['mp4', 'webm', 'mov', 'mkv'];
  const STT_LANGUAGE_STORAGE_KEY = 'oxide-stt-language';
  const RMS_BAR_SEEDS = [0.42, 0.6, 0.82, 0.68, 0.5, 0.74, 0.56];

  type SttLanguage =
    | 'auto'
    | 'en'
    | 'ru'
    | 'es'
    | 'fr'
    | 'de'
    | 'it'
    | 'pt'
    | 'uk'
    | 'pl'
    | 'tr'
    | 'ja'
    | 'ko'
    | 'zh'
    | 'hi'
    | 'ar';

  const STT_LANGUAGE_OPTIONS: { value: SttLanguage; label: string }[] = [
    { value: 'auto', label: 'Auto' },
    { value: 'en', label: 'English' },
    { value: 'ru', label: 'Russian' },
    { value: 'es', label: 'Spanish' },
    { value: 'fr', label: 'French' },
    { value: 'de', label: 'German' },
    { value: 'it', label: 'Italian' },
    { value: 'pt', label: 'Portuguese' },
    { value: 'uk', label: 'Ukrainian' },
    { value: 'pl', label: 'Polish' },
    { value: 'tr', label: 'Turkish' },
    { value: 'ja', label: 'Japanese' },
    { value: 'ko', label: 'Korean' },
    { value: 'zh', label: 'Chinese' },
    { value: 'hi', label: 'Hindi' },
    { value: 'ar', label: 'Arabic' },
  ];

  interface Props {
    prompt?: string;
    busy?: boolean;
    isLoaded?: boolean;
    canStop?: boolean;
    isRecording?: boolean;
    supports_text?: boolean;
    supports_image?: boolean;
    supports_audio?: boolean;
    supports_video?: boolean;
    isLoaderPanelVisible?: boolean;
    isChatHistoryVisible?: boolean;
    hasMessages?: boolean;
    onSend?: () => void;
    onStop?: () => void;
    onClear?: () => void;
    onAttach?: (detail: AttachDetail) => void;
    onVoiceTranscribe?: (text: string) => Promise<void>;
    onToggleLoaderPanel?: () => void;
    onToggleChatHistory?: () => void;
  }

  let {
    prompt = $bindable(''),
    busy = false,
    isLoaded = false,
    canStop = false,
    isRecording = $bindable(false),
    supports_text = true,
    supports_image = false,
    supports_audio = false,
    supports_video = false,
    isLoaderPanelVisible = false,
    isChatHistoryVisible: _isChatHistoryVisible = false,
    hasMessages = false,
    onSend,
    onStop,
    onClear,
    onAttach,
    onVoiceTranscribe,
    onToggleLoaderPanel,
    onToggleChatHistory: _onToggleChatHistory,
  }: Props = $props();

  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = $state(null);
  let voiceError: string | null = $state(null);
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let voiceErrorTimer: ReturnType<typeof setTimeout> | null = null;
  let voiceCapture: VoiceCapture | null = $state(null);
  let isTranscribing = $state(false);
  let sttLanguage = $state<SttLanguage>('auto');
  let voiceRms = $state(0);
  let rmsPhase = $state(0);
  let rmsUnlisten: UnlistenFn | null = null;
  let rmsTimer: ReturnType<typeof setInterval> | null = null;
  let lastRmsAt = $state(0);

  const RMS_BAR_COUNT = 6;
  const isVoiceActive = $derived(isRecording || isTranscribing);
  const sendDisabled = $derived(!isLoaded || isVoiceActive || (!busy && !prompt.trim()));
  const rmsBars = $derived(buildRmsBars(voiceRms, rmsPhase));

  // Build accept string for file input
  const accept = $derived(buildAccept());

  function buildAccept() {
    const extensions: string[] = [];
    if (supports_text) extensions.push(...TEXT_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_image) extensions.push(...IMAGE_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_audio) extensions.push(...AUDIO_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_video) extensions.push(...VIDEO_EXTENSIONS.map((ext) => `.${ext}`));
    return extensions.join(',') || TEXT_EXTENSIONS.map((ext) => `.${ext}`).join(',');
  }

  function clearErrorTimer() {
    if (errorTimer) {
      clearTimeout(errorTimer);
      errorTimer = null;
    }
  }

  function setError(message: string) {
    attachError = message;
    clearErrorTimer();
    errorTimer = setTimeout(() => {
      attachError = null;
      errorTimer = null;
    }, 4000);
  }

  function clearVoiceErrorTimer() {
    if (voiceErrorTimer) {
      clearTimeout(voiceErrorTimer);
      voiceErrorTimer = null;
    }
  }

  function setVoiceError(message: string) {
    voiceError = message;
    clearVoiceErrorTimer();
    voiceErrorTimer = setTimeout(() => {
      voiceError = null;
      voiceErrorTimer = null;
    }, 4000);
  }

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim() || isVoiceActive) return;
    onSend?.();
  }

  function triggerStop() {
    if (!canStop) return;
    onStop?.();
  }

  function triggerClear() {
    if (!prompt && !attachError && !voiceError) return;
    prompt = '';
    attachError = null;
    voiceError = null;
    clearErrorTimer();
    clearVoiceErrorTimer();
    onClear?.();
  }

  function triggerVoiceInput() {
    if (isTranscribing) return;
    if (isRecording) {
      void endVoiceCapture();
      return;
    }
    void beginVoiceCapture();
  }

  function triggerSettings() {
    onToggleLoaderPanel?.();
  }

  async function beginVoiceCapture() {
    if (isRecording || voiceCapture) return;
    try {
      voiceCapture = await startVoiceCaptureSession();
      voiceRms = 0;
      isRecording = true;
      voiceError = null;
      clearVoiceErrorTimer();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      if (message.includes('Recording already in progress')) {
        isRecording = true;
        return;
      }
      console.error('Failed to start voice capture', err);
      setVoiceError($t('chat.composer.voice.captureFailed'));
    }
  }

  async function endVoiceCapture() {
    if (!voiceCapture) {
      isRecording = false;
      return;
    }
    isTranscribing = true;
    isRecording = false;
    try {
      const language = sttLanguage === 'auto' ? null : sttLanguage;
      const text = await voiceCapture.stop(language);
      voiceCapture = null;
      await onVoiceTranscribe?.(text);
    } catch (err) {
      console.error('Failed to transcribe voice input', err);
      setVoiceError($t('chat.composer.voice.transcriptionFailed'));
    } finally {
      isTranscribing = false;
      voiceRms = 0;
    }
  }

  function handleSubmit(message: PromptInputMessage) {
    // Handle attached files from the ai-elements system
    for (const file of message.files) {
      if (file.filename && file.url) {
        onAttach?.({ filename: file.filename, content: file.url });
      }
    }
    // Trigger the actual send
    triggerSend();
  }

  function handleError(err: { code: string; message: string }) {
    setError(err.message);
  }

  function updateSttLanguage(next: SttLanguage) {
    sttLanguage = next;
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STT_LANGUAGE_STORAGE_KEY, next);
      } catch (err) {
        console.error('Failed to persist STT language preference', err);
      }
    }
  }

  function buildRmsBars(level: number, phase: number) {
    const intensity = Math.sqrt(Math.min(1, Math.max(0, level)));
    return RMS_BAR_SEEDS.slice(0, RMS_BAR_COUNT).map((seed, index) => {
      const jitter = 0.4 + 0.6 * (Math.sin(phase + index * 1.6) * 0.5 + 0.5);
      return Math.min(1, intensity * (0.5 + seed * jitter));
    });
  }

  let experimentalReady = $derived(
    experimentalFeatures.initialized && experimentalFeatures.enabled,
  );
  let experimentalStatusMessage = $derived(
    experimentalFeatures.initialized
      ? experimentalFeatures.enabled
        ? null
        : $t('chat.composer.experimental.disabled')
      : $t('chat.composer.experimental.loading'),
  );

  $effect(() => {
    if (!isVoiceActive) {
      voiceRms = 0;
    }
  });

  onMount(async () => {
    if (typeof localStorage !== 'undefined') {
      try {
        const saved = localStorage.getItem(STT_LANGUAGE_STORAGE_KEY);
        if (saved && STT_LANGUAGE_OPTIONS.some((option) => option.value === saved)) {
          sttLanguage = saved as SttLanguage;
        }
      } catch (err) {
        console.error('Failed to load STT language preference', err);
      }
    }

    try {
      rmsUnlisten = await listen<number>('voice_rms', (event) => {
        const next = Math.min(1, Math.max(0, event.payload ?? 0));
        voiceRms = next;
        lastRmsAt = Date.now();
        rmsPhase = Math.random() * Math.PI * 2;
      });
    } catch (err) {
      console.error('Failed to listen for voice RMS', err);
    }
  });

  onDestroy(() => {
    if (rmsUnlisten) {
      rmsUnlisten();
      rmsUnlisten = null;
    }
    if (rmsTimer) {
      clearInterval(rmsTimer);
      rmsTimer = null;
    }
  });

  $effect(() => {
    if (isVoiceActive && !rmsTimer) {
      rmsTimer = setInterval(() => {
        if (!isRecording) return;
        const now = Date.now();
        if (now - lastRmsAt > 200) {
          voiceRms = 0.35 + Math.random() * 0.45;
          rmsPhase = Math.random() * Math.PI * 2;
        }
      }, 120);
    }
    if (!isVoiceActive && rmsTimer) {
      clearInterval(rmsTimer);
      rmsTimer = null;
    }
  });
</script>

<div
  class={cn(
    'absolute left-1/2 w-[640px] max-w-[calc(100%-2rem)] -translate-x-1/2 px-0 transition-[top,bottom,transform] duration-300 z-10',
    hasMessages ? 'bottom-6 pb-0 bg-transparent' : 'top-1/2 -translate-y-1/2',
  )}
>
  <PromptInput
    class="rounded-2xl border bg-card/80 shadow-xl backdrop-blur supports-[backdrop-filter]:bg-card/70"
    {accept}
    multiple={false}
    maxFileSize={MAX_FILE_SIZE}
    onSubmit={handleSubmit}
    onError={handleError}
  >
    <div class="flex flex-col p-3 pb-4 sm:p-4 sm:pb-5">
      <!-- Attachments Preview -->
      <PromptInputAttachments class="px-0">
        {#snippet children(file)}
          <PromptInputAttachment data={file} />
        {/snippet}
      </PromptInputAttachments>

      <!-- Main Input Area -->
      <PromptInputTextarea
        bind:value={prompt}
        placeholder={isLoaded
          ? $t('chat.composer.placeholder')
          : $t('chat.composer.placeholderNotLoaded') || $t('chat.composer.placeholder')}
        class="composer-input min-h-[38px] max-h-[120px] w-full resize-none bg-transparent px-2 py-2 text-base outline-none"
      />

      <!-- Toolbar -->
      <div
        class="flex w-full items-center justify-between gap-2 px-1 pt-2"
        style="margin-bottom: 4px;"
      >
        {#if isVoiceActive}
          <!-- Voice Active State -->
          <div class="flex-1"></div>
          <div class="ml-auto flex flex-shrink-0 items-center gap-1.5">
            {#if isTranscribing}
              <PromptInputButton disabled aria-label={$t('chat.composer.voice.transcribing')}>
                <Spinner size={14} class="model-combobox-spinner" />
              </PromptInputButton>
            {:else}
              <div class="voice-spectrum" aria-hidden="true">
                {#each rmsBars as bar, index (index)}
                  <span class="voice-bar" style={`height:${10 + bar * 16}px`}></span>
                {/each}
              </div>
              <DropdownMenu.Root>
                <DropdownMenu.Trigger
                  class={cn(buttonVariants({ variant: 'ghost', size: 'icon-sm' }), 'rounded-full')}
                  aria-label="Select voice language"
                >
                  <CaretDown size={14} weight="bold" />
                </DropdownMenu.Trigger>
                <DropdownMenu.Content align="end" class="w-48">
                  {#each STT_LANGUAGE_OPTIONS as option}
                    <DropdownMenu.Item
                      class={cn(
                        'flex items-center justify-between',
                        sttLanguage === option.value && 'bg-accent',
                      )}
                      onclick={() => updateSttLanguage(option.value)}
                    >
                      <span>{option.label}</span>
                      {#if sttLanguage === option.value}
                        <Check size={14} weight="bold" class="text-accent-foreground" />
                      {/if}
                    </DropdownMenu.Item>
                  {/each}
                </DropdownMenu.Content>
              </DropdownMenu.Root>
            {/if}
            {#if isRecording && !isTranscribing}
              <PromptInputButton
                class="text-destructive"
                onclick={triggerVoiceInput}
                aria-label={$t('chat.composer.voice.stopRecording')}
              >
                <Stop size={16} weight="bold" />
              </PromptInputButton>
            {/if}
            <Button
              variant="default"
              size="icon-sm"
              class="rounded-full"
              onclick={busy ? triggerStop : triggerSend}
              disabled={sendDisabled}
              aria-label={$t('chat.composer.send')}
            >
              <ArrowUp size={16} weight="bold" />
            </Button>
          </div>
        {:else}
          <!-- Normal State -->
          <PromptInputTools class="translate-x-1.5">
            <PromptInputButton
              onclick={() => {
                // Get attachments from context would require being inside the context
                // For now, we trigger the file input directly
                const input = document.querySelector(
                  'input[name="prompt-input-files"]',
                ) as HTMLInputElement;
                input?.click();
              }}
              disabled={busy || !isLoaded || !experimentalReady}
              aria-label={$t('chat.composer.attach')}
              title={experimentalStatusMessage ?? undefined}
            >
              <Paperclip size={16} weight="bold" />
            </PromptInputButton>

            <PromptInputButton
              class={cn(isLoaderPanelVisible && 'text-primary')}
              onclick={triggerSettings}
              disabled={!isLoaded || busy}
              aria-label={$t('chat.composer.loaderSettings')}
            >
              <SlidersHorizontal size={16} weight="bold" />
            </PromptInputButton>

            {#if prompt || attachError}
              <PromptInputButton onclick={triggerClear} aria-label={$t('chat.composer.clear')}>
                <Broom size={16} weight="bold" />
              </PromptInputButton>
            {/if}
          </PromptInputTools>

          <PromptInputTools class="-translate-x-1.5">
            <PromptInputButton
              class={cn(isRecording && 'text-destructive')}
              onclick={triggerVoiceInput}
              disabled={busy || isTranscribing}
              aria-label={isRecording
                ? $t('chat.composer.voice.stopRecording')
                : $t('chat.composer.voice.startRecording')}
              aria-pressed={isRecording}
              title={isTranscribing ? $t('chat.composer.voice.transcribing') : undefined}
            >
              {#if isRecording}
                <Stop size={16} weight="bold" />
              {:else}
                <Microphone size={16} weight="bold" />
              {/if}
            </PromptInputButton>

            <Button
              variant="default"
              size="icon-sm"
              class="rounded-full"
              onclick={busy ? triggerStop : triggerSend}
              disabled={sendDisabled}
              aria-label={busy ? $t('chat.composer.stop') : $t('chat.composer.send')}
              type="button"
            >
              {#if busy}
                <Stop size={16} weight="bold" />
              {:else}
                <ArrowUp size={16} weight="bold" />
              {/if}
            </Button>
          </PromptInputTools>
        {/if}
      </div>

      <!-- Error Messages -->
      {#if attachError || voiceError}
        <Alert variant="destructive" class="text-sm">
          <AlertTitle>
            {voiceError
              ? $t('chat.composer.voice.errorTitle')
              : $t('chat.composer.errors.attachmentError')}
          </AlertTitle>
          <AlertDescription>{voiceError ?? attachError}</AlertDescription>
        </Alert>
      {/if}
    </div>
  </PromptInput>
</div>

<style>
  :global(textarea.composer-input) {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
    background-color: transparent !important;
  }

  :global(textarea.composer-input:focus),
  :global(textarea.composer-input:focus-visible) {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
    background-color: transparent !important;
  }

  .voice-spectrum {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 12px;
    height: 28px;
    min-width: 64px;
    flex-shrink: 0;
  }

  .voice-bar {
    display: inline-block;
    width: 4px;
    border-radius: 999px;
    background: var(--color-primary);
    transition: height 120ms ease;
  }
</style>
