<script lang="ts">
  interface Props {
    models?: any[];
    loading?: boolean;
  }

  let { models = [], loading = false }: Props = $props();
</script>

{#if !loading && models.length === 0}
  <div class="empty-state">
    <p>Модели не найдены</p>
    <span class="empty-hint">Попробуйте изменить параметры поиска</span>
  </div>
{/if}

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: calc(var(--space-12) * 2 + var(--space-2)); /* 200px = 25 units */
    color: var(--muted);
    text-align: center;
  }

  .empty-hint {
    font-size: var(--font-size-sm); /* 14px */
    color: var(--muted);
    opacity: 0.8;
  }
</style>