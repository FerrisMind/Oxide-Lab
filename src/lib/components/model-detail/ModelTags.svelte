<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { getTagColor } from './formatters';

  export let model: HFModel;
  
  let tagsCollapsed = false;
</script>

{#if model.tags && model.tags.length > 0}
  <div class="model-tags">
    <div class="section-header">
      <button 
        class="collapse-btn" 
        on:click={() => tagsCollapsed = !tagsCollapsed}
        aria-label={tagsCollapsed ? 'Развернуть теги' : 'Свернуть теги'}
      >
        <h3>Теги</h3>
        <span class="collapse-icon" class:collapsed={tagsCollapsed}>▼</span>
      </button>
    </div>
    {#if !tagsCollapsed}
      <div class="tags-container">
        {#each model.tags as tag}
          <span 
            class="tag" 
            style="background-color: {getTagColor(tag)}"
          >
            {tag}
          </span>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .model-tags {
    margin-bottom: 24px;
  }

  .model-tags h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 12px 0;
  }

  .section-header {
    margin-bottom: 12px;
  }

  .collapse-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s ease;
    color: var(--muted);
    display: inline-flex;
    align-items: center;
    gap: 8px;
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
    gap: 8px;
    margin-top: 8px;
  }

  .tag {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 20px;
    color: white;
    font-size: 0.875rem;
    font-weight: 500;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: all 0.2s ease;
  }

  .tag:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  }
</style>