<script lang="ts">
  export let enable_thinking = false;
</script>

<div class="param thinking-spacer">
  <div class="row">
    <input id="p-thinking" type="checkbox" bind:checked={enable_thinking} />
    <label for="p-thinking">Включить размышления</label>
  </div>
</div>