<script lang="ts">
  interface Props {
    loading?: boolean;
  }

  let { loading = false }: Props = $props();
</script>

{#if loading}
  <div class="loading-state">
    <div class="loading-spinner"></div>
    <p>Загрузка моделей...</p>
  </div>
{/if}

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: calc(var(--space-12) * 2 + var(--space-2)); /* 200px = 25 units */
    color: var(--muted);
    text-align: center;
  }

  .loading-spinner {
    width: var(--space-5); /* 32px */
    height: var(--space-5); /* 32px */
    border: 3px solid var(--border-color);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--space-3); /* 16px */
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
