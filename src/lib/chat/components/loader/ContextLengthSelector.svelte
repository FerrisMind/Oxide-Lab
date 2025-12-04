<script lang="ts">
  interface Props {
    ctx_limit_value?: number;
  }

  let { ctx_limit_value = $bindable(4096) }: Props = $props();
</script>

<div class="param">
  <label for="p-ctx">Длина контекста (токены)</label>
  <div class="row">
    <input id="p-ctx" type="range" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
    <input type="number" min={64} max={128000} step={1} bind:value={ctx_limit_value} />
  </div>
</div>