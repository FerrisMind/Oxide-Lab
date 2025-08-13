<script lang="ts">
  import type { ChatMessage } from "$lib/chat/types";
  import { registerAssistantBubble } from "$lib/chat/stream_render";
  export let messages: ChatMessage[] = [];
  export let messagesEl: HTMLDivElement | null = null;
</script>

<div class="messages" bind:this={messagesEl}>
  {#if messages.length === 0}
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


