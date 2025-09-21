<script lang="ts">
  import type { ChatMessage } from "$lib/chat/types";
  import { registerAssistantBubble } from "$lib/chat/stream_render";
  import ChatPlaceholder from "./ChatPlaceholder.svelte";

  export let messages: ChatMessage[] = [];
  export let messagesEl: HTMLDivElement | null = null;
  export let showModelNotice = false;

  const baseBackground = '#2d2d2d';
  $: placeholderOnly = showModelNotice && messages.length === 0;
</script>

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
    <div class="empty">Нет сообщений. Напишите что-нибудь…</div>
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
    </div>
  {/each}
</div>
