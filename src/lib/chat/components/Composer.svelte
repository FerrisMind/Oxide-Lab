<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import PaperPlaneRight from "phosphor-svelte/lib/PaperPlaneRight";
  import Stop from "phosphor-svelte/lib/Stop";
  import Paperclip from "phosphor-svelte/lib/Paperclip";
  import Microphone from "phosphor-svelte/lib/Microphone";

  export let prompt = "";
  export let busy = false;
  export let isLoaded = false;
  const dispatch = createEventDispatcher<{ send: void; stop: void; attach: void; voice: void }>();

  function onSend() { dispatch("send"); }
  function onStop() { dispatch("stop"); }
  function onAttach() { dispatch("attach"); }
  function onVoice() { dispatch("voice"); }
</script>

<div class="composer">
  <textarea
    id="chat-input"
    rows="4"
    bind:value={prompt}
    placeholder="Напишите сообщение..."
    on:keydown={(e) => { if ((e as KeyboardEvent).key === 'Enter' && !(e as KeyboardEvent).shiftKey) { e.preventDefault(); onSend(); } }}
  ></textarea>
  <div class="composer-buttons">
    <button class="composer-btn attach-btn" on:click={onAttach} title="Прикрепить файл" disabled={busy}>
      <Paperclip size={18} weight="bold" />
    </button>
    <button class="composer-btn voice-btn" on:click={onVoice} title="Голосовое сообщение" disabled={busy}>
      <Microphone size={18} weight="bold" />
    </button>
    {#if busy}
      <button class="composer-btn stop-btn" on:click={onStop} title="Остановить генерацию">
        <Stop size={20} weight="bold" />
      </button>
    {:else}
      <button class="composer-btn send-btn" on:click={onSend} disabled={!prompt.trim() || !isLoaded} title="Отправить сообщение">
        <PaperPlaneRight size={20} weight="bold" />
      </button>
    {/if}
  </div>
  <slot />
</div>


