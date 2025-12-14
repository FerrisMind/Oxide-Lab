<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { getTagColor } from './formatters';

  interface Props {
    model: HFModel;
  }

  let { model }: Props = $props();

  let tagsCollapsed = $state(false);
</script>

{#if model.tags && model.tags.length > 0}
  <div class="model-tags">
    <div class="section-header">
      <button
        class="collapse-btn"
        onclick={() => (tagsCollapsed = !tagsCollapsed)}
        aria-label={tagsCollapsed ? 'Развернуть теги' : 'Свернуть теги'}
      >
        <h3>Теги</h3>
        <span class="collapse-icon" class:collapsed={tagsCollapsed}>▼</span>
      </button>
    </div>
    {#if !tagsCollapsed}
      <div class="tags-container">
        {#each model.tags as tag}
          <span class="tag" style="background-color: {getTagColor(tag)}">
            {tag}
          </span>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .model-tags {
    margin-bottom: var(--space-4); /* 24px */
  }

  .model-tags h3 {
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px → 12px closest */
  }

  .section-header {
    margin-bottom: var(--space-3); /* 16px */
  }

  .collapse-btn {
    background: none;
    border: none;
    cursor: default;
    padding: var(--space-2); /* 8px */
    border-radius: var(--radius-lg); /* 16px */
    transition: all 0.2s ease;
    color: var(--muted);
    display: inline-flex;
    align-items: center;
    gap: var(--space-2); /* 8px */
  }

  .collapse-btn h3 {
    margin: 0;
  }

  .collapse-btn:hover {
    background: var(--panel-alt-bg);
    color: var(--text);
  }

  .collapse-icon {
    display: inline-block;
    transition: transform 0.2s ease;
    font-size: 0.875rem;
  }

  .collapse-icon.collapsed {
    transform: rotate(-90deg);
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2); /* 8px */
    margin-top: var(--space-2); /* 8px */
  }

  .tag {
    display: inline-block;
    padding: var(--space-1) var(--space-3); /* 4px 16px */
    border-radius: var(--radius-lg); /* 16px → 20px closest */
    color: white;
    font-size: 0.875rem;
    font-weight: var(--font-weight-medium);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: all 0.2s ease;
  }

  .tag:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  }
</style>
