<script lang="ts">
  import Chat from "../lib/chat/Chat.svelte";
</script>

<div class="page-shell">
  <Chat />
  
</div>

<style>
  .page-shell {
    min-height: 0;
    flex: 1 1 auto;
    display: flex;
    min-width: 0;
  }
  .page-shell :global(main.wrap) {
    flex: 1 1 auto;
    min-height: 0;
  }
</style>


