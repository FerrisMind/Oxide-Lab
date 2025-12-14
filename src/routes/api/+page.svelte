<script lang="ts">
  // Заглушка страницы API
</script>

<main class="wrap">
  <div class="api-page">
    <div class="page-header">
      <h1>Работа с моделью через API</h1>
      <p>
        Здесь будет интерфейс для отправки запросов к модели через API, тестирование эндпоинтов и
        просмотр ответов.
      </p>
    </div>

    <div class="api-content">
      <div class="card">
        <h2>API Endpoints</h2>
        <p>В разработке...</p>
      </div>
    </div>
  </div>
</main>

<style>
  .api-page {
    max-width: var(--content-max); /* 1200px */
    margin: 0 auto;
    padding: 0 var(--space-3); /* 16px */
  }

  .page-header {
    text-align: center;
    margin-bottom: var(--space-5); /* 32px */
    padding: var(--space-5) 0; /* 32px 0 */
  }

  .page-header h1 {
    font-size: var(--font-size-3xl); /* 40px → 2.5rem */
    font-weight: var(--font-weight-bold);
    color: var(--text);
    margin: 0 0 var(--space-3) 0; /* 16px */
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .page-header p {
    font-size: 1.1rem;
    color: var(--muted);
    margin: 0;
    max-width: 600px; /* 75 units */
    margin: 0 auto;
  }

  .api-content {
    margin-top: var(--space-5); /* 32px */
  }

  /* Адаптивность */
  @media (max-width: 768px) {
    .api-page {
      padding: 0 var(--space-3); /* 16px */
    }

    .page-header h1 {
      font-size: var(--font-size-2xl); /* 32px → 2rem */
    }

    .page-header p {
      font-size: var(--font-size-base); /* 16px → 1rem */
    }
  }
</style>
