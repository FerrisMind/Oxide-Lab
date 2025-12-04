<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';

  export let model: HFModel;
</script>

{#if model.formats && model.formats.length > 0}
  <div class="model-formats">
    <h3>Доступные форматы</h3>
    <div class="formats-container">
      {#each model.formats as format}
        <div class="format-item">
          <span class="format-name">{format}</span>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .model-formats {
    margin-bottom: 24px;
  }

  .model-formats h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 12px 0;
  }

  .formats-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .format-item {
    padding: 6px 12px;
    background: var(--panel-alt-bg);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text);
    transition: all 0.2s ease;
  }

  .format-item:hover {
    background: var(--panel-bg);
    border-color: var(--accent);
    transform: translateY(-1px);
  }

  .format-name {
    text-transform: uppercase;
  }
</style>