<script lang="ts">
  import type { ChatMessage } from "$lib/chat/types";
  import { registerAssistantBubble, getAssistantBubbleEl } from "$lib/chat/stream_render";
  import ChatPlaceholder from "./ChatPlaceholder.svelte";
  import InferenceMetricsDisplay from "./InferenceMetricsDisplay.svelte";
  import UserMessageActions from "./UserMessageActions.svelte";
  import { inferenceMetricsStore } from "$lib/stores/inference-metrics";
  import { t } from '$lib/i18n';

  // Используем Svelte 5 руны для props
  let { 
    messages = $bindable([]), 
    messagesEl = $bindable(null), 
    showModelNotice = false 
  }: {
    messages?: ChatMessage[];
    messagesEl?: HTMLDivElement | null;
    showModelNotice?: boolean;
  } = $props();

  const baseBackground = '#1a1a1a';
  
  // Производное значение для placeholderOnly
  let placeholderOnly = $derived(showModelNotice && messages.length === 0);
  
  // Получаем store с метриками
  let metricsMap = $state(new Map());
  
  $effect(() => {
    const unsubscribe = inferenceMetricsStore.subscribe(value => {
      metricsMap = value;
    });
    return unsubscribe;
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

<style>
  .metrics-container {
    display: flex;
    justify-content: flex-start;
    width: 100%;
    max-width: 80%;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .message.assistant:hover .metrics-container {
    opacity: 1;
  }

</style>

<div
  class="messages"
  bind:this={messagesEl}
  class:placeholder-only={placeholderOnly}
  class:with-placeholder={showModelNotice}
  style:background-color={baseBackground}
>
  {#if showModelNotice}
    <ChatPlaceholder variant="inline" />
  {/if}
  {#if messages.length === 0 && !showModelNotice}
    <div class="empty">{$t('chat.messages.empty')}</div>
  {/if}
  {#each messages as m, i}
    <div class="message {m.role}">
      <div class="bubble">
        {#if m.role === 'assistant'}
          <div class="rich" use:registerAssistantBubble={{ index: i }}></div>
        {:else}
          {m.content}
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

      <!-- Отображаем кнопку копирования для сообщений пользователя -->
      {#if m.role === 'user'}
        <UserMessageActions messageContent={m.content} />
      {/if}
      
      <!-- Кнопка будет добавлена здесь через JavaScript -->
    </div>
  {/each}
</div>
