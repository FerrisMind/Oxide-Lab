<script lang="ts">

  import { createIconManager } from './icon-manager';
  import { onDestroy } from 'svelte';

  let emptyIconEl: HTMLElement | undefined = $state();
  const iconManager = createIconManager();

  // Mount robot icon when component is ready
  $effect(() => {
    if (emptyIconEl) {
      iconManager.mountEmptyIcon(emptyIconEl);
    }
  });

  onDestroy(() => {
    iconManager.cleanup();
  });
</script>

<div class="empty-state">
  <div class="empty-icon" bind:this={emptyIconEl}></div>
  <h3>Выберите модель</h3>
  <p>Выберите модель из списка слева, чтобы увидеть подробную информацию</p>
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--muted);
  }

  .empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1rem;
    color: var(--muted);
  }
  
  .empty-icon :global(svg) {
    color: inherit;
    fill: currentColor;
  }

  .empty-state h3 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
  }
</style>