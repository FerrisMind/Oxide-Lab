<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import Paperclip from 'phosphor-svelte/lib/Paperclip';
  import Broom from 'phosphor-svelte/lib/Broom';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import SlidersHorizontal from 'phosphor-svelte/lib/SlidersHorizontal';
  import ClockCounterClockwise from 'phosphor-svelte/lib/ClockCounterClockwise';
  import File from 'phosphor-svelte/lib/File';
  import X from 'phosphor-svelte/lib/X';
  import Button from '$lib/components/ui/button/button.svelte';
  import * as InputGroup from '$lib/components/ui/input-group';
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
  const dispatch = createEventDispatcher<{
    send: void;
    stop: void;
    clear: void;
    attach: AttachDetail;
    'toggle-loader-panel': void;
    'toggle-chat-history': void;
  }>();

  export let prompt: string = '';
  export let busy: boolean = false;
  export let isLoaded: boolean = false;
  export let canStop: boolean = false;
  export let isRecording: boolean = false;
  export let supports_text: boolean = true;
  export let supports_image: boolean = false;
  export let supports_audio: boolean = false;
  export let supports_video: boolean = false;
  export let isLoaderPanelVisible: boolean = false;
  export let isChatHistoryVisible: boolean = false;

  let fileInput: HTMLInputElement | null = null;
  let textareaElement: HTMLTextAreaElement | null = null;
  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = null;
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let accept = DEFAULT_TEXT_ACCEPT;
  let attachedFiles: AttachDetail[] = [];
  $: experimentalReady = experimentalFeatures.initialized && experimentalFeatures.enabled;
  $: experimentalStatusMessage = experimentalFeatures.initialized
    ? experimentalFeatures.enabled
      ? null
      : $t('chat.composer.experimental.disabled')
    : $t('chat.composer.experimental.loading');

  // Переменные для автоматического изменения высоты
  let textareaHeight = 34; // Стандартная высота однострочного поля
  const MIN_HEIGHT = 34; // Минимальная высота (как у кнопок)
  const MAX_HEIGHT = 102; // Максимальная высота (3 строки)

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;
    dispatch('send');
  }

  function triggerStop() {
    if (!canStop) return;
    dispatch('stop');
  }

  function triggerClear() {
    if (!prompt && !attachError) return;
    prompt = '';
    attachError = null;
    clearErrorTimer();
    dispatch('clear');
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
    dispatch('toggle-loader-panel');
  }

  function triggerChatHistory() {
    dispatch('toggle-chat-history');
  }

  function removeAttachment(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  function triggerAttach() {
    if (!fileInput) return;
    fileInput.click();
  }

  $: accept = buildAccept();

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
        dispatch('attach', attachment);
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
        dispatch('attach', attachment);
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

  // Реактивное обновление высоты при изменении prompt
  $: if (prompt !== undefined) {
    // Используем setTimeout для корректного обновления после рендера
    setTimeout(() => {
      adjustTextareaHeight();
    }, 0);
  }
</script>

<div class="flex w-full flex-col gap-3">
  {#if attachedFiles.length > 0}
    <div class="flex flex-wrap gap-2">
      {#each attachedFiles as attachment, index}
        <Badge
          variant="secondary"
          class="bg-secondary/40 text-foreground/90 flex items-center gap-2 rounded-lg px-3 py-1 text-xs shadow-sm backdrop-blur"
        >
          <span class="flex items-center gap-1 text-muted-foreground">
            <File size={14} weight="bold" />
            <span class="max-w-[140px] truncate font-medium">{attachment.filename}</span>
          </span>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class="text-muted-foreground hover:text-foreground"
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

  <InputGroup.Root
    class={cn(
      '[--radius:1rem] border border-border/60 bg-card/90 shadow-lg shadow-black/5 backdrop-blur',
      'flex-col'
    )}
  >
    <InputGroup.Textarea
      bind:value={prompt}
      bind:ref={textareaElement}
      placeholder={$t('chat.composer.placeholder')}
      rows={1}
      data-slot="input-group-control"
      class="min-h-[34px] resize-none bg-transparent text-base text-foreground"
      style={`height: ${textareaHeight}px; overflow-y: ${textareaHeight >= MAX_HEIGHT ? 'auto' : 'hidden'};`}
      onkeydown={handleKeydown}
      oninput={handleTextareaInput}
    />

    <InputGroup.Addon
      align="block-end"
      class="w-full flex-col gap-2 border-t border-border/40 pt-2 text-sm text-muted-foreground"
    >
      <div class="flex w-full flex-col gap-2 sm:flex-row sm:items-center">
        <div class="flex flex-1 flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class={cn(
              'transition-colors',
              isChatHistoryVisible && 'bg-primary/15 text-primary hover:bg-primary/20'
            )}
            onclick={triggerChatHistory}
            aria-label={isChatHistoryVisible ? $t('chat.composer.hideHistory') : $t('chat.composer.showHistory')}
            disabled={!experimentalReady}
            title={experimentalStatusMessage ?? undefined}
          >
            <ClockCounterClockwise size={16} weight="bold" />
          </Button>

          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class={cn(
              'transition-colors',
              isLoaderPanelVisible && 'bg-primary/15 text-primary hover:bg-primary/20'
            )}
            onclick={triggerSettings}
            aria-label={$t('chat.composer.loaderSettings')}
          >
            <SlidersHorizontal size={16} weight="bold" />
          </Button>

          {#if prompt || attachError}
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              onclick={triggerClear}
              disabled={busy}
              aria-label={$t('chat.composer.clear')}
            >
              <Broom size={16} weight="bold" />
            </Button>
          {/if}
        </div>

        <div class="flex flex-wrap items-center gap-2 sm:justify-end">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            onclick={triggerAttach}
            disabled={busy || !isLoaded || !experimentalReady}
            aria-label={$t('chat.composer.attach')}
            title={experimentalStatusMessage ?? undefined}
          >
            <Paperclip size={16} weight="bold" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
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
            type="button"
            variant="default"
            size="icon"
            class="shadow-sm"
            onclick={busy ? triggerStop : triggerSend}
            disabled={!isLoaded || (!busy && !prompt.trim())}
            aria-label={busy ? $t('chat.composer.stop') : $t('chat.composer.send')}
          >
            {#if busy}
              <Stop size={16} weight="bold" />
            {:else}
              <ArrowUp size={16} weight="bold" />
            {/if}
          </Button>
        </div>
      </div>
    </InputGroup.Addon>
  </InputGroup.Root>

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
