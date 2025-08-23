<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  // Redirect to search page since models functionality is there
  onMount(() => {
    goto('/search', { replaceState: true });
  });
</script>

<div class="models-redirect">
  <div class="redirect-message">
    <h2>Перенаправление...</h2>
    <p>Переходим к поиску моделей...</p>
  </div>
</div>

<style>
  .models-redirect {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg);
  }

  .redirect-message {
    text-align: center;
    color: var(--text);
  }

  .redirect-message h2 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .redirect-message p {
    margin: 0;
    color: var(--muted);
    font-size: 1.1rem;
  }
</style>