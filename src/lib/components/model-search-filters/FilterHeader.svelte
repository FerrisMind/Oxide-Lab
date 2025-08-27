<script lang="ts">
  import Funnel from 'phosphor-svelte/lib/Funnel';
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import CaretUp from 'phosphor-svelte/lib/CaretUp';

  export let collapsed: boolean;

  function toggleCollapse() {
    collapsed = !collapsed;
  }
</script>

<div class="filters-header">
  <div class="filters-title">
    <Funnel size={16} />
    <span>Фильтры</span>
  </div>
  <button class="collapse-btn" on:click={toggleCollapse}>
    {#if collapsed}
      <CaretDown size={16} />
    {:else}
      <CaretUp size={16} />
    {/if}
  </button>
</div>

<style>
  .filters-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .filters-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: var(--muted);
     cursor: default;
    padding: 4px;
    border-radius: 4px;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .collapse-btn:hover {
    background: var(--panel-alt-bg);
    color: var(--text);
  }
</style>