<script lang="ts">
  export let repoId = "";
  export let revision = "";
  export let hubGgufFilename = "";
  export let isSafetensors = false;
</script>

<div class="field">
  <label for="hub-repo">{isSafetensors ? 'HF Hub repoId' : 'HF Hub repoId'}</label>
  <input id="hub-repo" placeholder="owner/repo" bind:value={repoId} />
</div>
<div class="field">
  <label for="hub-rev">revision (опционально)</label>
  <input id="hub-rev" placeholder="main" bind:value={revision} />
</div>
{#if !isSafetensors}
  <div class="field">
    <label for="hub-gguf">filename</label>
    <input id="hub-gguf" placeholder="model-Q4_K_M.gguf" bind:value={hubGgufFilename} />
  </div>
{/if}
{#if isSafetensors}
  <div class="param">
    <div class="hint">Скачиваются tokenizer.json и config.json; веса safetensors кэшируются. Инференс safetensors будет реализован в п.2 Roadmap.</div>
  </div>
{/if}