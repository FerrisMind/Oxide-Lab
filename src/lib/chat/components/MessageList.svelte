<script lang="ts">
  import type { ChatMessage } from "$lib/chat/types";
  import { registerAssistantBubble } from "$lib/chat/stream_render";
  import SidebarSimple from "phosphor-svelte/lib/SidebarSimple";
  export let messages: ChatMessage[] = [];
  export let messagesEl: HTMLDivElement | null = null;
</script>

<div class="messages-toolbar">
  <button class="toolbar-button left">
    <SidebarSimple size="20" />
  </button>
  <button class="toolbar-button right">
    <SidebarSimple size="20" />
  </button>
</div>

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

<style>
  .messages-toolbar {
    display: flex;
    justify-content: space-between;
    margin-bottom: 4px;
    margin-top: -10px;
    /* Remove negative side margins to avoid horizontal overflow */
    margin-left: 0;
    margin-right: 0;
  }
  
  .toolbar-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text);
  }
  
  .toolbar-button:hover {
    background-color: var(--border-color);
  }
  
  .toolbar-button.left {
    margin-right: auto;
  }
  
  .toolbar-button.right {
    margin-left: auto;
    transform: scaleX(-1);
  }
</style>
