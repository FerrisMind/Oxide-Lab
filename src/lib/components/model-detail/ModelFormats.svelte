<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';

  interface Props {
    model: HFModel;
  }

  let { model }: Props = $props();
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
    margin-bottom: var(--space-4); /* 24px */
  }

  .model-formats h3 {
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px → 12px closest */
  }

  .formats-container {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2); /* 8px */
  }

  .format-item {
    padding: var(--space-1) var(--space-3); /* 4px 16px → 6px 16px closest */
    background: var(--panel-alt-bg);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg); /* 16px */
    font-size: 0.875rem;
    font-weight: var(--font-weight-medium);
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
