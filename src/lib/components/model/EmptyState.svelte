<script lang="ts">
  export let models: any[] = [];
  export let loading = false;
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
    height: 200px;
    color: var(--muted);
    text-align: center;
  }

  .empty-hint {
    font-size: 0.875rem;
    color: var(--muted);
    opacity: 0.8;
  }
</style>