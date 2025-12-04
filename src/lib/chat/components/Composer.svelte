<script lang="ts">

  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import Paperclip from 'phosphor-svelte/lib/Paperclip';
  import Broom from 'phosphor-svelte/lib/Broom';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import SlidersHorizontal from 'phosphor-svelte/lib/SlidersHorizontal';
  import ClockCounterClockwise from 'phosphor-svelte/lib/ClockCounterClockwise';
  import File from 'phosphor-svelte/lib/File';
  import X from 'phosphor-svelte/lib/X';
  import Textarea from '$lib/components/ui/textarea/textarea.svelte';
  import Button from '$lib/components/ui/button/button.svelte';
  import Badge from '$lib/components/ui/badge/badge.svelte';
  import Alert from '$lib/components/ui/alert/alert.svelte';
  import AlertDescription from '$lib/components/ui/alert/alert-description.svelte';
  import AlertTitle from '$lib/components/ui/alert/alert-title.svelte';
  import { cn } from '$lib/utils.js';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
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
  const DEFAULT_TEXT_ACCEPT = TEXT_EXTENSIONS.map((ext) => `.${ext}`).join(',');

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
    isChatHistoryVisible = false,
    hasMessages = false,
    onSend,
    onStop,
    onClear,
    onAttach,
    onToggleLoaderPanel,
    onToggleChatHistory
  }: Props = $props();

  let fileInput: HTMLInputElement | null = $state(null);
  let textareaElement: HTMLTextAreaElement | null = $state(null);
  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = $state(null);
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let accept = $state(DEFAULT_TEXT_ACCEPT);
  let attachedFiles: AttachDetail[] = $state([]);

  // Переменные для автоматического изменения высоты
  let textareaHeight = $state(34); // Стандартная высота однострочного поля
  const MIN_HEIGHT = 34; // Минимальная высота (как у кнопок)
  const MAX_HEIGHT = 102; // Максимальная высота (3 строки)

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;
    onSend?.();
  }

  function triggerStop() {
    if (!canStop) return;
    onStop?.();
  }

  function triggerClear() {
    if (!prompt && !attachError) return;
    prompt = '';
    attachError = null;
    clearErrorTimer();
    onClear?.();
  }

  function triggerVoiceInput() {
    if (isRecording) {
      // Остановить запись
      isRecording = false;
      console.log('Остановка записи голоса');
    } else {
      // Начать запись
      isRecording = true;
      console.log('Начало записи голоса');
    }
  }

  function triggerSettings() {
    onToggleLoaderPanel?.();
  }

  function triggerChatHistory() {
    onToggleChatHistory?.();
  }

  function removeAttachment(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  function triggerAttach() {
    if (!fileInput) return;
    fileInput.click();
  }


  function buildAccept() {
    const extensions: string[] = [];
    if (supports_text) extensions.push(...TEXT_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_image) extensions.push(...IMAGE_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_audio) extensions.push(...AUDIO_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_video) extensions.push(...VIDEO_EXTENSIONS.map((ext) => `.${ext}`));
    return extensions.join(',') || DEFAULT_TEXT_ACCEPT;
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

  async function handleFileChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement | null;
    const file = input?.files?.[0];
    if (!file) return;

    try {
      if (file.size > MAX_FILE_SIZE) {
        setError($t('errors.file.tooLarge'));
        return;
      }

      const name = file.name;
      const mime = file.type || '';
      const topLevel = mime.split('/')[0];
      const ext = (name.split('.').pop() || '').toLowerCase();

      const isTextLike =
        topLevel === 'text' || TEXT_EXTENSIONS.includes(ext) || mime === 'application/json';
      const isImage = topLevel === 'image' || IMAGE_EXTENSIONS.includes(ext);
      const isAudio = topLevel === 'audio' || AUDIO_EXTENSIONS.includes(ext);
      const isVideo = topLevel === 'video' || VIDEO_EXTENSIONS.includes(ext);

      if (isImage && !supports_image) {
        setError($t('chat.composer.errors.imageNotSupported'));
        return;
      }
      if (isAudio && !supports_audio) {
        setError($t('chat.composer.errors.audioNotSupported'));
        return;
      }
      if (isVideo && !supports_video) {
        setError($t('chat.composer.errors.videoNotSupported'));
        return;
      }

      if (isTextLike) {
        const content = await file.text();
        const attachment = { filename: name, content };
        attachedFiles = [...attachedFiles, attachment];
        onAttach?.(attachment);
        attachError = null;
        clearErrorTimer();
        return;
      }

      let marker: string | null = null;
      if (isImage) {
        const url = URL.createObjectURL(file);
        marker = `[image: ${url}]`;
      } else if (isAudio) {
        const url = URL.createObjectURL(file);
        marker = `[audio: ${url}]`;
      } else if (isVideo) {
        const url = URL.createObjectURL(file);
        marker = `[video: ${url}]`;
      }

      if (marker) {
        const attachment = { filename: name, content: marker };
        attachedFiles = [...attachedFiles, attachment];
        onAttach?.(attachment);
        attachError = null;
        clearErrorTimer();
      } else {
        setError($t('chat.composer.errors.unsupportedFileType'));
      }
    } catch (err) {
      console.error('Failed to read attachment', err);
      setError($t('chat.composer.errors.fileReadFailed'));
    } finally {
      if (input) input.value = '';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      triggerSend();
    }
  }

  function adjustTextareaHeight() {
    if (!textareaElement) return;

    // Сбрасываем высоту для корректного расчета
    textareaElement.style.height = 'auto';

    // Получаем scrollHeight для определения необходимой высоты
    const scrollHeight = textareaElement.scrollHeight;

    // Ограничиваем высоту минимальными и максимальными значениями
    if (scrollHeight <= MIN_HEIGHT) {
      textareaHeight = MIN_HEIGHT;
    } else if (scrollHeight >= MAX_HEIGHT) {
      textareaHeight = MAX_HEIGHT;
    } else {
      textareaHeight = scrollHeight;
    }

    // Применяем новую высоту
    textareaElement.style.height = `${textareaHeight}px`;
  }

  function handleTextareaInput() {
    adjustTextareaHeight();
  }

  let experimentalReady = $derived(experimentalFeatures.initialized && experimentalFeatures.enabled);
  let experimentalStatusMessage = $derived(experimentalFeatures.initialized
    ? experimentalFeatures.enabled
      ? null
      : $t('chat.composer.experimental.disabled')
    : $t('chat.composer.experimental.loading'));
  $effect(() => {
    accept = buildAccept();
  });
  // Реактивное обновление высоты при изменении prompt
  $effect(() => {
    if (prompt !== undefined) {
      // Используем setTimeout для корректного обновления после рендера
      setTimeout(() => {
        adjustTextareaHeight();
      }, 0);
    }
  });
</script>

<div class="composer-wrapper flex max-w-[640px] flex-col gap-3 px-4 sm:px-0" class:centered={!hasMessages} class:hidden={!isLoaded}>
  {#if attachedFiles.length > 0}
    <div class="flex flex-wrap gap-2">
      {#each attachedFiles as attachment, index}
        <Badge
          variant="secondary"
          class="bg-secondary/40 text-foreground/90 flex items-center gap-2 rounded-lg px-3 py-1 text-xs shadow-sm backdrop-blur"
        >
          <span class="flex items-center gap-1 text-muted-foreground">
            <File size={14} weight="bold" />
            <span class="max-w-[640px] truncate font-medium">{attachment.filename}</span>
          </span>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class="text-muted-foreground transition hover:text-foreground"
            onclick={() => removeAttachment(index)}
            aria-label={$t('errors.file.removeFile') + ' ' + attachment.filename}
          >
            <X size={14} weight="bold" />
            <span class="sr-only">{$t('errors.file.removeFile')} {attachment.filename}</span>
          </Button>
        </Badge>
      {/each}
    </div>
  {/if}

  <div class="composer-surface rounded-2xl border bg-background/80 shadow-xl backdrop-blur supports-[backdrop-filter]:bg-background/70">
    <div class="flex flex-col gap-3 p-3">
      <Textarea
        bind:value={prompt}
        bind:ref={textareaElement}
        placeholder={isLoaded ? $t('chat.composer.placeholder') : $t('chat.composer.placeholderNotLoaded') || $t('chat.composer.placeholder')}
        class="composer-textarea resize-none border-none bg-transparent px-0 pb-1 pt-0 text-sm shadow-none focus-visible:border-transparent focus-visible:ring-0"
        style={`height:${textareaHeight}px`}
        onkeydown={handleKeydown}
        oninput={handleTextareaInput}
        disabled={!isLoaded}
      />

      <div class="composer-controls flex flex-wrap items-center gap-2">
        <div class="flex items-center gap-1.5">
          <Button
            variant="ghost"
            size="icon-sm"
            class="icon-button"
            onclick={triggerAttach}
            disabled={busy || !isLoaded || !experimentalReady}
            aria-label={$t('chat.composer.attach')}
            title={experimentalStatusMessage ?? undefined}
          >
            <Paperclip size={16} weight="bold" />
          </Button>

          <Button
            variant="ghost"
            size="icon-sm"
            class={cn('icon-button', isChatHistoryVisible && 'is-active')}
            onclick={triggerChatHistory}
            aria-label={isChatHistoryVisible ? $t('chat.composer.hideHistory') : $t('chat.composer.showHistory')}
            disabled={!experimentalReady}
            title={experimentalStatusMessage ?? undefined}
          >
            <ClockCounterClockwise size={16} weight="bold" />
          </Button>

          <Button
            variant="ghost"
            size="icon-sm"
            class={cn('icon-button', isLoaderPanelVisible && 'is-active')}
            onclick={triggerSettings}
            aria-label={$t('chat.composer.loaderSettings')}
          >
            <SlidersHorizontal size={16} weight="bold" />
          </Button>
        </div>

        <div class="ml-auto flex items-center gap-1.5">
          {#if prompt || attachError}
            <Button
              variant="ghost"
              size="icon-sm"
              class="icon-button"
              onclick={triggerClear}
              disabled={busy}
              aria-label={$t('chat.composer.clear')}
            >
              <Broom size={16} weight="bold" />
            </Button>
          {/if}

          <Button
            variant="ghost"
            size="icon-sm"
            class="icon-button"
            onclick={triggerVoiceInput}
            disabled={busy || !isLoaded || !experimentalReady}
            aria-label={isRecording ? $t('chat.composer.voice.stopRecording') : $t('chat.composer.voice.startRecording')}
            aria-pressed={isRecording}
            title={experimentalStatusMessage ?? undefined}
          >
            {#if isRecording}
              <Stop size={16} weight="bold" />
            {:else}
              <Microphone size={16} weight="bold" />
            {/if}
          </Button>

          <Button
            variant="default"
            size="icon-sm"
            class="send-button"
            onclick={busy ? triggerStop : triggerSend}
            disabled={!isLoaded || (!busy && !prompt.trim())}
            aria-label={busy ? $t('chat.composer.stop') : $t('chat.composer.send')}
          >
            {#if busy}
              <Stop size={16} weight="bold" />
            {:else}
              <ArrowUp size={16} weight="bold" />
            {/if}
            <span class="sr-only">{busy ? $t('chat.composer.stop') : $t('chat.composer.send')}</span>
          </Button>
        </div>
      </div>
    </div>
  </div>

  <input
    class="sr-only"
    type="file"
    {accept}
    bind:this={fileInput}
    onchange={handleFileChange}
  />

  {#if attachError}
    <Alert variant="destructive" class="text-sm">
      <AlertTitle>{$t('chat.composer.errors.attachmentError')}</AlertTitle>
      <AlertDescription>{attachError}</AlertDescription>
    </Alert>
  {/if}
</div>

<style>
  .composer-wrapper {
    position: fixed;
    left: 50%;
    bottom: 84px;
    transform: translateX(-50%);
    z-index: 100;
    width: 100%;
    transition: top 0.3s ease, bottom 0.3s ease, transform 0.3s ease;
  }

  .composer-wrapper.centered {
    top: 50%;
    bottom: auto;
    transform: translate(-50%, -50%);
  }

  .composer-wrapper.hidden {
    display: none;
  }

  :global(.composer-textarea) {
    min-height: 34px;
  }

  :global(.icon-button) {
    border-radius: 9999px;
  }

  :global(.icon-button.is-active) {
    background-color: hsl(var(--primary) / 0.12);
    color: hsl(var(--primary));
  }

  :global(.send-button) {
    border-radius: 9999px;
  }
</style>
