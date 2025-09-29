<script lang="ts">
  import Copy from 'phosphor-svelte/lib/Copy';
  import Check from 'phosphor-svelte/lib/Check';

  let { messageContent }: { messageContent: string } = $props();

  let copied = $state(false);

  async function copyMessage() {
    try {
      await navigator.clipboard.writeText(messageContent);
      copied = true;
      
      // Возвращаем иконку обратно через 1.5 секунды
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch (error) {
      console.error('Failed to copy message:', error);
    }
  }
</script>

<div class="user-message-actions">
  <button 
    class="copy-button" 
    onclick={copyMessage}
    title="Копировать сообщение"
  >
    {#if copied}
      <Check size={18} weight="regular" />
    {:else}
      <Copy size={18} weight="regular" />
    {/if}
  </button>
</div>


<style>
  .user-message-actions {
    display: flex;
    justify-content: flex-end;
    width: 100%;
    max-width: 80%;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  /* Используем :global для доступа к родительскому элементу */
  :global(.message.user:hover) .user-message-actions {
    opacity: 1;
  }

  .copy-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 0.2s ease;
    color: rgba(0, 0, 0, 0.5);
    margin-top: 0.4rem;
  }

  .copy-button:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.08);
  }

  .copy-button :global(svg) {
    opacity: 1;
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .copy-button {
      color: rgba(255, 255, 255, 0.6);
    }

    .copy-button:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }
</style>
