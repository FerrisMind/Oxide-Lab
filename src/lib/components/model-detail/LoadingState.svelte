<div class="loading-state">
  <div class="loading-spinner"></div>
  <p>Загрузка модели...</p>
</div>

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: var(--muted);
  }

  .loading-spinner {
    width: var(--space-5); /* 32px */
    height: var(--space-5); /* 32px */
    border: 3px solid var(--panel-alt-bg);
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
