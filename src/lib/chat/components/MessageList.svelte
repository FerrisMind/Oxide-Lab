<script lang="ts">
  import type { ChatMessage } from '$lib/chat/types';
  import {
    appendSegments,
    finalizeStreaming,
    getAssistantBubbleEl,
    registerAssistantBubble,
  } from '$lib/chat/stream_render';
  import { ConversationEmptyState } from '$lib/components/ai-elements/conversation';
  import InferenceMetricsDisplay from './InferenceMetricsDisplay.svelte';
  import UserMessageActions from './UserMessageActions.svelte';
  import { inferenceMetricsStore } from '$lib/stores/inference-metrics';
  import { Message, MessageContent } from '$lib/components/prompt-kit/message/index.js';
  import Sparkle from 'phosphor-svelte/lib/Sparkle';
  import { t } from '$lib/i18n';

  // Используем Svelte 5 руны для props
  let {
    messages = $bindable([]),
    showModelNotice = false,
  }: {
    messages?: ChatMessage[];
    showModelNotice?: boolean;
  } = $props();

  // Производное значение для placeholderOnly
  let placeholderOnly = $derived(showModelNotice && messages.length === 0);

  // Получаем store с метриками
  let metricsMap = $state(new Map());

  $effect(() => {
    const unsubscribe = inferenceMetricsStore.subscribe((value) => {
      metricsMap = value;
    });
    return unsubscribe;
  });

  // Ре-гидратация ассистентских сообщений при загрузке истории (plain text -> markdown bubble)
  $effect(() => {
    messages.forEach((m, i) => {
      if (m.role !== 'assistant') return;
      if (!m.content || m.content.trim() === '') return;
      const bubble = getAssistantBubbleEl(i);
      if (!bubble) return;
      if (bubble.textContent && bubble.textContent.trim().length > 0) return;

      appendSegments(i, bubble as HTMLDivElement, [{ kind: 'text', data: m.content }], false);
      finalizeStreaming(i);

      // Проверим наличие метрик после ре-гидратации (H7)
      const metrics = metricsMap.get(i);
      if (!metrics) {
      }
    });
  });

  // Функция для получения mdStreamEl для конкретного сообщения
  function getMdStreamEl(messageIndex: number): HTMLElement | null {
    const bubbleEl = getAssistantBubbleEl(messageIndex);
    if (bubbleEl) {
      return bubbleEl.querySelector('.md-stream');
    }
    return null;
  }
</script>

<div
  class="messages"
  class:placeholder-only={placeholderOnly}
  class:with-placeholder={showModelNotice}
>
  {#if showModelNotice}
    <ConversationEmptyState
      title={$t('chat.placeholder.title')}
      description={$t('chat.placeholder.description')}
    >
      {#snippet icon()}
        <Sparkle size={48} weight="duotone" class="text-muted-foreground" />
      {/snippet}
    </ConversationEmptyState>
  {/if}
  {#each messages as m, i}
    <Message class="message {m.role}">
      <div class="bubble">
        {#if m.role === 'assistant'}
          <div class="rich" use:registerAssistantBubble={{ index: i }}></div>
        {:else}
          <MessageContent class="user-content">{m.content}</MessageContent>
        {/if}
      </div>

      <!-- Отображаем метрики производительности под ответом ассистента -->
      {#if m.role === 'assistant'}
        {@const metrics = metricsMap.get(i)}
        {@const mdStreamEl = getMdStreamEl(i)}
        {#if metrics}
          <div class="metrics-container">
            <InferenceMetricsDisplay {metrics} {mdStreamEl} />
          </div>
        {/if}
      {/if}

      <!-- Отображаем кнопку копирования для сообщений пользователя и ассистента -->
      <UserMessageActions messageContent={m.content} />

      <!-- Кнопка будет добавлена здесь через JavaScript -->
    </Message>
  {/each}
</div>

<style>
  .metrics-container {
    display: flex;
    justify-content: flex-start;
    width: 100%;
    max-width: 80%;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  :global(.message.assistant:hover) .metrics-container {
    opacity: 1;
  }

  /* Message using prompt-kit Message component */
  :global(.message) {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    max-width: 100%;
    overflow-wrap: anywhere;
    word-break: break-word;
    margin-bottom: -16px; /* перекрытие между сообщениями */
  }

  /* User messages align right */
  :global(.message.user) {
    align-items: flex-end;
  }

  :global(.message.assistant) {
    align-items: flex-start;
  }

  /* User content styling */
  :global(.user-content) {
    background: var(--panel-bg);
    color: var(--text);
    border-radius: var(--message-border-radius);
    border-top-right-radius: 0;
    max-width: var(--message-content-max-width);
    padding: var(--space-3) var(--space-4);
  }
</style>
